#include "host.h"
#include <algorithm>
#include <iostream>

#if defined(_WIN32) || defined(_WIN64)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#endif

ShallowHost::ShallowHost()
{
    juce::addDefaultFormatsToManager(formatManager);
    setupGraph();
    player.setProcessor(&graph);
    deviceManager.addChangeListener(this);
    deviceManager.initialise(256, 256, nullptr, true);
    deviceManager.addAudioCallback(&player);
}

ShallowHost::~ShallowHost()
{
    if (juce::MessageManager::getInstanceWithoutCreating() != nullptr)
    {
        deviceManager.removeChangeListener(this);
        audioStop();
        activeWindows.clear();
    }
}

void ShallowHost::setupGraph()
{
    graph.clear();

    inputNode = graph.addNode(
        std::make_unique<juce::AudioProcessorGraph::AudioGraphIOProcessor>(
            juce::AudioProcessorGraph::AudioGraphIOProcessor::audioInputNode),
        juce::AudioProcessorGraph::NodeID{ 1000000 });

    outputNode = graph.addNode(
        std::make_unique<juce::AudioProcessorGraph::AudioGraphIOProcessor>(
            juce::AudioProcessorGraph::AudioGraphIOProcessor::audioOutputNode),
        juce::AudioProcessorGraph::NodeID{ 1000001 });

    rebuildConnections();
}

void ShallowHost::rebuildConnections()
{
    struct Params {
        ShallowHost* host;
    } params { this };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->host->rebuildConnectionsOnMessageThread();
        return nullptr;
    }, &params);
}

void ShallowHost::setMonoMode(bool mono)
{
    if (isMono != mono)
    {
        isMono = mono;
        rebuildConnections();
    }
}

void ShallowHost::rebuildConnectionsOnMessageThread()
{
    if (inputNode == nullptr || outputNode == nullptr) return;
    auto conns = graph.getConnections();
    for (int i = (int)conns.size(); --i >= 0;)
    {
        graph.removeConnection(conns[i]);
    }

    if (chainNodes.empty())
    {
        graph.addConnection({ { inputNode->nodeID, 0 }, { outputNode->nodeID, 0 } });
        if (isMono)
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { outputNode->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { inputNode->nodeID, 1 }, { outputNode->nodeID, 1 } });
        }
    }
    else
    {
        graph.addConnection({ { inputNode->nodeID, 0 }, { chainNodes[0]->nodeID, 0 } });
        if (isMono)
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { chainNodes[0]->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { inputNode->nodeID, 1 }, { chainNodes[0]->nodeID, 1 } });
        }

        for (size_t i = 0; i < chainNodes.size() - 1; ++i)
        {
            graph.addConnection({ { chainNodes[i]->nodeID, 0 }, { chainNodes[i + 1]->nodeID, 0 } });
            graph.addConnection({ { chainNodes[i]->nodeID, 1 }, { chainNodes[i + 1]->nodeID, 1 } });
        }

        graph.addConnection({ { chainNodes.back()->nodeID, 0 }, { outputNode->nodeID, 0 } });
        graph.addConnection({ { chainNodes.back()->nodeID, 1 }, { outputNode->nodeID, 1 } });
    }

    if (auto* device = deviceManager.getCurrentAudioDevice())
    {
        double sr = device->getCurrentSampleRate();
        int bs = device->getCurrentBufferSizeSamples();
        if (sr > 0 && bs > 0)
        {
            graph.suspendProcessing(true);
            graph.prepareToPlay(sr, bs);
            graph.suspendProcessing(false);
        }
    }
}

void ShallowHost::pumpMessageLoop()
{
#if defined(_WIN32) || defined(_WIN64)
    MSG msg;
    while (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE))
    {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
#endif
}

void ShallowHost::setAppDataDirectory(const std::string& path)
{
    appDataDir = juce::File(juce::String(path));
    loadKnownPlugins();
}

void ShallowHost::loadKnownPlugins()
{
    if (appDataDir == juce::File()) return;
    auto file = appDataDir.getChildFile("plugins.xml");
    if (file.existsAsFile())
    {
        if (auto xml = juce::XmlDocument::parse(file))
        {
            knownPluginList.recreateFromXml(*xml);
        }
    }
}

void ShallowHost::saveKnownPlugins()
{
    if (appDataDir == juce::File()) return;
    auto file = appDataDir.getChildFile("plugins.xml");
    if (auto xml = knownPluginList.createXml())
    {
        xml->writeTo(file);
    }
}

static std::thread* g_juceThread = nullptr;
static std::atomic<bool> g_juceRunning{ false };

void ShallowHost::initialize()
{
    if (g_juceThread != nullptr) return;
    g_juceRunning.store(true);
    g_juceThread = new std::thread([]() {
        juce::ScopedJuceInitialiser_GUI guiInit;
        auto* mm = juce::MessageManager::getInstance();
        while (g_juceRunning.load())
        {
            mm->runDispatchLoopUntil(20);
        }
    });
}

void ShallowHost::shutdown()
{
    if (g_juceThread == nullptr) return;

    auto& host = getInstance();
    host.audioStop();
    host.activeWindows.clear();
    host.deviceManager.removeChangeListener(&host);

    g_juceRunning.store(false);
    if (g_juceThread->joinable())
        g_juceThread->join();
    delete g_juceThread;
    g_juceThread = nullptr;
}

ShallowHost& ShallowHost::getInstance()
{
    static ShallowHost instance;
    return instance;
}
