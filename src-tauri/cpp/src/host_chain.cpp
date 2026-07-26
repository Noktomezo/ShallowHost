#include "host.h"
#include <algorithm>
#include <iostream>

std::string ShallowHost::scanPluginsJson(const std::string& vst2PathsJson, const std::string& vst3PathsJson)
{
    juce::FileSearchPath vst2Path;
    juce::var vst2Arr = juce::JSON::parse(juce::String(vst2PathsJson));
    if (vst2Arr.isArray())
    {
        for (int i = 0; i < vst2Arr.getArray()->size(); ++i)
        {
            vst2Path.add(juce::File(vst2Arr.getArray()->getReference(i).toString()));
        }
    }

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
        vst3Path = formatManager.getFormat(1)->getDefaultLocationsToSearch();
    }

    for (int fmtIdx = 0; fmtIdx < formatManager.getNumFormats(); ++fmtIdx)
    {
        auto* fmt = formatManager.getFormat(fmtIdx);
        if (fmt == nullptr) continue;

        juce::FileSearchPath searchPath = (fmt->getName() == "VST3") ? vst3Path : vst2Path;
        if (searchPath.getNumPaths() == 0) continue;

        juce::PluginDirectoryScanner scanner(
            knownPluginList,
            *fmt,
            searchPath,
            true,
            juce::File()
        );

        juce::String name;
        while (scanner.scanNextFile(true, name)) {}
    }

    for (int i = knownPluginList.getNumTypes() - 1; i >= 0; --i)
    {
        auto* desc = knownPluginList.getType(i);
        if (desc != nullptr && !juce::File(desc->fileOrIdentifier).exists())
        {
            knownPluginList.removeType(*desc);
        }
    }

    juce::Array<juce::var> arr;
    for (auto& desc : knownPluginList.getTypes())
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

    saveKnownPlugins();
    return juce::JSON::toString(juce::var(arr)).toStdString();
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

std::string ShallowHost::addToChainOnMessageThread(const std::string& uniqueId)
{
    pumpMessageLoop();
    auto desc = knownPluginList.getTypeForIdentifierString(juce::String(uniqueId));
    if (desc == nullptr)
    {
        std::cerr << "[sh] plugin desc not found for identifier: " << uniqueId << std::endl;
        return "";
    }

    juce::String error;
    auto instance = formatManager.createPluginInstance(*desc, graph.getSampleRate(), graph.getBlockSize(), error);
    if (instance == nullptr)
    {
        std::cerr << "[sh] failed to instantiate plugin: " << error.toStdString() << std::endl;
        return "";
    }

    instance->enableAllBuses();

    auto node = graph.addNode(std::move(instance));
    if (node == nullptr)
    {
        return "";
    }

    chainNodes.push_back(node);
    rebuildConnections();
    pumpMessageLoop();

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
        ps->host->pumpMessageLoop();
        auto desc = ps->host->knownPluginList.getTypeForIdentifierString(juce::String(*ps->uniqueId));
        if (desc == nullptr) return nullptr;

        juce::String error;
        auto instance = ps->host->formatManager.createPluginInstance(*desc, ps->host->graph.getSampleRate(), ps->host->graph.getBlockSize(), error);
        if (instance == nullptr) return nullptr;

        instance->enableAllBuses();

        if (ps->base64State->length() > 0)
        {
            juce::MemoryOutputStream os;
            if (juce::Base64::convertFromBase64(os, juce::String(*ps->base64State)))
            {
                auto block = os.getMemoryBlock();
                instance->setStateInformation(block.getData(), (int)block.getSize());
            }
        }

        auto node = ps->host->graph.addNode(std::move(instance));
        if (node != nullptr)
        {
            node->setBypassed(ps->bypassed);
            ps->host->chainNodes.push_back(node);
            ps->host->rebuildConnections();
            ps->result = std::to_string(node->nodeID.uid);
        }
        ps->host->pumpMessageLoop();
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
            host->graph.removeNode(node);
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
    closePluginGui(nodeId);

    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it != chainNodes.end())
    {
        graph.removeNode(*it);
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
    juce::Array<juce::var> arr;
    for (auto& node : chainNodes)
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
    return juce::JSON::toString(juce::var(arr)).toStdString();
}

std::string ShallowHost::saveStateJson()
{
    juce::Array<juce::var> arr;
    for (auto& node : chainNodes)
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
    return juce::JSON::toString(juce::var(arr)).toStdString();
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

    activeWindows.clear();

    for (auto& node : chainNodes)
    {
        graph.removeNode(node);
    }
    chainNodes.clear();

    for (int i = 0; i < arr->size(); ++i)
    {
        pumpMessageLoop();
        auto& item = arr->getReference(i);
        auto uniqueId = item.getProperty("unique_id", "").toString();
        auto bypassed = (bool)item.getProperty("bypassed", false);
        auto base64State = item.getProperty("state", "").toString();

        auto desc = knownPluginList.getTypeForIdentifierString(uniqueId);
        if (desc == nullptr) continue;

        juce::String error;
        auto instance = formatManager.createPluginInstance(*desc, graph.getSampleRate(), graph.getBlockSize(), error);
        if (instance == nullptr) continue;

        instance->enableAllBuses();

        if (base64State.isNotEmpty())
        {
            juce::MemoryOutputStream os;
            if (juce::Base64::convertFromBase64(os, base64State))
            {
                auto block = os.getMemoryBlock();
                instance->setStateInformation(block.getData(), (int)block.getSize());
            }
        }

        auto node = graph.addNode(std::move(instance));
        if (node != nullptr)
        {
            node->setBypassed(bypassed);
            chainNodes.push_back(node);
        }
        pumpMessageLoop();
    }

    rebuildConnections();
    return true;
}
