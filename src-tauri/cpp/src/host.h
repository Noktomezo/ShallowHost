#pragma once

#if defined(_WIN32)
  #if defined(SHALLOW_HOST_DLL_BUILD)
    #define SHALLOW_HOST_API __declspec(dllexport)
  #else
    #define SHALLOW_HOST_API __declspec(dllimport)
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
#include <unordered_map>
#include <atomic>
#include <algorithm>

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

    void audioDeviceIOCallbackWithContext(const float* const* inputChannelData, int numInputChannels,
                                          float* const* outputChannelData, int numOutputChannels,
                                          int numSamples, const juce::AudioIODeviceCallbackContext& context) override
    {
        juce::ScopedNoDenormals denormals;
        juce::AudioProcessorPlayer::audioDeviceIOCallbackWithContext(
            inputChannelData, numInputChannels, outputChannelData, numOutputChannels, numSamples, context);

        float inMax = 0.0f;
        if (inputChannelData != nullptr)
        {
            for (int c = 0; c < numInputChannels; ++c)
            {
                if (const float* buf = inputChannelData[c])
                {
                    for (int s = 0; s < numSamples; ++s)
                    {
                        float mag = std::abs(buf[s]);
                        if (mag > inMax) inMax = mag;
                    }
                }
            }
        }

        float outMax = 0.0f;
        if (outputChannelData != nullptr)
        {
            for (int c = 0; c < numOutputChannels; ++c)
            {
                if (const float* buf = outputChannelData[c])
                {
                    for (int s = 0; s < numSamples; ++s)
                    {
                        float mag = std::abs(buf[s]);
                        if (mag > outMax) outMax = mag;
                    }
                }
            }
        }

        float curIn = inputPeak.load(std::memory_order_relaxed);
        inputPeak.store(std::max(inMax, curIn * 0.94f), std::memory_order_relaxed);

        float curOut = outputPeak.load(std::memory_order_relaxed);
        outputPeak.store(std::max(outMax, curOut * 0.94f), std::memory_order_relaxed);
    }

    void getAudioLevels(float& inPeak, float& outPeak)
    {
        inPeak = inputPeak.load(std::memory_order_relaxed);
        outPeak = outputPeak.load(std::memory_order_relaxed);
        inputPeak.store(inPeak * 0.88f, std::memory_order_relaxed);
        outputPeak.store(outPeak * 0.88f, std::memory_order_relaxed);
    }
};

class SHALLOW_HOST_API ShallowHost : public juce::ChangeListener {
public:
    static void initialize();
    static void shutdown();
    static ShallowHost& getInstance();

    void setAppDataDirectory(const std::string& path);

    int audioStart(const char* driver, const char* inputDevice, const char* outputDevice,
                   int sampleRate, int bufferSize, int inputMask = -1, int outputMask = -1);
    int audioStop();
    void getAudioLevels(float& inPeak, float& outPeak) { player.getAudioLevels(inPeak, outPeak); }

    int audioStartOnMessageThread(const char* driver, const char* inputDevice, const char* outputDevice,
                                 int sampleRate, int bufferSize, int inputMask = -1, int outputMask = -1);
    int audioStopOnMessageThread();

    std::string getAudioDevicesJson(const char* driver = nullptr, const char* deviceName = nullptr);
    std::string scanPluginsJson(const std::string& vst2PathsJson = "[]", const std::string& vst3PathsJson = "[]");

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

    std::string addToChainWithState(const std::string& uniqueId, const std::string& base64State, bool bypassed);
    void clearChain();

    std::string saveStateJson();
    bool loadStateJson(const std::string& stateJson);

    void setMonoMode(bool mono);
    bool getMonoMode() const { return isMono; }

    void pumpMessageLoop();

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

    std::vector<juce::AudioProcessorGraph::Node::Ptr> chainNodes;
    juce::KnownPluginList knownPluginList;
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

    juce::File appDataDir;
    void loadKnownPlugins();
    void saveKnownPlugins();

    void setupGraph();
    void rebuildConnections();
    void rebuildConnectionsOnMessageThread();

    bool openPluginGuiOnMessageThread(const std::string& nodeId, const std::string& titlePrefix = "");
    bool closePluginGuiOnMessageThread(const std::string& nodeId);

    std::string addToChainOnMessageThread(const std::string& uniqueId, const std::string& base64State = "", bool bypassed = false);
    bool removeFromChainOnMessageThread(const std::string& nodeId);
    bool movePluginOnMessageThread(const std::string& nodeId, bool up);
    bool reorderChainOnMessageThread(const std::string& nodeId, int toIndex);
    bool bypassPluginOnMessageThread(const std::string& nodeId, bool bypassed);
    bool loadStateJsonOnMessageThread(const std::string& stateJson);

    std::string getPluginParametersJsonOnMessageThread(const std::string& nodeId);
    bool setPluginParameterOnMessageThread(const std::string& nodeId, int paramIndex, float value);

    JUCE_DECLARE_NON_COPYABLE_WITH_LEAK_DETECTOR(ShallowHost)
};

#ifdef SHALLOW_HOST_DLL_BUILD
#define SH_EXPORT extern "C" __declspec(dllexport)
#else
#define SH_EXPORT extern "C"
#endif

SH_EXPORT void sh_init();
SH_EXPORT void sh_shutdown();
SH_EXPORT void sh_set_data_dir(const char* path);
SH_EXPORT bool sh_audio_start(const char* driver, const char* input, const char* output, int sample_rate, int buffer_size, int input_mask, int output_mask);
SH_EXPORT bool sh_audio_stop();
SH_EXPORT void sh_get_audio_levels(float* in_peak, float* out_peak);

SH_EXPORT char* sh_get_audio_devices(const char* driver, const char* device_name);
SH_EXPORT char* sh_scan_plugins(const char* vst2_paths_json, const char* vst3_paths_json);

SH_EXPORT char* sh_add_to_chain(const char* unique_id);
SH_EXPORT bool sh_remove_from_chain(const char* node_id);
SH_EXPORT bool sh_move_plugin(const char* node_id, bool up);
SH_EXPORT bool sh_reorder_chain(const char* node_id, int to_index);
SH_EXPORT bool sh_bypass_plugin(const char* node_id, bool bypassed);
SH_EXPORT char* sh_get_chain();

SH_EXPORT char* sh_get_plugin_parameters(const char* node_id);
SH_EXPORT bool sh_set_plugin_parameter(const char* node_id, int param_index, float value);

SH_EXPORT bool sh_open_plugin_gui(const char* node_id, const char* title_prefix);
SH_EXPORT bool sh_close_plugin_gui(const char* node_id);

SH_EXPORT char* sh_save_state();
SH_EXPORT bool sh_load_state(const char* state);
SH_EXPORT void sh_set_mono_mode(bool mono);

SH_EXPORT void sh_free_string(char* ptr);
