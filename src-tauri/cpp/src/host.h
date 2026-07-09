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

// ponytail: subclass AudioProcessorPlayer to add ScopedNoDenormals before the
// entire plugin chain processes. JUCE doesn't auto-handle denormals — without
// this, decaying signals from plugins produce subnormal floats that trigger
// 100x CPU spikes → audio dropouts. ScopedNoDenormals sets FTZ+DAZ CPU flags
// for the audio thread scope.
class DenormalsPlayer : public juce::AudioProcessorPlayer {
public:
    using AudioProcessorPlayer::AudioProcessorPlayer;
    void audioDeviceIOCallbackWithContext(const float* const* inputChannelData, int numInputChannels,
                                          float* const* outputChannelData, int numOutputChannels,
                                          int numSamples, const juce::AudioIODeviceCallbackContext& context) override
    {
        juce::ScopedNoDenormals denormals;
        juce::AudioProcessorPlayer::audioDeviceIOCallbackWithContext(
            inputChannelData, numInputChannels, outputChannelData, numOutputChannels, numSamples, context);
    }
};

class SHALLOW_HOST_API ShallowHost : public juce::ChangeListener {
public:
    static void initialize();
    static void shutdown();
    static ShallowHost& getInstance();

    void setAppDataDirectory(const std::string& path);

    int audioStart(const char* driver, const char* inputDevice, const char* outputDevice,
                   int sampleRate, int bufferSize, int mono, int inputMask = 0, int outputMask = 0);
    int audioStop();

    int audioStartOnMessageThread(const char* driver, const char* inputDevice, const char* outputDevice,
                                 int sampleRate, int bufferSize, int mono, int inputMask = 0, int outputMask = 0);
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

    bool openPluginGui(const std::string& nodeId);
    bool closePluginGui(const std::string& nodeId);

    std::string saveStateJson();
    bool loadStateJson(const std::string& stateJson);

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
        void closeButtonPressed() override;
    private:
        std::string nodeId;
    };

    std::unordered_map<std::string, std::unique_ptr<PluginWindow>> activeWindows;

    juce::File appDataDir;
    bool monoMode = false;
    void loadKnownPlugins();
    void saveKnownPlugins();

    void setupGraph();
    void rebuildConnections();
    void rebuildConnectionsOnMessageThread();

    bool openPluginGuiOnMessageThread(const std::string& nodeId);
    bool closePluginGuiOnMessageThread(const std::string& nodeId);

    std::string addToChainOnMessageThread(const std::string& uniqueId);
    bool removeFromChainOnMessageThread(const std::string& nodeId);
    bool movePluginOnMessageThread(const std::string& nodeId, bool up);
    bool reorderChainOnMessageThread(const std::string& nodeId, int toIndex);
    bool loadStateJsonOnMessageThread(const std::string& stateJson);

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
SH_EXPORT bool sh_audio_start(const char* driver, const char* input, const char* output, int sample_rate, int buffer_size, bool mono, int input_mask, int output_mask);
SH_EXPORT bool sh_audio_stop();

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

SH_EXPORT bool sh_open_plugin_gui(const char* node_id);
SH_EXPORT bool sh_close_plugin_gui(const char* node_id);

SH_EXPORT char* sh_save_state();
SH_EXPORT bool sh_load_state(const char* state);

SH_EXPORT void sh_free_string(char* ptr);
