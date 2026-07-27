import type { DeviceInfo } from '../ui/DeviceSelect'
import type { AudioConfig } from '@/shared/model/audio-config-store'
import { invoke } from '@tauri-apps/api/core'
import { useAudioConfigStore } from '@/shared/model/audio-config-store'

interface AudioDevices {
  inputs: DeviceInfo[]
  outputs: DeviceInfo[]
  input_channels?: string[] | null
  output_channels?: string[] | null
}

async function prepareDriverSwitch(
  currentConfig: AudioConfig,
  nextPatch: Partial<AudioConfig>,
  updateConfigStore: (patch: Partial<AudioConfig>) => void,
  setDevices: (devs: AudioDevices) => void,
) {
  nextPatch.input_device = null
  nextPatch.output_device = null
  nextPatch.active_inputs = null
  nextPatch.active_outputs = null
  updateConfigStore(nextPatch)
  await invoke('set_audio_config', { config: { ...currentConfig, ...nextPatch } })
  const freshDevs = await invoke<AudioDevices>('get_audio_devices')
  setDevices(freshDevs)
  return freshDevs
}

async function handleAsioSwitch(
  currentConfig: AudioConfig,
  nextPatch: Partial<AudioConfig>,
  updateConfigStore: (patch: Partial<AudioConfig>) => void,
  setDevices: (devs: AudioDevices) => void,
) {
  if (currentConfig.driver === 'wasapi') {
    useAudioConfigStore.setState({
      lastWasapiInput: currentConfig.input_device,
      lastWasapiOutput: currentConfig.output_device,
    })
  }
  const freshDevs = await prepareDriverSwitch(currentConfig, nextPatch, updateConfigStore, setDevices)
  const last = useAudioConfigStore.getState().lastAsioDevice
  const restore = last && freshDevs.outputs.some((d: DeviceInfo) => d.name === last) ? last : '__none'
  nextPatch.input_device = restore
  nextPatch.output_device = restore
  const asioState = useAudioConfigStore.getState()
  nextPatch.active_inputs = restore !== '__none' ? asioState.lastAsioInputs : null
  nextPatch.active_outputs = restore !== '__none' ? asioState.lastAsioOutputs : null
}

async function handleWasapiSwitch(
  currentConfig: AudioConfig,
  nextPatch: Partial<AudioConfig>,
  updateConfigStore: (patch: Partial<AudioConfig>) => void,
  setDevices: (devs: AudioDevices) => void,
) {
  if (currentConfig.driver === 'asio' && currentConfig.output_device && currentConfig.output_device !== '__none') {
    useAudioConfigStore.setState({
      lastAsioDevice: currentConfig.output_device,
      lastAsioInputs: currentConfig.active_inputs ?? null,
      lastAsioOutputs: currentConfig.active_outputs ?? null,
    })
  }
  const freshDevs = await prepareDriverSwitch(currentConfig, nextPatch, updateConfigStore, setDevices)
  const wasapiState = useAudioConfigStore.getState()
  const restoreIn = wasapiState.lastWasapiInput && (wasapiState.lastWasapiInput === '__none' || freshDevs.inputs.some((d: DeviceInfo) => d.name === wasapiState.lastWasapiInput))
    ? wasapiState.lastWasapiInput
    : '__none'
  const restoreOut = wasapiState.lastWasapiOutput && (wasapiState.lastWasapiOutput === '__none' || freshDevs.outputs.some((d: DeviceInfo) => d.name === wasapiState.lastWasapiOutput))
    ? wasapiState.lastWasapiOutput
    : '__none'
  nextPatch.input_device = restoreIn
  nextPatch.output_device = restoreOut
}

export async function handleAudioConfigUpdate(
  patch: Partial<AudioConfig>,
  currentConfig: AudioConfig,
  updateConfigStore: (patch: Partial<AudioConfig>) => void,
  setDevices: (devs: AudioDevices) => void,
  setError: (err: string | null) => void,
) {
  const nextPatch = { ...patch }

  if (patch.driver && patch.driver !== currentConfig.driver) {
    if (patch.driver === 'asio') {
      await handleAsioSwitch(currentConfig, nextPatch, updateConfigStore, setDevices)
    }
    else {
      await handleWasapiSwitch(currentConfig, nextPatch, updateConfigStore, setDevices)
    }
  }

  updateConfigStore(nextPatch)
  const next = { ...currentConfig, ...nextPatch }
  try {
    await invoke('set_audio_config', { config: next })
    await invoke('restart_audio')
    const devs = await invoke<AudioDevices>('get_audio_devices')
    setDevices(devs)
  }
  catch (e) {
    setError(String(e))
  }
}
