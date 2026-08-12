#include "host.h"
#include <algorithm>
#include <iostream>
#include <future>

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
        const auto sampleCount = buffer.getNumSamples();
        buffer.addFrom(0, 0, buffer, 1, 0, sampleCount);
        buffer.applyGain(0, 0, sampleCount, 0.5f);
        buffer.copyFrom(1, 0, buffer, 0, 0, sampleCount);
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
    const auto error = deviceManager.initialise(0, 0, nullptr, false);
    if (error.isNotEmpty())
    {
        std::cerr << "[sh] audio device manager initialization failed: "
                  << error.toStdString() << std::endl;
    }
    deviceManager.addAudioCallback(&player);
}

ShallowHost::~ShallowHost()
{
    jassert(juce::MessageManager::getInstance()->isThisTheMessageThread());
    activeWindows.clear();
    clearPluginGuiStatus();
    deviceManager.removeAudioCallback(&player);
    deviceManager.removeChangeListener(this);
    audioStopOnMessageThread();
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

    rebuildConnectionsOnMessageThread();
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

    using Connection = juce::AudioProcessorGraph::Connection;
    using Node = juce::AudioProcessorGraph::Node;
    using UpdateKind = juce::AudioProcessorGraph::UpdateKind;

    auto conns = graph.getConnections();
    for (int i = (int)conns.size(); --i >= 0;)
    {
        graph.removeConnection(conns[i], UpdateKind::none);
    }

    const auto connect = [this](const Node::Ptr& source, int sourceChannel,
                                const Node::Ptr& destination, int destinationChannel)
    {
        if (source == nullptr || destination == nullptr) return;
        graph.addConnection(Connection{
            { source->nodeID, sourceChannel },
            { destination->nodeID, destinationChannel }
        }, UpdateKind::none);
    };

    const auto connectStages = [&connect](const Node::Ptr& source, const Node::Ptr& destination)
    {
        if (source == nullptr || destination == nullptr) return;
        const auto* sourceProcessor = source->getProcessor();
        const auto* destinationProcessor = destination->getProcessor();
        if (sourceProcessor == nullptr || destinationProcessor == nullptr) return;

        const int outputs = sourceProcessor->getTotalNumOutputChannels();
        const int inputs = destinationProcessor->getTotalNumInputChannels();
        if (outputs <= 0 || inputs <= 0) return;

        if (outputs == 1)
        {
            for (int input = 0; input < std::min(inputs, 2); ++input)
                connect(source, 0, destination, input);
            return;
        }

        if (inputs == 1)
        {
            connect(source, 0, destination, 0);
            return;
        }

        connect(source, 0, destination, 0);
        connect(source, 1, destination, 1);
    };

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
            connect(inputNode, leftChannel, monoNode, 0);
            connect(inputNode, rightChannel, monoNode, 1);
            connectStages(monoNode, outputNode);
        }
        else
        {
            connect(inputNode, leftChannel, outputNode, 0);
            connect(inputNode, rightChannel, outputNode, 1);
        }
    }
    else
    {
        auto firstNode = chainNodes.front();
        if (auto* firstProcessor = firstNode->getProcessor())
        {
            const int inputs = firstProcessor->getTotalNumInputChannels();
            if (inputs > 0)
            {
                connect(inputNode, leftChannel, firstNode, 0);
                if (inputs > 1)
                    connect(inputNode, rightChannel, firstNode, 1);
            }
        }

        for (size_t i = 0; i < chainNodes.size() - 1; ++i)
        {
            connectStages(chainNodes[i], chainNodes[i + 1]);
        }

        if (isMono && monoNode != nullptr)
        {
            connectStages(chainNodes.back(), monoNode);
            connectStages(monoNode, outputNode);
        }
        else
        {
            connectStages(chainNodes.back(), outputNode);
        }
    }

    graph.rebuild();
}

void ShallowHost::setAppDataDirectory(const std::string& path)
{
    struct Params {
        ShallowHost* host;
        const std::string* path;
    } params { this, &path };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->host->setAppDataDirectoryOnMessageThread(*ps->path);
        return nullptr;
    }, &params);
}

void ShallowHost::setAppDataDirectoryOnMessageThread(const std::string& path)
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
        juce::TemporaryFile temporary(file);
        if (!xml->writeTo(temporary.getFile())
            || !temporary.overwriteTargetFileWithTemporary())
        {
            std::cerr << "[sh] failed to save plugin cache: "
                      << file.getFullPathName().toStdString() << std::endl;
        }
    }
}

static ShallowHost* g_instance = nullptr;
static std::thread* g_juceThread = nullptr;
static std::atomic<bool> g_juceRunning{ false };

void ShallowHost::initialize()
{
    if (g_juceThread != nullptr) return;
    g_juceRunning.store(true);
    std::promise<void> initPromise;
    auto initFuture = initPromise.get_future();

    g_juceThread = new std::thread([p = std::move(initPromise)]() mutable {
        juce::ScopedJuceInitialiser_GUI guiInit;
        auto* mm = juce::MessageManager::getInstance();
        g_instance = new ShallowHost();
        p.set_value();
        while (g_juceRunning.load())
        {
            mm->runDispatchLoopUntil(20);
        }
    });

    initFuture.wait();
}

void ShallowHost::shutdown()
{
    if (g_juceThread == nullptr) return;

    if (g_instance != nullptr)
    {
        juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
            auto* h = static_cast<ShallowHost*>(p);
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
    jassert(g_instance != nullptr);
    return *g_instance;
}
