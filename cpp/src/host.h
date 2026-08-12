#pragma once

#if defined(_WIN32)
  #if defined(SHALLOW_HOST_DLL_BUILD)
    #define SHALLOW_HOST_API __declspec(dllexport)
  #else
    #define SHALLOW_HOST_API
  #endif
#else
  #define SHALLOW_HOST_API
#endif

#include <juce_audio_basics/juce_audio_basics.h>
#include <juce_audio_devices/juce_audio_devices.h>
#include <juce_audio_formats/juce_audio_formats.h>
#include <juce_audio_processors/juce_audio_processors.h>
#include <juce_audio_utils/juce_audio_utils.h>
#include <juce_core/juce_core.h>
#include <juce_data_structures/juce_data_structures.h>
#include <juce_events/juce_events.h>
#include <juce_graphics/juce_graphics.h>
#include <juce_gui_basics/juce_gui_basics.h>
#include <juce_gui_extra/juce_gui_extra.h>

#include <string>
#include <vector>
#include <memory>
#include <mutex>
#include <unordered_map>
#include <unordered_set>
#include <atomic>
#include <algorithm>
#include <cmath>
#include <cstdint>

// ponytail: subclass AudioProcessorPlayer to add ScopedNoDenormals before the
// entire plugin chain processes. JUCE doesn't auto-handle denormals — without
// this, decaying signals from plugins produce subnormal floats that trigger
// 100x CPU spikes → audio dropouts. ScopedNoDenormals sets FTZ+DAZ CPU flags
// for the audio thread scope.
class DenormalsPlayer : public juce::AudioProcessorPlayer {
public:
    using AudioProcessorPlayer::AudioProcessorPlayer;
    std::atomic<float> inputPeak{ 0.0f };
    std::atomic<float> outputPeak{ 0.0f };

    void audioDeviceAboutToStart(juce::AudioIODevice* device) override
    {
        juce::AudioProcessorPlayer::audioDeviceAboutToStart(device);
    }

    void audioDeviceIOCallbackWithContext(const float* const* inputChannelData, int numInputChannels,
                                          float* const* outputChannelData, int numOutputChannels,
                                          int numSamples, const juce::AudioIODeviceCallbackContext& context) override
    {
        juce::ScopedNoDenormals denormals;
        float inMax = 0.0f;
        if (inputChannelData != nullptr)
        {
            for (int c = 0; c < numInputChannels; ++c)
            {
                if (const float* buf = inputChannelData[c])
                {
                    const auto range = juce::FloatVectorOperations::findMinAndMax(buf, numSamples);
                    inMax = std::max(inMax, std::max(std::abs(range.getStart()),
                                                     std::abs(range.getEnd())));
                }
            }
        }

        juce::AudioProcessorPlayer::audioDeviceIOCallbackWithContext(
            inputChannelData, numInputChannels, outputChannelData, numOutputChannels, numSamples, context);

        float outMax = 0.0f;
        if (outputChannelData != nullptr)
        {
            for (int c = 0; c < numOutputChannels; ++c)
            {
                if (const float* buf = outputChannelData[c])
                {
                    const auto range = juce::FloatVectorOperations::findMinAndMax(buf, numSamples);
                    outMax = std::max(outMax, std::max(std::abs(range.getStart()),
                                                       std::abs(range.getEnd())));
                }
            }
        }

        updatePeak(inputPeak, inMax);
        updatePeak(outputPeak, outMax);
    }

    void getAudioLevels(float& inPeak, float& outPeak)
    {
        inPeak = inputPeak.exchange(0.0f, std::memory_order_relaxed);
        outPeak = outputPeak.exchange(0.0f, std::memory_order_relaxed);
    }

private:
    static void updatePeak(std::atomic<float>& peak, float candidate) noexcept
    {
        auto current = peak.load(std::memory_order_relaxed);
        while (candidate > current
               && ! peak.compare_exchange_weak(current, candidate,
                                                std::memory_order_relaxed,
                                                std::memory_order_relaxed))
        {
        }
    }
};
class SHALLOW_HOST_API ShallowHost : public juce::ChangeListener,
                                    private juce::AudioProcessorListener {
public:
    static void initialize();
    static void shutdown();
    static ShallowHost& getInstance();

    void setAppDataDirectory(const std::string& path);

    int audioStart(const char* driver, const char* inputDevice, const char* outputDevice,
                   int sampleRate, int bufferSize, int inputMask, int outputMask, bool mono);
    int audioStop();
    void getAudioLevels(float& inPeak, float& outPeak) { player.getAudioLevels(inPeak, outPeak); }

    int audioStartOnMessageThread(const char* driver, const char* inputDevice, const char* outputDevice,
                                 int sampleRate, int bufferSize, int inputMask, int outputMask, bool mono);
    int audioStopOnMessageThread();

    std::string getAudioDevicesJson(const char* driver = nullptr, const char* deviceName = nullptr);
    std::string scanPluginsJson(const std::string& pluginPathsJson = "{}");
    std::string startPluginScanJson(const std::string& pluginPathsJson);
    std::string scanNextPluginJson();

    std::string addToChain(const std::string& uniqueId);
    bool removeFromChain(const std::string& nodeId);
    bool movePlugin(const std::string& nodeId, bool up);
    bool reorderChain(const std::string& nodeId, int toIndex);
    bool bypassPlugin(const std::string& nodeId, bool bypassed);
    std::string getChainJson();

    std::string getPluginParametersJson(const std::string& nodeId);
    bool setPluginParameter(const std::string& nodeId, int paramIndex, float value);

    bool openPluginGui(const std::string& nodeId, const std::string& titlePrefix = "");
    bool closePluginGui(const std::string& nodeId);
    bool isPluginGuiOpen(const std::string& nodeId) const;

    std::string addToChainWithState(const std::string& uniqueId, const std::string& base64State, bool bypassed);
    void clearChain();

    std::string saveStateJson();
    bool loadStateJson(const std::string& stateJson);
    std::uint64_t getStateRevision() const noexcept
    {
        return stateRevision.load(std::memory_order_relaxed);
    }

    void setMonoMode(bool mono);
    bool getMonoMode() const { return isMono; }

    juce::AudioPluginFormatManager& getFormatManager() { return formatManager; }

    void changeListenerCallback(juce::ChangeBroadcaster* source) override;

private:
    ShallowHost();
    ~ShallowHost();

    juce::AudioDeviceManager deviceManager;
    juce::AudioPluginFormatManager formatManager;
    juce::AudioProcessorGraph graph;
    DenormalsPlayer player;

    juce::AudioProcessorGraph::Node::Ptr inputNode;
    juce::AudioProcessorGraph::Node::Ptr outputNode;
    juce::AudioProcessorGraph::Node::Ptr monoNode;

    std::vector<juce::AudioProcessorGraph::Node::Ptr> chainNodes;
    juce::KnownPluginList knownPluginList;
    juce::KnownPluginList pluginScanList;
    std::vector<std::unique_ptr<juce::PluginDirectoryScanner>> pluginScanners;
    std::size_t pluginScannerIndex = 0;
    int pluginScanPublishedCount = 0;
    bool pluginScanActive = false;
    bool isMono = false;

    class PluginWindow : public juce::DocumentWindow {
    public:
        PluginWindow(const std::string& nodeId_, const juce::String& name, std::unique_ptr<juce::AudioProcessorEditor> editor)
            : DocumentWindow(name, juce::Colours::darkgrey, DocumentWindow::allButtons),
              nodeId(nodeId_)
        {
            setContentOwned(editor.release(), true);
            setUsingNativeTitleBar(true);
            setVisible(true);
        }
        void closeButtonPressed() override
        {
            juce::String idCopy = nodeId;
            juce::MessageManager::callAsync([idCopy]() {
                ShallowHost::getInstance().closePluginGui(idCopy.toStdString());
            });
        }
    private:
        std::string nodeId;
    };

    std::unordered_map<std::string, std::unique_ptr<PluginWindow>> activeWindows;
    mutable std::mutex pluginGuiStatusMutex;
    std::unordered_set<std::string> openPluginGuiIds;
    std::unordered_set<std::string> scannedDeviceTypes;
    std::atomic<std::uint64_t> stateRevision{ 0 };

    juce::File appDataDir;
    void setAppDataDirectoryOnMessageThread(const std::string& path);
    void loadKnownPlugins();
    void saveKnownPlugins();

    void setupGraph();
    void rebuildConnectionsOnMessageThread();

    bool openPluginGuiOnMessageThread(const std::string& nodeId, const std::string& titlePrefix = "");
    bool closePluginGuiOnMessageThread(const std::string& nodeId);
    void markPluginGuiOpen(const std::string& nodeId);
    void markPluginGuiClosed(const std::string& nodeId);
    void clearPluginGuiStatus();

    std::string addToChainOnMessageThread(const std::string& uniqueId, const std::string& base64State = "", bool bypassed = false);
    bool removeFromChainOnMessageThread(const std::string& nodeId);
    bool movePluginOnMessageThread(const std::string& nodeId, bool up);
    bool reorderChainOnMessageThread(const std::string& nodeId, int toIndex);
    bool bypassPluginOnMessageThread(const std::string& nodeId, bool bypassed);
    bool loadStateJsonOnMessageThread(const std::string& stateJson);

    std::string getPluginParametersJsonOnMessageThread(const std::string& nodeId);
    bool setPluginParameterOnMessageThread(const std::string& nodeId, int paramIndex, float value);

    void audioProcessorParameterChanged(juce::AudioProcessor*, int, float) override;
    void audioProcessorChanged(juce::AudioProcessor*,
                               const juce::AudioProcessorListener::ChangeDetails&) override;

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(ShallowHost)
};
