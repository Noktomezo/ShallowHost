#include "native_host.h"
#include "host.h"

namespace shallow_host::native {

void init()
{
    ShallowHost::initialize();
}

void shutdown()
{
    ShallowHost::shutdown();
}

void setDataDir(const std::string& path)
{
    ShallowHost::getInstance().setAppDataDirectory(path);
}

bool audioStart(const std::string& driver, const std::string& input, const std::string& output,
                std::int32_t sampleRate, std::int32_t bufferSize, std::int32_t inputMask,
                std::int32_t outputMask, bool mono)
{
    return ShallowHost::getInstance().audioStart(
        driver.c_str(), input.c_str(), output.c_str(), sampleRate, bufferSize,
        inputMask, outputMask, mono);
}

bool audioStop()
{
    return ShallowHost::getInstance().audioStop();
}

AudioLevels audioLevels()
{
    AudioLevels levels {};
    ShallowHost::getInstance().getAudioLevels(levels.input, levels.output);
    return levels;
}

std::string audioDevices(const std::string& driver, const std::string& device)
{
    return ShallowHost::getInstance().getAudioDevicesJson(driver.c_str(), device.c_str());
}

std::string scanPlugins(const std::string& vst3PathsJson)
{
    return ShallowHost::getInstance().scanPluginsJson(vst3PathsJson);
}

std::string startPluginScan(const std::string& pluginPathsJson)
{
    return ShallowHost::getInstance().startPluginScanJson(pluginPathsJson);
}

std::string scanNextPlugin()
{
    return ShallowHost::getInstance().scanNextPluginJson();
}

std::string addToChain(const std::string& uniqueId)
{
    return ShallowHost::getInstance().addToChain(uniqueId);
}

void clearChain()
{
    ShallowHost::getInstance().clearChain();
}

bool removeFromChain(const std::string& nodeId)
{
    return ShallowHost::getInstance().removeFromChain(nodeId);
}

bool reorderChain(const std::string& nodeId, std::int32_t toIndex)
{
    return ShallowHost::getInstance().reorderChain(nodeId, toIndex);
}

bool bypassPlugin(const std::string& nodeId, bool bypassed)
{
    return ShallowHost::getInstance().bypassPlugin(nodeId, bypassed);
}

std::string chain()
{
    return ShallowHost::getInstance().getChainJson();
}

std::string parameters(const std::string& nodeId)
{
    return ShallowHost::getInstance().getPluginParametersJson(nodeId);
}

bool openPluginGui(const std::string& nodeId, const std::string& titlePrefix)
{
    return ShallowHost::getInstance().openPluginGui(nodeId, titlePrefix);
}

std::string saveState()
{
    return ShallowHost::getInstance().saveStateJson();
}

bool loadState(const std::string& state)
{
    return ShallowHost::getInstance().loadStateJson(state);
}

std::uint64_t stateRevision()
{
    return ShallowHost::getInstance().getStateRevision();
}

void setMonoMode(bool mono)
{
    ShallowHost::getInstance().setMonoMode(mono);
}

} // namespace shallow_host::native
