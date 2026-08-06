#include "host.h"
#include <iostream>

int ShallowHost::audioStart(const char* driver, const char* inputDevice, const char* outputDevice,
                            int sampleRate, int bufferSize, int inputMask, int outputMask, bool mono)
{
    struct Params {
        ShallowHost* host;
        const char* driver;
        const char* inputDevice;
        const char* outputDevice;
        int sampleRate;
        int bufferSize;
        int inputMask;
        int outputMask;
        bool mono;
        int result;
    } params { this, driver, inputDevice, outputDevice, sampleRate, bufferSize, inputMask, outputMask, mono, 0 };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->audioStartOnMessageThread(
            ps->driver, ps->inputDevice, ps->outputDevice,
            ps->sampleRate, ps->bufferSize,
            ps->inputMask, ps->outputMask, ps->mono);
        return nullptr;
    }, &params);

    return params.result;
}

static juce::String getAudioTypeName(const char* driver)
{
    if (driver != nullptr && strlen(driver) > 0)
    {
        juce::String d(driver);
        if (d.equalsIgnoreCase("asio")) return "ASIO";
        if (d.equalsIgnoreCase("wasapi_exclusive") || d.equalsIgnoreCase("wasapi-exclusive") || d.equalsIgnoreCase("wasapi_ex"))
            return "Windows Audio (Exclusive Mode)";
        if (d.equalsIgnoreCase("wasapi_low_latency") || d.equalsIgnoreCase("wasapi-low-latency") || d.equalsIgnoreCase("wasapi_ll"))
            return "Windows Audio (Low Latency Mode)";
        return "Windows Audio";
    }
    return "Windows Audio";
}

int ShallowHost::audioStartOnMessageThread(const char* driver, const char* inputDevice, const char* outputDevice,
                                         int sampleRate, int bufferSize, int inputMask, int outputMask, bool mono)
{
    std::cout << "[sh] audioStartOnMessageThread: driver=" << (driver ? driver : "null")
              << ", inputDevice=" << (inputDevice ? inputDevice : "null")
              << ", outputDevice=" << (outputDevice ? outputDevice : "null")
              << ", inputMask=" << inputMask << ", outputMask=" << outputMask
              << ", mono=" << (mono ? "true" : "false") << std::endl;

    // Set the routing mode before opening the device so its first callback uses
    // the requested graph instead of briefly passing through stereo audio.
    isMono = mono;

    juce::String typeName = getAudioTypeName(driver);

    deviceManager.closeAudioDevice();
    deviceManager.setCurrentAudioDeviceType(typeName, true);

    juce::String inputName = (inputDevice != nullptr && juce::String(inputDevice) != "__default" && juce::String(inputDevice) != "__none") ? juce::String(inputDevice) : juce::String();
    juce::String outputName = (outputDevice != nullptr && juce::String(outputDevice) != "__default" && juce::String(outputDevice) != "__none") ? juce::String(outputDevice) : juce::String();

    juce::AudioIODeviceType* typeObject = nullptr;
    for (auto* type : deviceManager.getAvailableDeviceTypes())
    {
        if (type->getTypeName() == typeName)
        {
            typeObject = type;
            break;
        }
    }

    if (typeObject != nullptr)
    {
        std::string tName = typeName.toStdString();
        if (scannedDeviceTypes.find(tName) == scannedDeviceTypes.end())
        {
            typeObject->scanForDevices();
            scannedDeviceTypes.insert(tName);
        }

        if (inputDevice == nullptr || juce::String(inputDevice) == "__default" || juce::String(inputDevice).isEmpty())
        {
            int defaultIdx = typeObject->getDefaultDeviceIndex(true);
            auto names = typeObject->getDeviceNames(true);
            if (defaultIdx >= 0 && defaultIdx < names.size())
                inputName = names[defaultIdx];
        }

        if (outputDevice == nullptr || juce::String(outputDevice) == "__default" || juce::String(outputDevice).isEmpty())
        {
            int defaultIdx = typeObject->getDefaultDeviceIndex(false);
            auto names = typeObject->getDeviceNames(false);
            if (defaultIdx >= 0 && defaultIdx < names.size())
                outputName = names[defaultIdx];
        }
    }

    bool isNoneInput = (inputMask == 0 || (inputDevice != nullptr && juce::String(inputDevice) == "__none"));
    bool isNoneOutput = (outputMask == 0 || (outputDevice != nullptr && juce::String(outputDevice) == "__none"));

    if (isNoneInput && isNoneOutput)
    {
        deviceManager.closeAudioDevice();
        std::cout << "[sh] both input and output are none/disabled, closed device." << std::endl;
        return 1;
    }

    juce::AudioDeviceManager::AudioDeviceSetup setup;
    setup.inputDeviceName = isNoneInput ? juce::String() : inputName;
    setup.outputDeviceName = isNoneOutput ? juce::String() : outputName;
    setup.sampleRate = sampleRate > 0 ? sampleRate : 48000.0;
    setup.bufferSize = bufferSize > 0 ? bufferSize : 512;

    setup.inputChannels.clear();
    if (!isNoneInput)
    {
        if (inputMask >= 0)
        {
            for (int i = 0; i < 32; ++i)
            {
                if (((unsigned int)inputMask & (1u << i)) != 0)
                    setup.inputChannels.setBit(i);
            }
        }
        else
        {
            setup.inputChannels.setRange(0, 2, true);
        }
    }

    setup.outputChannels.clear();
    if (!isNoneOutput)
    {
        if (outputMask >= 0)
        {
            for (int i = 0; i < 32; ++i)
            {
                if (((unsigned int)outputMask & (1u << i)) != 0)
                    setup.outputChannels.setBit(i);
            }
        }
        else
        {
            setup.outputChannels.setRange(0, 2, true);
        }
    }

    setup.useDefaultInputChannels = (inputMask < 0) && setup.inputDeviceName.isEmpty() && !isNoneInput;
    setup.useDefaultOutputChannels = (outputMask < 0) && setup.outputDeviceName.isEmpty() && !isNoneOutput;

    std::cout << "[sh] setup: inputDeviceName=\"" << setup.inputDeviceName.toStdString()
              << "\", useDefaultInputChannels=" << (setup.useDefaultInputChannels ? "true" : "false")
              << ", inputChannelsCount=" << setup.inputChannels.countNumberOfSetBits()
              << ", outputDeviceName=\"" << setup.outputDeviceName.toStdString()
              << ", useDefaultOutputChannels=" << (setup.useDefaultOutputChannels ? "true" : "false")
              << ", outputChannelsCount=" << setup.outputChannels.countNumberOfSetBits() << std::endl;

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
    scannedDeviceTypes.clear();
    return 1;
}

void ShallowHost::changeListenerCallback(juce::ChangeBroadcaster* source)
{
    if (source == &deviceManager)
    {
        scannedDeviceTypes.clear();
        rebuildConnectionsOnMessageThread();
    }
}

std::string ShallowHost::getAudioDevicesJson(const char* driver, const char* deviceName)
{
    struct Params {
        ShallowHost* host;
        const char* driver;
        const char* deviceName;
        std::string result;
    } params { this, driver, deviceName, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        juce::DynamicObject::Ptr obj = new juce::DynamicObject();

        juce::Array<juce::var> inputsArray;
        juce::Array<juce::var> outputsArray;
        juce::Array<juce::var> inputChannelNamesArray;
        juce::Array<juce::var> outputChannelNamesArray;

        juce::String targetType = getAudioTypeName(ps->driver);

        juce::AudioIODeviceType* typeObject = nullptr;
        for (auto* type : ps->host->deviceManager.getAvailableDeviceTypes())
        {
            if (type->getTypeName() == targetType)
            {
                typeObject = type;
                break;
            }
        }

        if (typeObject != nullptr)
        {
            std::string tName = targetType.toStdString();
            if (ps->host->scannedDeviceTypes.find(tName) == ps->host->scannedDeviceTypes.end())
            {
                typeObject->scanForDevices();
                ps->host->scannedDeviceTypes.insert(tName);
            }

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
            if (ps->deviceName != nullptr && juce::String(ps->deviceName).isNotEmpty() && juce::String(ps->deviceName) != "__none" && juce::String(ps->deviceName) != "__default")
            {
                activeDeviceName = ps->deviceName;
            }
            else if (auto* currentDevice = ps->host->deviceManager.getCurrentAudioDevice())
            {
                if (ps->host->deviceManager.getCurrentAudioDeviceType() == targetType)
                    activeDeviceName = currentDevice->getName();
            }

            if (activeDeviceName.isNotEmpty())
            {
                bool gotChannels = false;
                if (auto* currentDevice = ps->host->deviceManager.getCurrentAudioDevice())
                {
                    if (currentDevice->getName() == activeDeviceName && ps->host->deviceManager.getCurrentAudioDeviceType() == targetType)
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

        ps->result = juce::JSON::toString(juce::var(obj.get())).toStdString();
        return nullptr;
    }, &params);

    return params.result;
}
