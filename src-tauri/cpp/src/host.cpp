#include "host.h"
#include <algorithm>
#include <future>
#include <iostream>

#if defined(_WIN32) || defined(_WIN64)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <objbase.h>
#endif

namespace {
class MonoSumProcessor : public juce::AudioProcessor {
public:
    MonoSumProcessor()
        : juce::AudioProcessor(juce::AudioProcessor::BusesProperties()
            .withInput("Input", juce::AudioChannelSet::stereo(), true)
            .withOutput("Output", juce::AudioChannelSet::stereo(), true)) {}
    ~MonoSumProcessor() override = default;

    const juce::String getName() const override { return "MonoSum"; }
    bool acceptsMidi() const override { return false; }
    bool producesMidi() const override { return false; }
    bool supportsMPE() const override { return false; }
    bool isMidiEffect() const override { return false; }
    double getTailLengthSeconds() const override { return 0.0; }
    int getNumPrograms() override { return 1; }
    int getCurrentProgram() override { return 0; }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override { return {}; }
    void changeProgramName(int, const juce::String&) override {}

    void prepareToPlay(double, int) override {}
    void releaseResources() override {}

    void processBlock(juce::AudioBuffer<float>& buffer, juce::MidiBuffer&) override
    {
        if (buffer.getNumChannels() < 2) return;
        auto* L = buffer.getWritePointer(0);
        auto* R = buffer.getWritePointer(1);
        for (int i = 0; i < buffer.getNumSamples(); ++i)
        {
            float m = std::abs(L[i]) >= std::abs(R[i]) ? L[i] : R[i];
            L[i] = m;
            R[i] = m;
        }
    }

    void getStateInformation(juce::MemoryBlock&) override {}
    void setStateInformation(const void*, int) override {}

    bool hasEditor() const override { return false; }
    juce::AudioProcessorEditor* createEditor() override { return nullptr; }
};
}

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

    monoNode = graph.addNode(std::make_unique<MonoSumProcessor>(),
                             juce::AudioProcessorGraph::NodeID{ 1000002 });

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
    struct Params {
        ShallowHost* host;
        bool mono;
    } params { this, mono };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        if (ps->host->isMono != ps->mono)
        {
            ps->host->isMono = ps->mono;
            ps->host->rebuildConnectionsOnMessageThread();
        }
        return nullptr;
    }, &params);
}

void ShallowHost::rebuildConnectionsOnMessageThread()
{
    if (inputNode == nullptr || outputNode == nullptr) return;
    auto conns = graph.getConnections();
    for (int i = (int)conns.size(); --i >= 0;)
    {
        graph.removeConnection(conns[i]);
    }

    int leftChannel = 0;
    int rightChannel = 1;
    if (isMono)
    {
        leftChannel = 0;
        rightChannel = 0;
        if (auto* device = deviceManager.getCurrentAudioDevice())
        {
            auto mask = device->getActiveInputChannels();
            int ordinalIndex = 0;
            for (int i = 0; i < 32; ++i)
            {
                if (mask[i])
                {
                    leftChannel = ordinalIndex;
                    rightChannel = ordinalIndex;
                    break;
                }
            }
        }
    }

    if (chainNodes.empty())
    {
        if (isMono && monoNode != nullptr)
        {
            graph.addConnection({ { inputNode->nodeID, leftChannel }, { monoNode->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, rightChannel }, { monoNode->nodeID, 1 } });
            graph.addConnection({ { monoNode->nodeID, 0 }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { monoNode->nodeID, 1 }, { outputNode->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { inputNode->nodeID, leftChannel }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, rightChannel }, { outputNode->nodeID, 1 } });
        }
    }
    else
    {
        graph.addConnection({ { inputNode->nodeID, leftChannel }, { chainNodes[0]->nodeID, 0 } });
        graph.addConnection({ { inputNode->nodeID, rightChannel }, { chainNodes[0]->nodeID, 1 } });

        for (size_t i = 0; i < chainNodes.size() - 1; ++i)
        {
            graph.addConnection({ { chainNodes[i]->nodeID, 0 }, { chainNodes[i + 1]->nodeID, 0 } });
            graph.addConnection({ { chainNodes[i]->nodeID, 1 }, { chainNodes[i + 1]->nodeID, 1 } });
        }

        if (isMono && monoNode != nullptr)
        {
            graph.addConnection({ { chainNodes.back()->nodeID, 0 }, { monoNode->nodeID, 0 } });
            graph.addConnection({ { chainNodes.back()->nodeID, 1 }, { monoNode->nodeID, 1 } });
            graph.addConnection({ { monoNode->nodeID, 0 }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { monoNode->nodeID, 1 }, { outputNode->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { chainNodes.back()->nodeID, 0 }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { chainNodes.back()->nodeID, 1 }, { outputNode->nodeID, 1 } });
        }
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
    if (auto* mm = juce::MessageManager::getInstanceWithoutCreating())
    {
        if (!mm->isThisTheMessageThread()) return;
    }
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
    if (!appDataDir.exists())
    {
        appDataDir.createDirectory();
    }

#if defined(_WIN32)
    juce::File sysWmic("C:\\Windows\\System32\\wbem\\WMIC.exe");
    if (!sysWmic.exists())
    {
        juce::File dummyBat = appDataDir.getChildFile("wmic.bat");
        if (!dummyBat.exists())
        {
            dummyBat.replaceWithText("@echo off\r\necho UUID=00000000-0000-0000-0000-000000000000\r\nexit /b 0\r\n");
        }
        juce::File dummyCmd = appDataDir.getChildFile("wmic.cmd");
        if (!dummyCmd.exists())
        {
            dummyBat.copyFileTo(dummyCmd);
        }

        juce::String currentPath = juce::SystemStats::getEnvironmentVariable("PATH", "");
        if (!currentPath.containsIgnoreCase(appDataDir.getFullPathName()))
        {
            juce::String newPath = appDataDir.getFullPathName() + ";" + currentPath;
            SetEnvironmentVariableA("PATH", newPath.toRawUTF8());
        }
    }
#endif

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
    std::promise<void> initPromise;
    auto initFuture = initPromise.get_future();

    g_juceThread = new std::thread([p = std::move(initPromise)]() mutable {
#if defined(_WIN32)
        CoInitializeEx(nullptr, COINIT_MULTITHREADED);
#endif
        juce::ScopedJuceInitialiser_GUI guiInit;
        auto* mm = juce::MessageManager::getInstance();
        p.set_value();
        while (g_juceRunning.load())
        {
            mm->runDispatchLoopUntil(20);
        }
#if defined(_WIN32)
        CoUninitialize();
#endif
    });

    initFuture.wait();
}

static ShallowHost* g_instance = nullptr;

void ShallowHost::shutdown()
{
    if (g_juceThread == nullptr) return;

    if (g_instance != nullptr)
    {
        g_instance->audioStop();

        juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
            auto* h = static_cast<ShallowHost*>(p);
            h->activeWindows.clear();
            h->deviceManager.removeChangeListener(h);
            delete h;
            return nullptr;
        }, g_instance);

        g_instance = nullptr;
    }

    g_juceRunning.store(false);
    if (g_juceThread->joinable())
        g_juceThread->join();
    delete g_juceThread;
    g_juceThread = nullptr;
}

ShallowHost& ShallowHost::getInstance()
{
    if (g_instance == nullptr)
    {
        g_instance = new ShallowHost();
    }
    return *g_instance;
}
