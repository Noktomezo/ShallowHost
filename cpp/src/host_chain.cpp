#include "host.h"
#include <algorithm>
#include <iostream>

std::string ShallowHost::scanPluginsJson(const std::string& vst3PathsJson)
{
    juce::FileSearchPath vst3Path;
    juce::var vst3Arr = juce::JSON::parse(juce::String(vst3PathsJson));
    if (vst3Arr.isArray())
    {
        for (int i = 0; i < vst3Arr.getArray()->size(); ++i)
        {
            vst3Path.add(juce::File(vst3Arr.getArray()->getReference(i).toString()));
        }
    }
    else
    {
        for (int i = 0; i < formatManager.getNumFormats(); ++i)
        {
            if (auto* fmt = formatManager.getFormat(i))
            {
                if (fmt->getName() == "VST3")
                {
                    vst3Path = fmt->getDefaultLocationsToSearch();
                    break;
                }
            }
        }
    }

    juce::KnownPluginList tempKnownList;
    struct CacheSnapshotParams {
        ShallowHost* host;
        juce::Array<juce::PluginDescription>* descriptions;
        juce::StringArray* blacklist;
    } cacheSnapshotParams { this, nullptr, nullptr };

    juce::Array<juce::PluginDescription> cachedDescriptions;
    juce::StringArray cachedBlacklist;
    cacheSnapshotParams.descriptions = &cachedDescriptions;
    cacheSnapshotParams.blacklist = &cachedBlacklist;
    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* params = static_cast<CacheSnapshotParams*>(p);
        for (const auto& description : params->host->knownPluginList.getTypes())
        {
            params->descriptions->add(description);
        }
        params->blacklist->addArray(params->host->knownPluginList.getBlacklistedFiles());
        return nullptr;
    }, &cacheSnapshotParams);

    // PluginDirectoryScanner can skip unchanged binaries only when its list is
    // pre-populated with the persisted descriptions from the previous scan.
    for (const auto& description : cachedDescriptions)
    {
        tempKnownList.addType(description);
    }
    for (const auto& blacklisted : cachedBlacklist)
    {
        tempKnownList.addToBlacklist(blacklisted);
    }

    const auto deadMansPedal = appDataDir == juce::File()
        ? juce::File()
        : appDataDir.getChildFile("plugin-scan-dead-mans-pedal.txt");

    for (int fmtIdx = 0; fmtIdx < formatManager.getNumFormats(); ++fmtIdx)
    {
        auto* fmt = formatManager.getFormat(fmtIdx);
        if (fmt == nullptr) continue;

        juce::FileSearchPath searchPath = fmt->getName() == "VST3"
            ? vst3Path
            : fmt->getDefaultLocationsToSearch();
        if (searchPath.getNumPaths() == 0) continue;

        juce::PluginDirectoryScanner scanner(
            tempKnownList,
            *fmt,
            searchPath,
            true,
            deadMansPedal
        );

        juce::String name;
        while (scanner.scanNextFile(true, name)) {}
    }

    struct Params {
        ShallowHost* host;
        const juce::KnownPluginList* tempKnownList;
        std::string result;
    } params { this, &tempKnownList, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        for (auto& desc : ps->tempKnownList->getTypes())
        {
            ps->host->knownPluginList.addType(desc);
        }
        for (const auto& blacklisted : ps->tempKnownList->getBlacklistedFiles())
        {
            ps->host->knownPluginList.addToBlacklist(blacklisted);
        }
        const auto knownTypes = ps->host->knownPluginList.getTypes();
        for (int i = knownTypes.size() - 1; i >= 0; --i)
        {
            const auto& desc = knownTypes.getReference(i);
            if (!juce::File(desc.fileOrIdentifier).exists())
            {
                ps->host->knownPluginList.removeType(desc);
            }
        }

        juce::Array<juce::var> arr;
        for (auto& desc : ps->host->knownPluginList.getTypes())
        {
            juce::DynamicObject::Ptr obj = new juce::DynamicObject();
            obj->setProperty("name", desc.name);
            obj->setProperty("vendor", desc.manufacturerName);
            obj->setProperty("version", desc.version);
            obj->setProperty("category", desc.category);
            obj->setProperty("path", desc.fileOrIdentifier);
            obj->setProperty("unique_id", desc.createIdentifierString());
            obj->setProperty("format", desc.pluginFormatName);
            obj->setProperty("has_editor", true);
            obj->setProperty("accepts_midi", desc.isInstrument);
            arr.add(juce::var(obj.get()));
        }

        ps->host->saveKnownPlugins();
        ps->result = juce::JSON::toString(juce::var(arr)).toStdString();
        return nullptr;
    }, &params);

    return params.result;
}

std::string ShallowHost::addToChain(const std::string& uniqueId)
{
    struct Params {
        ShallowHost* host;
        const std::string* uniqueId;
        std::string result;
    } params { this, &uniqueId, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->addToChainOnMessageThread(*ps->uniqueId);
        return nullptr;
    }, &params);

    return params.result;
}

std::string ShallowHost::addToChainOnMessageThread(const std::string& uniqueId, const std::string& base64State, bool bypassed)
{
    auto desc = knownPluginList.getTypeForIdentifierString(juce::String(uniqueId));
    if (desc == nullptr)
    {
        std::cerr << "[sh] plugin desc not found for identifier: " << uniqueId << std::endl;
        return "";
    }

    juce::String error;
    double sr = graph.getSampleRate() > 0.0 ? graph.getSampleRate() : 48000.0;
    int bs = graph.getBlockSize() > 0 ? graph.getBlockSize() : 512;
    auto instance = formatManager.createPluginInstance(*desc, sr, bs, error);
    if (instance == nullptr)
    {
        std::cerr << "[sh] failed to instantiate plugin: " << error.toStdString() << std::endl;
        return "";
    }

    instance->enableAllBuses();

    if (base64State.length() > 0)
    {
        juce::MemoryOutputStream os;
        if (juce::Base64::convertFromBase64(os, juce::String(base64State)))
        {
            auto block = os.getMemoryBlock();
            instance->setStateInformation(block.getData(), (int)block.getSize());
        }
    }

    auto node = graph.addNode(
        std::move(instance),
        std::nullopt,
        juce::AudioProcessorGraph::UpdateKind::none);
    if (node == nullptr)
    {
        return "";
    }

    node->setBypassed(bypassed);
    chainNodes.push_back(node);
    rebuildConnections();

    return std::to_string(node->nodeID.uid);
}

std::string ShallowHost::addToChainWithState(const std::string& uniqueId, const std::string& base64State, bool bypassed)
{
    struct Params {
        ShallowHost* host;
        const std::string* uniqueId;
        const std::string* base64State;
        bool bypassed;
        std::string result;
    } params { this, &uniqueId, &base64State, bypassed, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->addToChainOnMessageThread(*ps->uniqueId, *ps->base64State, ps->bypassed);
        return nullptr;
    }, &params);

    return params.result;
}

void ShallowHost::clearChain()
{
    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* host = static_cast<ShallowHost*>(p);
        host->activeWindows.clear();
        for (auto& node : host->chainNodes)
        {
            host->graph.removeNode(node, juce::AudioProcessorGraph::UpdateKind::none);
        }
        host->chainNodes.clear();
        host->rebuildConnections();
        return nullptr;
    }, this);
}

bool ShallowHost::removeFromChain(const std::string& nodeId)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        bool result;
    } params { this, &nodeId, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->removeFromChainOnMessageThread(*ps->nodeId);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::removeFromChainOnMessageThread(const std::string& nodeId)
{
    activeWindows.erase(nodeId);

    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it != chainNodes.end())
    {
        graph.removeNode(*it, juce::AudioProcessorGraph::UpdateKind::none);
        chainNodes.erase(it);
        rebuildConnections();
        return true;
    }

    return false;
}

bool ShallowHost::movePlugin(const std::string& nodeId, bool up)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        bool up;
        bool result;
    } params { this, &nodeId, up, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->movePluginOnMessageThread(*ps->nodeId, ps->up);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::movePluginOnMessageThread(const std::string& nodeId, bool up)
{
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it == chainNodes.end()) return false;

    size_t index = std::distance(chainNodes.begin(), it);

    if (up && index > 0)
    {
        std::swap(chainNodes[index], chainNodes[index - 1]);
        rebuildConnections();
        return true;
    }
    else if (!up && index < chainNodes.size() - 1)
    {
        std::swap(chainNodes[index], chainNodes[index + 1]);
        rebuildConnections();
        return true;
    }

    return false;
}

bool ShallowHost::reorderChain(const std::string& nodeId, int toIndex)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        int toIndex;
        bool result;
    } params { this, &nodeId, toIndex, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->reorderChainOnMessageThread(*ps->nodeId, ps->toIndex);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::reorderChainOnMessageThread(const std::string& nodeId, int toIndex)
{
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it == chainNodes.end()) return false;

    int fromIndex = (int)std::distance(chainNodes.begin(), it);
    if (fromIndex == toIndex) return true;
    if (toIndex < 0 || toIndex >= (int)chainNodes.size()) return false;

    auto elem = *it;
    chainNodes.erase(it);
    chainNodes.insert(chainNodes.begin() + toIndex, elem);

    rebuildConnections();
    return true;
}

bool ShallowHost::bypassPlugin(const std::string& nodeId, bool bypassed)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        bool bypassed;
        bool result;
    } params { this, &nodeId, bypassed, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->bypassPluginOnMessageThread(*ps->nodeId, ps->bypassed);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::bypassPluginOnMessageThread(const std::string& nodeId, bool bypassed)
{
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it != chainNodes.end())
    {
        (*it)->setBypassed(bypassed);
        return true;
    }

    return false;
}

std::string ShallowHost::getChainJson()
{
    struct Params {
        ShallowHost* host;
        std::string result;
    } params { this, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        juce::Array<juce::var> arr;
        for (auto& node : ps->host->chainNodes)
        {
            auto* proc = node->getProcessor();
            if (proc == nullptr) continue;

            auto* instance = dynamic_cast<juce::AudioPluginInstance*>(proc);
            if (instance == nullptr) continue;

            juce::DynamicObject::Ptr obj = new juce::DynamicObject();
            obj->setProperty("id", juce::String(std::to_string(node->nodeID.uid)));
            obj->setProperty("name", instance->getPluginDescription().name);
            obj->setProperty("vendor", instance->getPluginDescription().manufacturerName);
            obj->setProperty("format", instance->getPluginDescription().pluginFormatName);
            obj->setProperty("bypassed", node->isBypassed());
            obj->setProperty("unique_id", instance->getPluginDescription().createIdentifierString());
            arr.add(juce::var(obj.get()));
        }
        ps->result = juce::JSON::toString(juce::var(arr)).toStdString();
        return nullptr;
    }, &params);

    return params.result;
}

std::string ShallowHost::saveStateJson()
{
    struct Params {
        ShallowHost* host;
        std::string result;
    } params { this, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        juce::Array<juce::var> arr;
        for (auto& node : ps->host->chainNodes)
        {
            auto* proc = node->getProcessor();
            if (proc == nullptr) continue;

            auto* instance = dynamic_cast<juce::AudioPluginInstance*>(proc);
            if (instance == nullptr) continue;

            juce::DynamicObject::Ptr obj = new juce::DynamicObject();
            obj->setProperty("unique_id", instance->getPluginDescription().createIdentifierString());
            obj->setProperty("name", instance->getPluginDescription().name);
            obj->setProperty("vendor", instance->getPluginDescription().manufacturerName);
            obj->setProperty("format", instance->getPluginDescription().pluginFormatName);
            obj->setProperty("bypassed", node->isBypassed());

            juce::MemoryBlock block;
            proc->getStateInformation(block);
            juce::String base64 = juce::Base64::toBase64(block.getData(), block.getSize());
            obj->setProperty("state", juce::var(base64));

            arr.add(juce::var(obj.get()));
        }
        ps->result = juce::JSON::toString(juce::var(arr)).toStdString();
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::loadStateJson(const std::string& stateJson)
{
    struct Params {
        ShallowHost* host;
        const std::string* stateJson;
        bool result;
    } params { this, &stateJson, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->loadStateJsonOnMessageThread(*ps->stateJson);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::loadStateJsonOnMessageThread(const std::string& stateJson)
{
    juce::var json = juce::JSON::parse(juce::String(stateJson));
    if (!json.isArray()) return false;

    auto* arr = json.getArray();
    if (arr == nullptr) return false;

    struct RestoredPlugin {
        std::unique_ptr<juce::AudioPluginInstance> instance;
        bool bypassed;
    };
    std::vector<RestoredPlugin> restoredPlugins;
    restoredPlugins.reserve(static_cast<size_t>(arr->size()));

    for (int i = 0; i < arr->size(); ++i)
    {
        auto& item = arr->getReference(i);
        auto uniqueId = item.getProperty("unique_id", "").toString();
        auto bypassed = (bool)item.getProperty("bypassed", false);
        auto base64State = item.getProperty("state", "").toString();

        auto desc = knownPluginList.getTypeForIdentifierString(uniqueId);
        if (desc == nullptr)
        {
            std::cerr << "[sh] cannot restore missing plugin: "
                      << uniqueId.toStdString() << std::endl;
            return false;
        }

        juce::String error;
        double sr = graph.getSampleRate() > 0.0 ? graph.getSampleRate() : 48000.0;
        int bs = graph.getBlockSize() > 0 ? graph.getBlockSize() : 512;
        auto instance = formatManager.createPluginInstance(*desc, sr, bs, error);
        if (instance == nullptr)
        {
            std::cerr << "[sh] cannot restore plugin " << desc->name.toStdString()
                      << ": " << error.toStdString() << std::endl;
            return false;
        }

        instance->enableAllBuses();

        if (base64State.isNotEmpty())
        {
            juce::MemoryOutputStream os;
            if (!juce::Base64::convertFromBase64(os, base64State))
            {
                std::cerr << "[sh] invalid saved state for plugin: "
                          << desc->name.toStdString() << std::endl;
                return false;
            }
            auto block = os.getMemoryBlock();
            instance->setStateInformation(block.getData(), (int)block.getSize());
        }

        restoredPlugins.push_back({ std::move(instance), bypassed });
    }

    activeWindows.clear();
    for (auto& node : chainNodes)
    {
        graph.removeNode(node, juce::AudioProcessorGraph::UpdateKind::none);
    }
    chainNodes.clear();

    for (auto& plugin : restoredPlugins)
    {
        auto node = graph.addNode(
            std::move(plugin.instance),
            std::nullopt,
            juce::AudioProcessorGraph::UpdateKind::none);
        if (node == nullptr)
        {
            std::cerr << "[sh] failed to add restored plugin to graph" << std::endl;
            rebuildConnections();
            return false;
        }
        node->setBypassed(plugin.bypassed);
        chainNodes.push_back(node);
    }

    rebuildConnections();
    return true;
}
