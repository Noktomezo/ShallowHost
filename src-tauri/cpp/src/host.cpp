#include "host.h"
#include <algorithm>
#include <iostream>

ShallowHost::ShallowHost()
{
    juce::addDefaultFormatsToManager(formatManager);
    setupGraph();
    player.setProcessor(&graph);
    deviceManager.addChangeListener(this);
    deviceManager.initialise(256, 256, nullptr, true);
    deviceManager.addAudioCallback(&player);
}

ShallowHost::~ShallowHost()
{
    if (juce::MessageManager::getInstanceWithoutCreating() != nullptr)
    {
        deviceManager.removeChangeListener(this);
        audioStop();
        activeWindows.clear();
    }
}

void ShallowHost::setupGraph()
{
    graph.clear();

    inputNode = graph.addNode(
        std::make_unique<juce::AudioProcessorGraph::AudioGraphIOProcessor>(
            juce::AudioProcessorGraph::AudioGraphIOProcessor::audioInputNode),
        juce::AudioProcessorGraph::NodeID{ 1000000 });

    outputNode = graph.addNode(
        std::make_unique<juce::AudioProcessorGraph::AudioGraphIOProcessor>(
            juce::AudioProcessorGraph::AudioGraphIOProcessor::audioOutputNode),
        juce::AudioProcessorGraph::NodeID{ 1000001 });

    rebuildConnections();
}

void ShallowHost::rebuildConnections()
{
    struct Params {
        ShallowHost* host;
    } params { this };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->host->rebuildConnectionsOnMessageThread();
        return nullptr;
    }, &params);
}

void ShallowHost::rebuildConnectionsOnMessageThread()
{
    if (inputNode == nullptr || outputNode == nullptr) return;
    auto conns = graph.getConnections();
    for (int i = (int)conns.size(); --i >= 0;)
    {
        graph.removeConnection(conns[i]);
    }

    if (chainNodes.empty())
    {
        if (monoMode)
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, 0 }, { outputNode->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { outputNode->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, 1 }, { outputNode->nodeID, 1 } });
        }
    }
    else
    {
        if (monoMode)
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { chainNodes[0]->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, 0 }, { chainNodes[0]->nodeID, 1 } });
        }
        else
        {
            graph.addConnection({ { inputNode->nodeID, 0 }, { chainNodes[0]->nodeID, 0 } });
            graph.addConnection({ { inputNode->nodeID, 1 }, { chainNodes[0]->nodeID, 1 } });
        }

        for (size_t i = 0; i < chainNodes.size() - 1; ++i)
        {
            graph.addConnection({ { chainNodes[i]->nodeID, 0 }, { chainNodes[i + 1]->nodeID, 0 } });
            graph.addConnection({ { chainNodes[i]->nodeID, 1 }, { chainNodes[i + 1]->nodeID, 1 } });
        }

        graph.addConnection({ { chainNodes.back()->nodeID, 0 }, { outputNode->nodeID, 0 } });
        graph.addConnection({ { chainNodes.back()->nodeID, 1 }, { outputNode->nodeID, 1 } });
    }
}

void ShallowHost::setAppDataDirectory(const std::string& path)
{
    appDataDir = juce::File(juce::String(path));
    loadKnownPlugins();
}

void ShallowHost::loadKnownPlugins()
{
    if (appDataDir == juce::File()) return;
    auto file = appDataDir.getChildFile("plugins.xml");
    if (file.existsAsFile())
    {
        if (auto xml = juce::XmlDocument::parse(file))
        {
            knownPluginList.recreateFromXml(*xml);
        }
    }
}

void ShallowHost::saveKnownPlugins()
{
    if (appDataDir == juce::File()) return;
    auto file = appDataDir.getChildFile("plugins.xml");
    if (auto xml = knownPluginList.createXml())
    {
        xml->writeTo(file);
    }
}

void ShallowHost::initialize()
{
    juce::MessageManager::getInstance();
}

void ShallowHost::shutdown()
{
    auto& host = getInstance();
    host.deviceManager.removeChangeListener(&host);
    host.audioStop();
    host.activeWindows.clear();
    juce::MessageManager::deleteInstance();
}

ShallowHost& ShallowHost::getInstance()
{
    static ShallowHost instance;
    return instance;
}

int ShallowHost::audioStart(const char* driver, const char* inputDevice, const char* outputDevice,
                            int sampleRate, int bufferSize, int mono, int inputMask, int outputMask)
{
    struct Params {
        ShallowHost* host;
        const char* driver;
        const char* inputDevice;
        const char* outputDevice;
        int sampleRate;
        int bufferSize;
        int mono;
        int inputMask;
        int outputMask;
        int result;
    } params { this, driver, inputDevice, outputDevice, sampleRate, bufferSize, mono, inputMask, outputMask, 0 };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->audioStartOnMessageThread(
            ps->driver, ps->inputDevice, ps->outputDevice,
            ps->sampleRate, ps->bufferSize, ps->mono,
            ps->inputMask, ps->outputMask);
        return nullptr;
    }, &params);

    return params.result;
}

int ShallowHost::audioStartOnMessageThread(const char* driver, const char* inputDevice, const char* outputDevice,
                                         int sampleRate, int bufferSize, int mono, int inputMask, int outputMask)
{
    if (driver != nullptr && strlen(driver) > 0)
    {
        juce::String typeName = juce::String(driver).equalsIgnoreCase("asio") ? "ASIO" : "Windows Audio";
        if (deviceManager.getCurrentAudioDeviceType() != typeName)
        {
            deviceManager.setCurrentAudioDeviceType(typeName, true);
        }
    }
    else
    {
        if (deviceManager.getCurrentDeviceTypeObject() == nullptr)
        {
            deviceManager.setCurrentAudioDeviceType("Windows Audio", true);
        }
    }

    juce::String inputName = (inputDevice != nullptr && juce::String(inputDevice) != "__default" && juce::String(inputDevice) != "__none") ? juce::String(inputDevice) : juce::String();
    juce::String outputName = (outputDevice != nullptr && juce::String(outputDevice) != "__default" && juce::String(outputDevice) != "__none") ? juce::String(outputDevice) : juce::String();

    juce::AudioDeviceManager::AudioDeviceSetup setup;
    setup.inputDeviceName = (inputMask == 0 || juce::String(inputDevice) == "__none") ? juce::String() : inputName;
    setup.outputDeviceName = (outputMask == 0 || juce::String(outputDevice) == "__none") ? juce::String() : outputName;

    if (setup.inputDeviceName.isEmpty() && setup.outputDeviceName.isEmpty())
    {
        deviceManager.closeAudioDevice();
        return 1;
    }

    setup.sampleRate = sampleRate > 0 ? sampleRate : 48000.0;
    setup.bufferSize = bufferSize > 0 ? bufferSize : 512;

    setup.inputChannels.clear();
    if (inputMask >= 0)
    {
        for (int i = 0; i < 32; ++i)
        {
            if ((inputMask & (1 << i)) != 0)
                setup.inputChannels.setBit(i);
        }
    }
    else
    {
        if (mono)
        {
            setup.inputChannels.setBit(0);
        }
        else
        {
            setup.inputChannels.setRange(0, 2, true);
        }
    }

    setup.outputChannels.clear();
    if (outputMask >= 0)
    {
        for (int i = 0; i < 32; ++i)
        {
            if ((outputMask & (1 << i)) != 0)
                setup.outputChannels.setBit(i);
        }
    }
    else
    {
        setup.outputChannels.setRange(0, 2, true);
    }

    setup.useDefaultInputChannels = (inputMask < 0) && setup.inputDeviceName.isEmpty();
    setup.useDefaultOutputChannels = (outputMask < 0) && setup.outputDeviceName.isEmpty();

    monoMode = (mono != 0);

    auto err = deviceManager.setAudioDeviceSetup(setup, true);

    if (err.isNotEmpty())
    {
        std::cerr << "[sh] audio start failed: " << err.toStdString() << std::endl;
        return 0;
    }

    rebuildConnectionsOnMessageThread();
    return 1;
}

int ShallowHost::audioStop()
{
    struct Params {
        ShallowHost* host;
        int result;
    } params { this, 0 };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->audioStopOnMessageThread();
        return nullptr;
    }, &params);

    return params.result;
}

int ShallowHost::audioStopOnMessageThread()
{
    deviceManager.closeAudioDevice();
    return 1;
}

std::string ShallowHost::getAudioDevicesJson(const char* driver, const char* deviceName)
{
    juce::DynamicObject::Ptr obj = new juce::DynamicObject();

    juce::Array<juce::var> inputsArray;
    juce::Array<juce::var> outputsArray;
    juce::Array<juce::var> inputChannelNamesArray;
    juce::Array<juce::var> outputChannelNamesArray;

    juce::String targetType = (driver != nullptr && juce::String(driver).equalsIgnoreCase("asio")) ? "ASIO" : "Windows Audio";

    juce::AudioIODeviceType* typeObject = nullptr;
    for (auto* type : deviceManager.getAvailableDeviceTypes())
    {
        if (type->getTypeName() == targetType)
        {
            typeObject = type;
            break;
        }
    }

    if (typeObject != nullptr)
    {
        typeObject->scanForDevices();

        juce::String defaultInputName;
        juce::String defaultOutputName;

        int defInIdx = typeObject->getDefaultDeviceIndex(true);
        auto inDevNames = typeObject->getDeviceNames(true);
        if (defInIdx >= 0 && defInIdx < inDevNames.size()) {
            defaultInputName = inDevNames[defInIdx];
        }

        int defOutIdx = typeObject->getDefaultDeviceIndex(false);
        auto outDevNames = typeObject->getDeviceNames(false);
        if (defOutIdx >= 0 && defOutIdx < outDevNames.size()) {
            defaultOutputName = outDevNames[defOutIdx];
        }

        for (int i = 0; i < inDevNames.size(); ++i)
        {
            juce::DynamicObject::Ptr devObj = new juce::DynamicObject();
            devObj->setProperty("name", inDevNames[i]);
            devObj->setProperty("default", inDevNames[i] == defaultInputName);
            inputsArray.add(juce::var(devObj.get()));
        }

        for (int i = 0; i < outDevNames.size(); ++i)
        {
            juce::DynamicObject::Ptr devObj = new juce::DynamicObject();
            devObj->setProperty("name", outDevNames[i]);
            devObj->setProperty("default", outDevNames[i] == defaultOutputName);
            outputsArray.add(juce::var(devObj.get()));
        }

        juce::String activeDeviceName;
        if (deviceName != nullptr && juce::String(deviceName).isNotEmpty() && juce::String(deviceName) != "__none" && juce::String(deviceName) != "__default")
        {
            activeDeviceName = deviceName;
        }
        else if (auto* currentDevice = deviceManager.getCurrentAudioDevice())
        {
            if (deviceManager.getCurrentAudioDeviceType() == targetType)
                activeDeviceName = currentDevice->getName();
        }

        if (activeDeviceName.isNotEmpty())
        {
            bool gotChannels = false;
            if (auto* currentDevice = deviceManager.getCurrentAudioDevice())
            {
                if (currentDevice->getName() == activeDeviceName && deviceManager.getCurrentAudioDeviceType() == targetType)
                {
                    auto inNames = currentDevice->getInputChannelNames();
                    for (int i = 0; i < inNames.size(); ++i)
                        inputChannelNamesArray.add(inNames[i]);

                    auto outNames = currentDevice->getOutputChannelNames();
                    for (int i = 0; i < outNames.size(); ++i)
                        outputChannelNamesArray.add(outNames[i]);

                    gotChannels = true;
                }
            }

            if (!gotChannels)
            {
                std::unique_ptr<juce::AudioIODevice> tempDevice (typeObject->createDevice (activeDeviceName, activeDeviceName));
                if (tempDevice != nullptr)
                {
                    auto inNames = tempDevice->getInputChannelNames();
                    for (int i = 0; i < inNames.size(); ++i)
                        inputChannelNamesArray.add(inNames[i]);

                    auto outNames = tempDevice->getOutputChannelNames();
                    for (int i = 0; i < outNames.size(); ++i)
                        outputChannelNamesArray.add(outNames[i]);
                }
            }
        }
    }

    obj->setProperty("inputs", inputsArray);
    obj->setProperty("outputs", outputsArray);
    obj->setProperty("input_channels", inputChannelNamesArray);
    obj->setProperty("output_channels", outputChannelNamesArray);

    return juce::JSON::toString(juce::var(obj.get())).toStdString();
}

std::string ShallowHost::scanPluginsJson()
{
    knownPluginList.clear();
    juce::VST3PluginFormat vst3Format;
    auto paths = vst3Format.getDefaultLocationsToSearch();

    juce::PluginDirectoryScanner scanner(knownPluginList, vst3Format, paths, true, juce::File(), true);
    juce::String pluginName;
    while (scanner.scanNextFile(true, pluginName)) {}

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

    return std::to_string(node->nodeID.uid);
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
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));

    closePluginGuiOnMessageThread(nodeId);

    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return node->nodeID == id;
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
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return node->nodeID == id;
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
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return node->nodeID == id;
    });

    if (it == chainNodes.end()) return false;
    if (toIndex < 0 || toIndex >= (int)chainNodes.size()) return false;

    auto node = *it;
    chainNodes.erase(it);
    chainNodes.insert(chainNodes.begin() + toIndex, node);
    rebuildConnections();
    return true;
}

bool ShallowHost::bypassPlugin(const std::string& nodeId, bool bypassed)
{
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    if (auto* node = graph.getNodeForId(id))
    {
        node->setBypassed(bypassed);
        return true;
    }
    return false;
}

std::string ShallowHost::getChainJson()
{
    juce::Array<juce::var> arr;
    for (auto& node : chainNodes)
    {
        juce::DynamicObject::Ptr obj = new juce::DynamicObject();
        obj->setProperty("id", juce::String(std::to_string(node->nodeID.uid)));
        if (auto* proc = node->getProcessor())
        {
            obj->setProperty("name", proc->getName());
            if (auto* instance = dynamic_cast<juce::AudioPluginInstance*>(proc))
            {
                obj->setProperty("format", instance->getPluginDescription().pluginFormatName);
                obj->setProperty("vendor", instance->getPluginDescription().manufacturerName);
            }
            else
            {
                obj->setProperty("format", "");
                obj->setProperty("vendor", "");
            }
        }
        obj->setProperty("bypassed", node->isBypassed());
        arr.add(juce::var(obj.get()));
    }
    return juce::JSON::toString(juce::var(arr)).toStdString();
}

std::string ShallowHost::getPluginParametersJson(const std::string& nodeId)
{
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    auto* node = graph.getNodeForId(id);
    if (node == nullptr) return "[]";

    auto* proc = node->getProcessor();
    if (proc == nullptr) return "[]";

    juce::Array<juce::var> arr;
    auto& params = proc->getParameters();
    for (int i = 0; i < params.size(); ++i)
    {
        auto* param = params[i];
        juce::DynamicObject::Ptr obj = new juce::DynamicObject();
        obj->setProperty("index", i);
        obj->setProperty("name", param->getName(128));
        obj->setProperty("unit", param->getLabel());
        obj->setProperty("min", 0.0);
        obj->setProperty("max", 1.0);
        obj->setProperty("default", (double)param->getDefaultValue());
        obj->setProperty("step_count", (int)param->getNumSteps());
        obj->setProperty("value", (double)param->getValue());
        arr.add(juce::var(obj.get()));
    }

    return juce::JSON::toString(juce::var(arr)).toStdString();
}

bool ShallowHost::setPluginParameter(const std::string& nodeId, int paramIndex, float value)
{
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    auto* node = graph.getNodeForId(id);
    if (node == nullptr) return false;

    auto* proc = node->getProcessor();
    if (proc == nullptr) return false;

    auto& params = proc->getParameters();
    if (paramIndex < 0 || paramIndex >= params.size()) return false;

    params[paramIndex]->setValue(value);
    return true;
}

bool ShallowHost::openPluginGuiOnMessageThread(const std::string& nodeId)
{
    auto id = juce::AudioProcessorGraph::NodeID(std::stoul(nodeId));
    auto* node = graph.getNodeForId(id);
    if (node == nullptr) return false;

    auto* proc = node->getProcessor();
    if (proc == nullptr || !proc->hasEditor()) return false;

    auto it = activeWindows.find(nodeId);
    if (it != activeWindows.end())
    {
        it->second->toFront(true);
        return true;
    }

    auto editor = std::unique_ptr<juce::AudioProcessorEditor>(proc->createEditorIfNeeded());
    if (editor == nullptr) return false;

    auto window = std::make_unique<PluginWindow>(nodeId, proc->getName(), std::move(editor));
    window->setSize(window->getContentComponent()->getWidth(), window->getContentComponent()->getHeight());
    window->setResizable(false, false);

    activeWindows[nodeId] = std::move(window);
    return true;
}

bool ShallowHost::closePluginGuiOnMessageThread(const std::string& nodeId)
{
    auto it = activeWindows.find(nodeId);
    if (it != activeWindows.end())
    {
        activeWindows.erase(it);
        return true;
    }
    return false;
}

bool ShallowHost::openPluginGui(const std::string& nodeId)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        bool success;
    } params { this, &nodeId, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->success = ps->host->openPluginGuiOnMessageThread(*ps->nodeId);
        return nullptr;
    }, &params);

    return params.success;
}

bool ShallowHost::closePluginGui(const std::string& nodeId)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        bool success;
    } params { this, &nodeId, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->success = ps->host->closePluginGuiOnMessageThread(*ps->nodeId);
        return nullptr;
    }, &params);

    return params.success;
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
        obj->setProperty("bypassed", node->isBypassed());

        juce::MemoryBlock block;
        proc->getStateInformation(block);
        juce::String base64 = juce::Base64::toBase64(block.getData(), block.getSize());
        obj->setProperty("state", base64);

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
    }

    rebuildConnections();
    return true;
}

void ShallowHost::PluginWindow::closeButtonPressed()
{
    juce::MessageManager::callAsync([id = nodeId]() {
        ShallowHost::getInstance().closePluginGui(id);
    });
}

void ShallowHost::changeListenerCallback(juce::ChangeBroadcaster* source)
{
    if (source == &deviceManager)
    {
        rebuildConnections();
    }
}
