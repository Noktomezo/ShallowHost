#include "host.h"
#include <algorithm>

#if defined(_WIN32) || defined(_WIN64)
#ifndef NOMINMAX
#define NOMINMAX
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <shellapi.h>
#endif

std::string ShallowHost::getPluginParametersJson(const std::string& nodeId)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        std::string result;
    } params { this, &nodeId, "" };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->getPluginParametersJsonOnMessageThread(*ps->nodeId);
        return nullptr;
    }, &params);

    return params.result;
}

std::string ShallowHost::getPluginParametersJsonOnMessageThread(const std::string& nodeId)
{
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it == chainNodes.end()) return "[]";

    auto* proc = (*it)->getProcessor();
    if (proc == nullptr) return "[]";

    juce::Array<juce::var> arr;
    auto& pList = proc->getParameters();

    for (int i = 0; i < pList.size(); ++i)
    {
        auto* param = pList[i];
        if (param == nullptr) continue;

        juce::DynamicObject::Ptr obj = new juce::DynamicObject();
        obj->setProperty("index", i);
        obj->setProperty("name", param->getName(100));
        obj->setProperty("value", param->getValue());

        juce::String textVal = param->getText(param->getValue(), 100);
        obj->setProperty("text_value", textVal);

        arr.add(juce::var(obj.get()));
    }

    return juce::JSON::toString(juce::var(arr)).toStdString();
}

bool ShallowHost::setPluginParameter(const std::string& nodeId, int paramIndex, float value)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        int paramIndex;
        float value;
        bool result;
    } params { this, &nodeId, paramIndex, value, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->result = ps->host->setPluginParameterOnMessageThread(*ps->nodeId, ps->paramIndex, ps->value);
        return nullptr;
    }, &params);

    return params.result;
}

bool ShallowHost::setPluginParameterOnMessageThread(const std::string& nodeId, int paramIndex, float value)
{
    auto it = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (it == chainNodes.end()) return false;

    auto* proc = (*it)->getProcessor();
    if (proc == nullptr) return false;

    auto& pList = proc->getParameters();
    if (paramIndex >= 0 && paramIndex < pList.size())
    {
        if (auto* param = pList[paramIndex])
        {
            param->setValueNotifyingHost(value);
            return true;
        }
    }

    return false;
}

bool ShallowHost::openPluginGui(const std::string& nodeId, const std::string& titlePrefix)
{
    struct Params {
        ShallowHost* host;
        const std::string* nodeId;
        const std::string* titlePrefix;
        bool success;
    } params { this, &nodeId, &titlePrefix, false };

    juce::MessageManager::getInstance()->callFunctionOnMessageThread([](void* p) -> void* {
        auto* ps = static_cast<Params*>(p);
        ps->success = ps->host->openPluginGuiOnMessageThread(*ps->nodeId, *ps->titlePrefix);
        return nullptr;
    }, &params);

    return params.success;
}

bool ShallowHost::openPluginGuiOnMessageThread(const std::string& nodeId, const std::string& titlePrefix)
{
    auto winIt = activeWindows.find(nodeId);
    if (winIt != activeWindows.end() && winIt->second != nullptr)
    {
        winIt->second->toFront(true);
        return true;
    }

    auto nodeIt = std::find_if(chainNodes.begin(), chainNodes.end(), [&](const auto& node) {
        return std::to_string(node->nodeID.uid) == nodeId;
    });

    if (nodeIt == chainNodes.end()) return false;

    auto* proc = (*nodeIt)->getProcessor();
    if (proc == nullptr || !proc->hasEditor()) return false;

    auto* editor = proc->createEditorIfNeeded();
    if (editor == nullptr) return false;

    juce::String procName = proc->getName();
    juce::String windowTitle;
    if (titlePrefix.length() > 0)
    {
        windowTitle = juce::String::fromUTF8(titlePrefix.c_str()) + " \xe2\x86\x92 " + procName;
    }
    else
    {
        windowTitle = "ShallowHost \xe2\x86\x92 Plugins \xe2\x86\x92 " + procName;
    }

    auto win = std::make_unique<PluginWindow>(nodeId, windowTitle, std::unique_ptr<juce::AudioProcessorEditor>(editor));
    win->setVisible(true);

#if defined(_WIN32) || defined(_WIN64)
    if (auto* peer = win->getPeer())
    {
        HWND hwnd = (HWND) peer->getNativeHandle();
        if (hwnd != NULL)
        {
            static HICON s_hIconBig = NULL;
            static HICON s_hIconSmall = NULL;
            static bool s_iconLoaded = false;
            if (!s_iconLoaded)
            {
                char exePath[MAX_PATH];
                if (GetModuleFileNameA(NULL, exePath, MAX_PATH) > 0)
                {
                    ExtractIconExA(exePath, 0, &s_hIconBig, &s_hIconSmall, 1);
                }
                s_iconLoaded = true;
            }
            if (s_hIconSmall != NULL)
            {
                SendMessageA(hwnd, WM_SETICON, ICON_SMALL, (LPARAM)s_hIconSmall);
            }
            if (s_hIconBig != NULL)
            {
                SendMessageA(hwnd, WM_SETICON, ICON_BIG, (LPARAM)s_hIconBig);
            }
        }
    }
#endif

    activeWindows[nodeId] = std::move(win);
    return true;
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

bool ShallowHost::closePluginGuiOnMessageThread(const std::string& nodeId)
{
    auto it = activeWindows.find(nodeId);
    if (it != activeWindows.end())
    {
        auto win = std::move(it->second);
        activeWindows.erase(it);
        juce::MessageManager::callAsync([w = std::move(win)]() mutable {
            w.reset();
        });
        return true;
    }
    return false;
}
