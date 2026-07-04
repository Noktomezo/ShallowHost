#include "host.h"
#include <cstring>
#include <cstdlib>

// Helper to duplicate std::string to C-string using _strdup (Windows standard)
static char* copyToC(const std::string& str)
{
    return _strdup(str.c_str());
}

SH_EXPORT void sh_init()
{
    ShallowHost::initialize();
}

SH_EXPORT void sh_shutdown()
{
    ShallowHost::shutdown();
}

SH_EXPORT void sh_set_data_dir(const char* path)
{
    if (path)
    {
        ShallowHost::getInstance().setAppDataDirectory(std::string(path));
    }
}

SH_EXPORT bool sh_audio_start(const char* driver, const char* input, const char* output, int sample_rate, int buffer_size, bool mono, int input_mask, int output_mask)
{
    return ShallowHost::getInstance().audioStart(
        driver ? driver : "",
        input ? input : "",
        output ? output : "",
        sample_rate,
        buffer_size,
        mono,
        input_mask,
        output_mask
    );
}

SH_EXPORT bool sh_audio_stop()
{
    return ShallowHost::getInstance().audioStop();
}

SH_EXPORT char* sh_get_audio_devices(const char* driver, const char* device_name)
{
    return copyToC(ShallowHost::getInstance().getAudioDevicesJson(driver, device_name));
}

SH_EXPORT char* sh_scan_plugins()
{
    return copyToC(ShallowHost::getInstance().scanPluginsJson());
}

SH_EXPORT char* sh_add_to_chain(const char* unique_id)
{
    if (!unique_id) return nullptr;
    return copyToC(ShallowHost::getInstance().addToChain(std::string(unique_id)));
}

SH_EXPORT bool sh_remove_from_chain(const char* node_id)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().removeFromChain(std::string(node_id));
}

SH_EXPORT bool sh_move_plugin(const char* node_id, bool up)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().movePlugin(std::string(node_id), up);
}

SH_EXPORT bool sh_reorder_chain(const char* node_id, int to_index)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().reorderChain(std::string(node_id), to_index);
}

SH_EXPORT bool sh_bypass_plugin(const char* node_id, bool bypassed)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().bypassPlugin(std::string(node_id), bypassed);
}

SH_EXPORT char* sh_get_chain()
{
    return copyToC(ShallowHost::getInstance().getChainJson());
}

SH_EXPORT char* sh_get_plugin_parameters(const char* node_id)
{
    if (!node_id) return nullptr;
    return copyToC(ShallowHost::getInstance().getPluginParametersJson(std::string(node_id)));
}

SH_EXPORT bool sh_set_plugin_parameter(const char* node_id, int param_index, float value)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().setPluginParameter(std::string(node_id), param_index, value);
}

SH_EXPORT bool sh_open_plugin_gui(const char* node_id)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().openPluginGui(std::string(node_id));
}

SH_EXPORT bool sh_close_plugin_gui(const char* node_id)
{
    if (!node_id) return false;
    return ShallowHost::getInstance().closePluginGui(std::string(node_id));
}

SH_EXPORT char* sh_save_state()
{
    return copyToC(ShallowHost::getInstance().saveStateJson());
}

SH_EXPORT bool sh_load_state(const char* state)
{
    if (!state) return false;
    return ShallowHost::getInstance().loadStateJson(std::string(state));
}

SH_EXPORT void sh_free_string(char* ptr)
{
    if (ptr)
    {
        free(ptr);
    }
}
