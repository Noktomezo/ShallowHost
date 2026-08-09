// Extended JUCE-name compatibility for Xaymar/vst2sdk.
#pragma once

#include "aeffect.h"

enum AudioMasterOpcodes
{
    audioMasterAutomate = VST_HOST_OPCODE_AUTOMATE,
    audioMasterVersion = VST_HOST_OPCODE_VST_VERSION,
    audioMasterCurrentId = VST_HOST_OPCODE_CURRENT_EFFECT_ID,
    audioMasterIdle = VST_HOST_OPCODE_KEEPALIVE_OR_IDLE,
    audioMasterPinConnected = 4,
    audioMasterWantMidi = 6,
    audioMasterGetTime = 7,
    audioMasterProcessEvents = 8,
    audioMasterSetTime = 9,
    audioMasterTempoAt = 10,
    audioMasterGetNumAutomatableParameters = 11,
    audioMasterGetParameterQuantization = 12,
    audioMasterIOChanged = VST_HOST_OPCODE_IO_MODIFIED,
    audioMasterNeedIdle = 14,
    audioMasterSizeWindow = VST_HOST_OPCODE_EDITOR_RESIZE,
    audioMasterGetSampleRate = VST_HOST_OPCODE_GET_SAMPLE_RATE,
    audioMasterGetBlockSize = VST_HOST_OPCODE_GET_BLOCK_SIZE,
    audioMasterGetInputLatency = VST_HOST_OPCODE_INPUT_LATENCY,
    audioMasterGetOutputLatency = VST_HOST_OPCODE_OUTPUT_LATENCY,
    audioMasterGetPreviousPlug = VST_HOST_OPCODE_INPUT_GET_ATTACHED_EFFECT,
    audioMasterGetNextPlug = VST_HOST_OPCODE_OUTPUT_GET_ATTACHED_EFFECT,
    audioMasterWillReplaceOrAccumulate = 22,
    audioMasterGetCurrentProcessLevel = VST_HOST_OPCODE_GET_ACTIVE_THREAD,
    audioMasterGetAutomationState = 24,
    audioMasterOfflineStart = 25,
    audioMasterOfflineRead = 26,
    audioMasterOfflineWrite = 27,
    audioMasterOfflineGetCurrentPass = 28,
    audioMasterOfflineGetCurrentMetaPass = 29,
    audioMasterSetOutputSampleRate = 30,
    audioMasterGetOutputSpeakerArrangement = VST_HOST_OPCODE_OUTPUT_GET_SPEAKER_ARRANGEMENT,
    audioMasterGetVendorString = VST_HOST_OPCODE_VENDOR_NAME,
    audioMasterGetProductString = VST_HOST_OPCODE_PRODUCT_NAME,
    audioMasterGetVendorVersion = VST_HOST_OPCODE_VENDOR_VERSION,
    audioMasterVendorSpecific = VST_HOST_OPCODE_CUSTOM,
    audioMasterSetIcon = 36,
    audioMasterCanDo = VST_HOST_OPCODE_SUPPORTS,
    audioMasterGetLanguage = VST_HOST_OPCODE_LANGUAGE,
    audioMasterOpenWindow = 39,
    audioMasterCloseWindow = 40,
    audioMasterGetDirectory = VST_HOST_OPCODE_GET_EFFECT_DIRECTORY,
    audioMasterUpdateDisplay = VST_HOST_OPCODE_REFRESH,
    audioMasterBeginEdit = VST_HOST_OPCODE_PARAM_START_EDIT,
    audioMasterEndEdit = VST_HOST_OPCODE_PARAM_STOP_EDIT
};

enum VstEventTypes { kVstMidiType = VST_EVENT_TYPE_MIDI, kVstSysExType = VST_EVENT_TYPE_MIDI_SYSEX };
enum VstTimeInfoFlags
{
    kVstTransportChanged = 1 << 0,
    kVstTransportPlaying = 1 << 1,
    kVstTransportCycleActive = 1 << 2,
    kVstTransportRecording = 1 << 3,
    kVstAutomationWriting = 1 << 6,
    kVstAutomationReading = 1 << 7,
    kVstNanosValid = 1 << 8,
    kVstPpqPosValid = 1 << 9,
    kVstTempoValid = 1 << 10,
    kVstBarsValid = 1 << 11,
    kVstCyclePosValid = 1 << 12,
    kVstTimeSigValid = 1 << 13,
    kVstSmpteValid = 1 << 14
};

using VstSmpteFrameRate = VstInt32;
enum : VstSmpteFrameRate
{
    kVstSmpte24fps = 0, kVstSmpte25fps = 1, kVstSmpte2997fps = 2, kVstSmpte30fps = 3,
    kVstSmpte2997dfps = 4, kVstSmpte30dfps = 5, kVstSmpte239fps = 10,
    kVstSmpte249fps = 11, kVstSmpte599fps = 12, kVstSmpte60fps = 13
};
enum VstProcessPrecision { kVstProcessPrecision32 = 0, kVstProcessPrecision64 = 1 };

using VstPlugCategory = VstInt32;
enum : VstPlugCategory
{
    kPlugCategUnknown = VST_EFFECT_CATEGORY_UNCATEGORIZED,
    kPlugCategEffect = VST_EFFECT_CATEGORY_EFFECT,
    kPlugCategSynth = VST_EFFECT_CATEGORY_INSTRUMENT,
    kPlugCategAnalysis = VST_EFFECT_CATEGORY_METERING,
    kPlugCategMastering = VST_EFFECT_CATEGORY_MASTERING,
    kPlugCategSpacializer = VST_EFFECT_CATEGORY_SPATIAL,
    kPlugCategRoomFx = VST_EFFECT_CATEGORY_DELAY_OR_ECHO,
    kPlugSurroundFx = VST_EFFECT_CATEGORY_EXTERNAL,
    kPlugCategRestoration = VST_EFFECT_CATEGORY_RESTORATION,
    kPlugCategOfflineProcess = VST_EFFECT_CATEGORY_OFFLINE,
    kPlugCategShell = VST_EFFECT_CATEGORY_CONTAINER,
    kPlugCategGenerator = VST_EFFECT_CATEGORY_WAVEGENERATOR,
    kPlugCategMaxCount = VST_EFFECT_CATEGORY_MAX
};

using VstSpeakerArrangementType = VstInt32;
enum : VstSpeakerArrangementType
{
    kSpeakerArrUserDefined = -2, kSpeakerArrEmpty = -1, kSpeakerArrMono = 0,
    kSpeakerArrStereo = 1, kSpeakerArrStereoSurround = 2, kSpeakerArrStereoCenter = 3,
    kSpeakerArrStereoSide = 4, kSpeakerArrStereoCLfe = 5, kSpeakerArr30Cine = 6,
    kSpeakerArr30Music = 7, kSpeakerArr31Cine = 8, kSpeakerArr31Music = 9,
    kSpeakerArr40Cine = 10, kSpeakerArr40Music = 11, kSpeakerArr41Cine = 12,
    kSpeakerArr41Music = 13, kSpeakerArr50 = 14, kSpeakerArr51 = 15,
    kSpeakerArr60Cine = 16, kSpeakerArr60Music = 17, kSpeakerArr61Cine = 18,
    kSpeakerArr61Music = 19, kSpeakerArr70Cine = 20, kSpeakerArr70Music = 21,
    kSpeakerArr71Cine = 22, kSpeakerArr71Music = 23, kSpeakerArr80Cine = 24,
    kSpeakerArr80Music = 25, kSpeakerArr81Cine = 26, kSpeakerArr81Music = 27,
    kSpeakerArr102 = 28
};

using VstSpeakerType = VstInt32;
enum : VstSpeakerType
{
    kSpeakerL = 1, kSpeakerR = 2, kSpeakerC = 3, kSpeakerLfe = 4, kSpeakerLs = 5,
    kSpeakerRs = 6, kSpeakerLc = 7, kSpeakerRc = 8, kSpeakerS = 9, kSpeakerSl = 10,
    kSpeakerSr = 11, kSpeakerTm = 12, kSpeakerTfl = 13, kSpeakerTfc = 14,
    kSpeakerTfr = 15, kSpeakerTrl = 16, kSpeakerTrc = 17, kSpeakerTrr = 18, kSpeakerLfe2 = 19
};

enum
{
    kVstMaxNameLen = 64, kVstMaxLabelLen = 8, kVstMaxShortLabelLen = 8,
    kVstMaxProductStrLen = 64, kVstMaxVendorStrLen = 64,
    kVstPinIsStereo = VST_STREAM_FLAG_STEREO, kVstPinUseSpeaker = VST_STREAM_FLAG_USE_TYPE
};

struct VstEvent { VstInt32 type, byteSize, deltaFrames, flags; char data[16]; };
struct VstMidiEvent
{
    VstInt32 type, byteSize, deltaFrames, flags, noteLength, noteOffset;
    char midiData[4], detune, noteOffVelocity, reserved1, reserved2;
};
struct VstMidiSysexEvent
{
    VstInt32 type, byteSize, deltaFrames, flags, dumpBytes;
    VstIntPtr resvd1;
    char* sysexDump;
    VstIntPtr resvd2;
};
struct VstEvents { VstInt32 numEvents; VstIntPtr reserved; VstEvent* events[2]; };
struct VstTimeInfo
{
    double samplePos, sampleRate, nanoSeconds, ppqPos, tempo, barStartPos, cycleStartPos, cycleEndPos;
    VstInt32 timeSigNumerator, timeSigDenominator, smpteOffset;
    VstSmpteFrameRate smpteFrameRate;
    VstInt32 samplesToNextClock, flags;
};
struct VstSpeakerProperties
{
    float azimuth, elevation, radius, reserved;
    char name[kVstMaxNameLen];
    VstSpeakerType type;
    char future[28];
};
struct VstSpeakerArrangement
{
    VstInt32 type, numChannels;
    VstSpeakerProperties speakers[8];
};
struct VstPinProperties
{
    char label[kVstMaxNameLen];
    VstInt32 flags;
    VstSpeakerArrangementType arrangementType;
    char shortLabel[kVstMaxShortLabelLen];
    char future[48];
};
struct MidiKeyName
{
    VstInt32 thisProgramIndex, thisKeyNumber;
    char keyName[kVstMaxNameLen];
    VstInt32 reserved;
    char future[64];
};

static_assert(sizeof(VstSpeakerProperties) == sizeof(::vst_speaker_properties_t));
static_assert(sizeof(VstEvent) == sizeof(::vst_event_t));
static_assert(sizeof(VstMidiEvent) == sizeof(::vst_event_midi_t));
static_assert(sizeof(VstMidiSysexEvent) == sizeof(::vst_event_midi_sysex_t));
static_assert(offsetof(VstSpeakerProperties, type) == offsetof(::vst_speaker_properties_t, type));
static_assert(offsetof(VstSpeakerArrangement, speakers) == offsetof(::vst_speaker_arrangement_t, speakers));
static_assert(sizeof(VstPinProperties) == sizeof(::vst_stream_properties_t));
