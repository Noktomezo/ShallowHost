#include "host.h"

#include <cmath>

namespace {
void addPaths(juce::FileSearchPath& destination, const juce::var& value)
{
    const auto* paths = value.getArray();
    if (paths == nullptr) return;

    for (const auto& path : *paths)
        destination.add(juce::File(path.toString()));
}

juce::FileSearchPath configuredPath(const juce::var& config, const juce::Identifier& key)
{
    juce::FileSearchPath result;
    if (const auto* object = config.getDynamicObject())
        addPaths(result, object->getProperty(key));
    return result;
}

juce::var pluginsJson(const juce::KnownPluginList& plugins)
{
    juce::Array<juce::var> values;
    for (const auto& description : plugins.getTypes())
    {
        juce::DynamicObject::Ptr plugin = new juce::DynamicObject();
        plugin->setProperty("name", description.name);
        plugin->setProperty("vendor", description.manufacturerName);
        plugin->setProperty("version", description.version);
        plugin->setProperty("category", description.category);
        plugin->setProperty("path", description.fileOrIdentifier);
        plugin->setProperty("unique_id", description.createIdentifierString());
        plugin->setProperty("format", description.pluginFormatName);
        plugin->setProperty("has_editor", true);
        plugin->setProperty("accepts_midi", description.isInstrument);
        values.add(juce::var(plugin.get()));
    }
    return juce::var(values);
}

std::string scanStepJson(bool done, float progress, const juce::KnownPluginList& plugins,
                         const juce::String& current = {})
{
    juce::DynamicObject::Ptr step = new juce::DynamicObject();
    step->setProperty("done", done);
    step->setProperty("progress", juce::jlimit(0.0f, 1.0f, progress));
    step->setProperty("current", current);
    step->setProperty("plugins", pluginsJson(plugins));
    return juce::JSON::toString(juce::var(step.get())).toStdString();
}
}

std::string ShallowHost::startPluginScanJson(const std::string& pluginPathsJson)
{
    pluginScanners.clear();
    pluginScanList.clear();
    pluginScannerIndex = 0;
    pluginScanActive = false;

    struct CacheSnapshot {
        ShallowHost* host;
        juce::Array<juce::PluginDescription> descriptions;
        juce::StringArray blacklist;
    } snapshot { this, {}, {} };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* data) -> void* {
        auto& params = *static_cast<CacheSnapshot*>(data);
        params.descriptions.addArray(params.host->knownPluginList.getTypes());
        params.blacklist.addArray(params.host->knownPluginList.getBlacklistedFiles());
        return nullptr;
    }, &snapshot);

    for (const auto& description : snapshot.descriptions) pluginScanList.addType(description);
    for (const auto& path : snapshot.blacklist) pluginScanList.addToBlacklist(path);

    const auto config = juce::JSON::parse(juce::String(pluginPathsJson));
    auto vst2Path = configuredPath(config, "vst2");
    auto vst3Path = configuredPath(config, "vst3");
    if (config.isArray()) addPaths(vst3Path, config);

    const auto deadMansPedal = appDataDir == juce::File()
        ? juce::File()
        : appDataDir.getChildFile("plugin-scan-dead-mans-pedal.txt");

    for (int index = 0; index < formatManager.getNumFormats(); ++index)
    {
        auto* format = formatManager.getFormat(index);
        if (format == nullptr) continue;

        const auto name = format->getName();
        const auto searchPath = name == "VST" ? vst2Path
            : name == "VST3" ? vst3Path
            : format->getDefaultLocationsToSearch();
        if (searchPath.getNumPaths() == 0) continue;

        pluginScanners.push_back(std::make_unique<juce::PluginDirectoryScanner>(
            pluginScanList, *format, searchPath, true, deadMansPedal));
    }

    pluginScanActive = true;
    if (pluginScanners.empty()) return scanNextPluginJson();
    return scanStepJson(false, 0.0f, pluginScanList);
}

std::string ShallowHost::scanNextPluginJson()
{
    if (! pluginScanActive)
    {
        struct CurrentPlugins {
            const ShallowHost* host;
            std::string json;
        } result { this, {} };
        juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* data) -> void* {
            auto& params = *static_cast<CurrentPlugins*>(data);
            params.json = scanStepJson(true, 1.0f, params.host->knownPluginList);
            return nullptr;
        }, &result);
        return result.json;
    }

    juce::String current;
    if (pluginScannerIndex < pluginScanners.size())
    {
        auto& scanner = *pluginScanners[pluginScannerIndex];
        const bool hasMore = scanner.scanNextFile(true, current);
        if (! hasMore) ++pluginScannerIndex;

        if (pluginScannerIndex < pluginScanners.size())
        {
            const auto formatCount = static_cast<float>(pluginScanners.size());
            auto progress = static_cast<float>(pluginScannerIndex) / formatCount;
            if (hasMore)
            {
                const auto scannerProgress = scanner.getProgress();
                if (std::isfinite(scannerProgress))
                    progress = (static_cast<float>(pluginScannerIndex) + scannerProgress) / formatCount;
            }
            return scanStepJson(false, progress, pluginScanList, current);
        }
    }

    struct FinalResult {
        ShallowHost* host;
        const juce::KnownPluginList* scannedPlugins;
        std::string json;
    } result { this, &pluginScanList, {} };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* data) -> void* {
        auto& params = *static_cast<FinalResult*>(data);
        for (const auto& description : params.scannedPlugins->getTypes())
            params.host->knownPluginList.addType(description);
        for (const auto& path : params.scannedPlugins->getBlacklistedFiles())
            params.host->knownPluginList.addToBlacklist(path);

        const auto knownTypes = params.host->knownPluginList.getTypes();
        for (int index = knownTypes.size() - 1; index >= 0; --index)
        {
            const auto& description = knownTypes.getReference(index);
            if (! juce::File(description.fileOrIdentifier).exists())
                params.host->knownPluginList.removeType(description);
        }

        params.host->saveKnownPlugins();
        params.json = scanStepJson(true, 1.0f, params.host->knownPluginList);
        return nullptr;
    }, &result);

    pluginScanners.clear();
    pluginScanList.clear();
    pluginScannerIndex = 0;
    pluginScanActive = false;
    return result.json;
}

std::string ShallowHost::scanPluginsJson(const std::string& pluginPathsJson)
{
    auto result = startPluginScanJson(pluginPathsJson);
    while (pluginScanActive) result = scanNextPluginJson();

    const auto parsed = juce::JSON::parse(juce::String(result));
    if (const auto* object = parsed.getDynamicObject())
        return juce::JSON::toString(object->getProperty("plugins")).toStdString();
    return "[]";
}
