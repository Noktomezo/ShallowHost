#include "host.h"

namespace {
void addPaths(juce::FileSearchPath& destination, const juce::var& value)
{
    const auto* paths = value.getArray();
    if (paths == nullptr) return;

    for (const auto& path : *paths)
    {
        destination.add(juce::File(path.toString()));
    }
}

juce::FileSearchPath configuredPath(const juce::var& config, const juce::Identifier& key)
{
    juce::FileSearchPath result;
    if (const auto* object = config.getDynamicObject())
    {
        addPaths(result, object->getProperty(key));
    }
    return result;
}
}

std::string ShallowHost::scanPluginsJson(const std::string& pluginPathsJson)
{
    const auto config = juce::JSON::parse(juce::String(pluginPathsJson));
    auto vst2Path = configuredPath(config, "vst2");
    auto vst3Path = configuredPath(config, "vst3");

    // Keep the pre-VST2 Rust payload readable while existing config files migrate.
    if (config.isArray()) addPaths(vst3Path, config);

    juce::KnownPluginList scannedPlugins;
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

    for (const auto& description : snapshot.descriptions) scannedPlugins.addType(description);
    for (const auto& path : snapshot.blacklist) scannedPlugins.addToBlacklist(path);

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

        juce::PluginDirectoryScanner scanner(
            scannedPlugins, *format, searchPath, true, deadMansPedal);
        juce::String ignoredName;
        while (scanner.scanNextFile(true, ignoredName)) {}
    }

    struct Result {
        ShallowHost* host;
        const juce::KnownPluginList* scannedPlugins;
        std::string json;
    } result { this, &scannedPlugins, {} };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* data) -> void* {
        auto& params = *static_cast<Result*>(data);
        for (const auto& description : params.scannedPlugins->getTypes())
            params.host->knownPluginList.addType(description);
        for (const auto& path : params.scannedPlugins->getBlacklistedFiles())
            params.host->knownPluginList.addToBlacklist(path);

        const auto knownTypes = params.host->knownPluginList.getTypes();
        for (int index = knownTypes.size() - 1; index >= 0; --index)
        {
            const auto& description = knownTypes.getReference(index);
            if (!juce::File(description.fileOrIdentifier).exists())
                params.host->knownPluginList.removeType(description);
        }

        juce::Array<juce::var> plugins;
        for (const auto& description : params.host->knownPluginList.getTypes())
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
            plugins.add(juce::var(plugin.get()));
        }

        params.host->saveKnownPlugins();
        params.json = juce::JSON::toString(juce::var(plugins)).toStdString();
        return nullptr;
    }, &result);

    return result.json;
}
