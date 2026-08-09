// JUCE-name compatibility for the clean-room Xaymar/vst2sdk ABI definitions.
#pragma once

using VstInt16 = int16_t;
using VstInt32 = int32_t;
using VstInt64 = int64_t;
using VstIntPtr = intptr_t;

struct AEffect;

#ifndef VSTCALLBACK
 #if defined(_WIN32)
  #define VSTCALLBACK __cdecl
 #else
  #define VSTCALLBACK
 #endif
#endif

using audioMasterCallback = VstIntPtr (VSTCALLBACK*) (AEffect*, VstInt32, VstInt32, VstIntPtr, void*, float);
using AEffectDispatcherProc = VstIntPtr (VSTCALLBACK*) (AEffect*, VstInt32, VstInt32, VstIntPtr, void*, float);
using AEffectProcessProc = void (VSTCALLBACK*) (AEffect*, float**, float**, VstInt32);
using AEffectProcessDoubleProc = void (VSTCALLBACK*) (AEffect*, double**, double**, VstInt32);
using AEffectSetParameterProc = void (VSTCALLBACK*) (AEffect*, VstInt32, float);
using AEffectGetParameterProc = float (VSTCALLBACK*) (AEffect*, VstInt32);

enum : VstInt32 { kEffectMagic = VST_MAGICNUMBER };

struct ERect
{
    VstInt16 top, left, bottom, right;
};

enum AEffectOpcodes
{
    effOpen = VST_EFFECT_OPCODE_CREATE,
    effClose = VST_EFFECT_OPCODE_DESTROY,
    effSetProgram = VST_EFFECT_OPCODE_PROGRAM_SET,
    effGetProgram = VST_EFFECT_OPCODE_PROGRAM_GET,
    effSetProgramName = VST_EFFECT_OPCODE_PROGRAM_SET_NAME,
    effGetProgramName = VST_EFFECT_OPCODE_PROGRAM_GET_NAME,
    effGetParamLabel = VST_EFFECT_OPCODE_PARAM_LABEL,
    effGetParamDisplay = VST_EFFECT_OPCODE_PARAM_VALUE_TO_STRING,
    effGetParamName = VST_EFFECT_OPCODE_PARAM_NAME,
    effSetSampleRate = VST_EFFECT_OPCODE_SET_SAMPLE_RATE,
    effSetBlockSize = VST_EFFECT_OPCODE_SET_BLOCK_SIZE,
    effMainsChanged = VST_EFFECT_OPCODE_SUSPEND_RESUME,
    effEditGetRect = VST_EFFECT_OPCODE_EDITOR_GET_RECT,
    effEditOpen = VST_EFFECT_OPCODE_EDITOR_OPEN,
    effEditClose = VST_EFFECT_OPCODE_EDITOR_CLOSE,
    effEditIdle = VST_EFFECT_OPCODE_EDITOR_KEEP_ALIVE,
    effEditTop = VST_EFFECT_OPCODE_14,
    effIdentify = VST_EFFECT_OPCODE_FOURCC,
    effGetChunk = VST_EFFECT_OPCODE_GET_CHUNK_DATA,
    effSetChunk = VST_EFFECT_OPCODE_SET_CHUNK_DATA,
    effProcessEvents = VST_EFFECT_OPCODE_EVENT,
    effCanBeAutomated = VST_EFFECT_OPCODE_PARAM_IS_AUTOMATABLE,
    effGetProgramNameIndexed = VST_EFFECT_OPCODE_1D,
    effConnectInput = VST_EFFECT_OPCODE_1F,
    effConnectOutput = VST_EFFECT_OPCODE_20,
    effGetInputProperties = VST_EFFECT_OPCODE_INPUT_GET_PROPERTIES,
    effGetOutputProperties = VST_EFFECT_OPCODE_OUTPUT_GET_PROPERTIES,
    effGetPlugCategory = VST_EFFECT_OPCODE_CATEGORY,
    effSetSpeakerArrangement = VST_EFFECT_OPCODE_SET_SPEAKER_ARRANGEMENT,
    effSetBypass = VST_EFFECT_OPCODE_BYPASS,
    effGetEffectName = VST_EFFECT_OPCODE_EFFECT_NAME,
    effGetVendorString = VST_EFFECT_OPCODE_VENDOR_NAME,
    effGetProductString = VST_EFFECT_OPCODE_PRODUCT_NAME,
    effGetVendorVersion = VST_EFFECT_OPCODE_VENDOR_VERSION,
    effVendorSpecific = VST_EFFECT_OPCODE_CUSTOM,
    effCanDo = VST_EFFECT_OPCODE_SUPPORTS,
    effGetTailSize = VST_EFFECT_OPCODE_TAIL_SAMPLES,
    effIdle = VST_EFFECT_OPCODE_IDLE,
    effKeysRequired = VST_EFFECT_OPCODE_39,
    effGetMidiKeyName = VST_EFFECT_OPCODE_42,
    effGetSpeakerArrangement = VST_EFFECT_OPCODE_GET_SPEAKER_ARRANGEMENT,
    effShellGetNextPlugin = VST_EFFECT_OPCODE_CONTAINER_NEXT_EFFECT_ID,
    effStartProcess = VST_EFFECT_OPCODE_PROCESS_BEGIN,
    effStopProcess = VST_EFFECT_OPCODE_PROCESS_END,
    effSetProcessPrecision = VST_EFFECT_OPCODE_4D
};

enum AEffectFlags
{
    effFlagsHasEditor = VST_EFFECT_FLAG_EDITOR,
    effFlagsCanReplacing = VST_EFFECT_FLAG_SUPPORTS_FLOAT,
    effFlagsProgramChunks = VST_EFFECT_FLAG_CHUNKS,
    effFlagsIsSynth = VST_EFFECT_FLAG_INSTRUMENT,
    effFlagsNoSoundInStop = VST_EFFECT_FLAG_SILENT_TAIL,
    effFlagsCanDoubleReplacing = VST_EFFECT_FLAG_SUPPORTS_DOUBLE
};

struct AEffect
{
    VstInt32 magic;
    AEffectDispatcherProc dispatcher;
    AEffectProcessProc process;
    AEffectSetParameterProc setParameter;
    AEffectGetParameterProc getParameter;
    VstInt32 numPrograms, numParams, numInputs, numOutputs, flags;
    VstIntPtr resvd1, resvd2;
    VstInt32 initialDelay, realQualities, offQualities;
    float ioRatio;
    void* object;
    void* user;
    VstInt32 uniqueID, version;
    AEffectProcessProc processReplacing;
    AEffectProcessDoubleProc processDoubleReplacing;
    char future[56];
};

static_assert(sizeof(ERect) == sizeof(::vst_rect_t));
static_assert(sizeof(AEffect) == sizeof(::vst_effect_t));
static_assert(offsetof(AEffect, magic) == offsetof(::vst_effect_t, magic_number));
static_assert(offsetof(AEffect, dispatcher) == offsetof(::vst_effect_t, control));
static_assert(offsetof(AEffect, initialDelay) == offsetof(::vst_effect_t, delay));
static_assert(offsetof(AEffect, object) == offsetof(::vst_effect_t, effect_internal));
static_assert(offsetof(AEffect, uniqueID) == offsetof(::vst_effect_t, unique_id));
static_assert(offsetof(AEffect, processReplacing) == offsetof(::vst_effect_t, process_float));
static_assert(offsetof(AEffect, processDoubleReplacing) == offsetof(::vst_effect_t, process_double));
