#include "host.h"
#include <algorithm>
#include <future>
#include <iostream>
#include <mutex>
#include <thread>
#include <vector>

#if defined(_WIN32)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <objbase.h>
#endif

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

    struct ScanTask {
        juce::AudioPluginFormat* format;
        juce::File file;
    };
    std::vector<ScanTask> tasks;

    for (int fmtIdx = 0; fmtIdx < formatManager.getNumFormats(); ++fmtIdx)
    {
        auto* fmt = formatManager.getFormat(fmtIdx);
        if (fmt == nullptr) continue;

        juce::FileSearchPath searchPath = (fmt->getName() == "VST3") ? vst3Path : vst2Path;
        if (searchPath.getNumPaths() == 0) continue;

        juce::Array<juce::File> foundFiles;
        searchPath.findChildFiles(foundFiles, juce::File::findFilesAndDirectories, true);

        for (const auto& file : foundFiles)
        {
            if (fmt->fileMightContainThisPluginType(file.getFullPathName()))
            {
                tasks.push_back({ fmt, file });
            }
        }
    }

    juce::KnownPluginList tempKnownList;
    std::mutex listMutex;

    unsigned int numThreads = (std::max)(1u, std::thread::hardware_concurrency());
    size_t totalTasks = tasks.size();
    size_t chunkSize = (totalTasks + numThreads - 1) / numThreads;

    std::vector<std::future<void>> futures;

    for (unsigned int t = 0; t < numThreads; ++t)
    {
        size_t start = t * chunkSize;
        size_t end = (std::min)(start + chunkSize, totalTasks);
        if (start >= totalTasks) break;

        futures.push_back(std::async(std::launch::async, [this, &tasks, start, end, &tempKnownList, &listMutex]() {
#if defined(_WIN32)
            CoInitializeEx(nullptr, COINIT_MULTITHREADED);
#endif
            for (size_t i = start; i < end; ++i)
            {
                const auto& task = tasks[i];
                juce::OwnedArray<juce::PluginDescription> results;
                task.format->findAllTypesForFile(results, task.file.getFullPathName());

                if (!results.isEmpty())
                {
                    std::lock_guard<std::mutex> lock(listMutex);
                    for (auto* desc : results)
                    {
                        if (desc != nullptr)
                        {
                            tempKnownList.addType(*desc);
                        }
                    }
                }
            }
#if defined(_WIN32)
            CoUninitialize();
#endif
        }));
    }

    for (auto& f : futures)
    {
        f.get();
    }

    struct Params {
        ShallowHost* host;
        const juce::KnownPluginList* tempKnownList;
        std::string result;
    } params { this, &tempKnownList, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        if (ps->tempKnownList->getNumTypes() > 0)
        {
            for (auto& desc : ps->tempKnownList->getTypes())
            {
                ps->host->knownPluginList.addType(desc);
            }
        }
        for (int i = ps->host->knownPluginList.getNumTypes() - 1; i >= 0; --i)
        {
            auto* desc = ps->host->knownPluginList.getType(i);
            if (desc != nullptr && !juce::File(desc->fileOrIdentifier).exists())
            {
                ps->host->knownPluginList.removeType(*desc);
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
#if defined(_WIN32)
    CoInitializeEx(nullptr, COINIT_MULTITHREADED);
#endif
    pumpMessageLoop();
    juce::String idStr(uniqueId);
    std::unique_ptr<juce::PluginDescription> customDesc;
    const juce::PluginDescription* desc = nullptr;

    for (int i = 0; i < knownPluginList.getNumTypes(); ++i)
    {
        if (auto* d = knownPluginList.getType(i))
        {
            if (d->createIdentifierString() == idStr)
            {
                desc = d;
                break;
            }
        }
    }

    if (desc == nullptr)
    {
        for (int i = 0; i < knownPluginList.getNumTypes(); ++i)
        {
            if (auto* d = knownPluginList.getType(i))
            {
                if (!d->name.isEmpty() && idStr.containsIgnoreCase(d->name))
                {
                    desc = d;
                    break;
                }
            }
        }
    }

    if (desc == nullptr)
    {
        customDesc = knownPluginList.getTypeForIdentifierString(idStr);
        desc = customDesc.get();
    }

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

    auto node = graph.addNode(std::move(instance));
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
    activeWindows.erase(nodeId);

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

    activeWindows.clear();

    for (auto& node : chainNodes)
    {
        graph.removeNode(node);
    }
    chainNodes.clear();

    for (int i = 0; i < arr->size(); ++i)
    {
        auto& item = arr->getReference(i);
        auto uniqueId = item.getProperty("unique_id", "").toString();
        auto bypassed = (bool)item.getProperty("bypassed", false);
        auto base64State = item.getProperty("state", "").toString();

        auto desc = knownPluginList.getTypeForIdentifierString(uniqueId);
        if (desc == nullptr) continue;

        juce::String error;
        double sr = graph.getSampleRate() > 0.0 ? graph.getSampleRate() : 48000.0;
        int bs = graph.getBlockSize() > 0 ? graph.getBlockSize() : 512;
        auto instance = formatManager.createPluginInstance(*desc, sr, bs, error);
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
    }

    rebuildConnections();
    return true;
}
