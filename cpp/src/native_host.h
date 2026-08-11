#pragma once

#include <cstdint>
#include <string>

namespace shallow_host::native {

struct AudioLevels {
    float input;
    float output;
};

void init();
void shutdown();
void setDataDir(const std::string& path);
bool audioStart(const std::string& driver, const std::string& input, const std::string& output,
                std::int32_t sampleRate, std::int32_t bufferSize, std::int32_t inputMask,
                std::int32_t outputMask, bool mono);
bool audioStop();
AudioLevels audioLevels();
std::string audioDevices(const std::string& driver, const std::string& device);
std::string scanPlugins(const std::string& vst3PathsJson);
std::string startPluginScan(const std::string& pluginPathsJson);
std::string scanNextPlugin();
std::string addToChain(const std::string& uniqueId);
void clearChain();
bool removeFromChain(const std::string& nodeId);
bool reorderChain(const std::string& nodeId, std::int32_t toIndex);
bool bypassPlugin(const std::string& nodeId, bool bypassed);
std::string chain();
std::string parameters(const std::string& nodeId);
bool openPluginGui(const std::string& nodeId, const std::string& titlePrefix);
std::string saveState();
bool loadState(const std::string& state);
std::uint64_t stateRevision();
void setMonoMode(bool mono);

} // namespace shallow_host::native
