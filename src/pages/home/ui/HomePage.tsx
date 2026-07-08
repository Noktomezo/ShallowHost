import type { DragEndEvent } from '@dnd-kit/core'
import type { AudioDevices, DeviceInfo } from './AudioConfigCard'
import type { AudioConfig } from '@/shared/model/audio-config-store'
import {
  DndContext,

  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { restrictToVerticalAxis } from '@dnd-kit/modifiers'
import {
  SortableContext,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { Link } from '@tanstack/react-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ArrowRight, Paintbrush, Plus } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAudioConfigStore } from '@/shared/model/audio-config-store'
import { useChainStore } from '@/shared/model/chain-store'
import { Button } from '@/shared/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import { AudioConfigCard } from './AudioConfigCard'
import { SortableChainCard } from './SortableChainCard'

// ponytail: module-level cache survives route unmount/remount — avoids
// channel section flash while get_audio_devices fetch is in-flight.
let devicesCache: AudioDevices = { inputs: [], outputs: [] }

export function HomePage() {
  const { t } = useTranslation()
  const [error, setError] = useState<string | null>(null)
  const chain = useChainStore(s => s.chain)
  const refreshChain = useChainStore(s => s.refresh)
  const config = useAudioConfigStore(s => s.config)
  const updateConfigStore = useAudioConfigStore(s => s.updateConfig)
  const loadFromBackend = useAudioConfigStore(s => s.loadFromBackend)
  // ponytail: module-level cache survives route unmount/remount — avoids
  // "No channels available" flash while get_audio_devices fetch is in-flight.
  const [devices, setDevicesState] = useState<AudioDevices>(devicesCache)
  const setDevices = (d: AudioDevices) => {
    devicesCache = d
    setDevicesState(d)
  }

  const clearChain = async () => {
    try {
      for (const p of chain) {
        await invoke('remove_from_chain', { pluginId: p.id })
      }
      await refreshChain()
    }
    catch (e) {
      console.error(e)
    }
  }

  useEffect(() => {
    async function init() {
      refreshChain()
      await loadFromBackend()
      const devs = await invoke<AudioDevices>('get_audio_devices')
      setDevices(devs)

      // Get the freshly updated config from store
      const currentConfig = useAudioConfigStore.getState().config

      // Auto-select system defaults if config is empty and driver is wasapi
      if (currentConfig.driver === 'wasapi') {
        const patch: Partial<AudioConfig> = {}
        let needsUpdate = false
        // ponytail: also heal stale configs (e.g. ASIO device name saved under
        // WASAPI driver from a prior bug) — if the device isn't in the fresh
        // list, reset to default. __none is a valid user choice, keep it.
        const isStaleOrEmpty = (dev: string | null, list: DeviceInfo[]) =>
          !dev
          || dev === '__default'
          || (dev !== '__none' && !list.some(d => d.name === dev))
        // ponytail: restore last WASAPI device with __none fallback — no auto-default.
        const resolveWasapi = (saved: string | null, list: DeviceInfo[]) =>
          saved && saved !== '__default' && (saved === '__none' || list.some(d => d.name === saved))
            ? saved
            : '__none'
        const store = useAudioConfigStore.getState()
        if (isStaleOrEmpty(currentConfig.input_device, devs.inputs)) {
          patch.input_device = resolveWasapi(store.lastWasapiInput, devs.inputs)
          needsUpdate = true
        }
        if (isStaleOrEmpty(currentConfig.output_device, devs.outputs)) {
          patch.output_device = resolveWasapi(store.lastWasapiOutput, devs.outputs)
          needsUpdate = true
        }
        if (needsUpdate) {
          updateConfigStore(patch)
          const updated = { ...currentConfig, ...patch }
          await invoke('set_audio_config', { config: updated })
          await invoke('restart_audio')
          const freshDevs = await invoke<AudioDevices>('get_audio_devices')
          setDevices(freshDevs)
        }
      }
    }
    init().catch(() => {})
  }, [refreshChain, loadFromBackend])

  // ponytail: backend polls devices every 500ms for hotplug; react to events
  // instead of polling from frontend (avoids IPC spam + UI hangs).
  useEffect(() => {
    const unlistenDevices = listen<AudioDevices>('audio-devices-changed', (e) => {
      setDevices(e.payload)
    })
    const unlistenConfig = listen('audio-config-changed', () => {
      loadFromBackend()
    })
    return () => {
      unlistenDevices.then(fn => fn())
      unlistenConfig.then(fn => fn())
    }
  }, [loadFromBackend])

  async function updateConfig(patch: Partial<AudioConfig>) {
    const nextPatch = { ...patch }
    if (patch.driver && patch.driver !== config.driver) {
      if (patch.driver === 'asio') {
        // ponytail: stash current WASAPI devices for restore on switch-back.
        if (config.driver === 'wasapi') {
          useAudioConfigStore.setState({
            lastWasapiInput: config.input_device,
            lastWasapiOutput: config.output_device,
          })
        }
        // ponytail: restore last-used ASIO device if still present, else __none.
        // Fetch fresh ASIO list first — `devices` holds the WASAPI list here.
        nextPatch.input_device = null
        nextPatch.output_device = null
        nextPatch.active_inputs = null
        nextPatch.active_outputs = null
        updateConfigStore(nextPatch)
        await invoke('set_audio_config', { config: { ...config, ...nextPatch } })
        const freshDevs = await invoke<AudioDevices>('get_audio_devices')
        setDevices(freshDevs)
        const last = useAudioConfigStore.getState().lastAsioDevice
        const restore = last && freshDevs.outputs.some(d => d.name === last) ? last : '__none'
        nextPatch.input_device = restore
        nextPatch.output_device = restore
        // ponytail: restore stashed channels too — indices are bit-flags, device
        // ignores invalid bits; same-device restore is exact.
        const asioState = useAudioConfigStore.getState()
        nextPatch.active_inputs = restore !== '__none' ? asioState.lastAsioInputs : null
        nextPatch.active_outputs = restore !== '__none' ? asioState.lastAsioOutputs : null
      }
      else {
        // ponytail: `devices` still holds the ASIO list here, so picking defaults
        // from it copies the ASIO device name across → "No such device" on start.
        // Push driver change to backend first, enumerate fresh WASAPI devices,
        // then restore last WASAPI device with __none fallback (no auto-default).
        // Stash current ASIO device for restore on switch-back.
        if (config.driver === 'asio' && config.output_device && config.output_device !== '__none') {
          useAudioConfigStore.setState({
            lastAsioDevice: config.output_device,
            lastAsioInputs: config.active_inputs ?? null,
            lastAsioOutputs: config.active_outputs ?? null,
          })
        }
        nextPatch.input_device = null
        nextPatch.output_device = null
        nextPatch.active_inputs = null
        nextPatch.active_outputs = null
        updateConfigStore(nextPatch)
        await invoke('set_audio_config', { config: { ...config, ...nextPatch } })
        const freshDevs = await invoke<AudioDevices>('get_audio_devices')
        setDevices(freshDevs)
        const wasapiState = useAudioConfigStore.getState()
        const restoreIn = wasapiState.lastWasapiInput && (wasapiState.lastWasapiInput === '__none' || freshDevs.inputs.some(d => d.name === wasapiState.lastWasapiInput))
          ? wasapiState.lastWasapiInput
          : '__none'
        const restoreOut = wasapiState.lastWasapiOutput && (wasapiState.lastWasapiOutput === '__none' || freshDevs.outputs.some(d => d.name === wasapiState.lastWasapiOutput))
          ? wasapiState.lastWasapiOutput
          : '__none'
        nextPatch.input_device = restoreIn
        nextPatch.output_device = restoreOut
      }
    }
    updateConfigStore(nextPatch)
    const next = { ...config, ...nextPatch }
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

  async function reorderPlugin(id: string, toIndex: number) {
    try {
      await invoke('reorder_chain', { pluginId: id, toIndex })
      await refreshChain()
    }
    catch (e) {
      setError(String(e))
    }
  }

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 5 } }),
  )

  function handleDragEnd(e: DragEndEvent) {
    const { active, over } = e
    if (!over || active.id === over.id)
      return
    const toIndex = chain.findIndex(p => p.id === over.id)
    if (toIndex < 0)
      return
    reorderPlugin(active.id as string, toIndex)
  }

  return (
    <div className="flex flex-col gap-0.5">
      <h1 className="text-xl font-semibold">{t('home.title')}</h1>
      <p className="text-sm text-muted-foreground">{t('home.description')}</p>

      <div className="mt-3">
        <AudioConfigCard
          config={config}
          devices={devices}
          updateConfig={updateConfig}
        />
      </div>

      {error && (
        <p className="mt-2 text-sm text-destructive">{error}</p>
      )}

      <Card className="mt-3 w-full">
        <CardHeader className="gap-0.5">
          <CardTitle>{t('home.chain')}</CardTitle>
          <CardDescription>{t('home.addHint')}</CardDescription>
          <CardAction className="flex items-center gap-1.5 self-center">
            <Link to="/plugins">
              <Tooltip>
                <TooltipTrigger render={(
                  <Button
                    variant="default"
                    size="icon"
                    className="cursor-pointer"
                    aria-label={t('home.goToPlugins')}
                  >
                    <ArrowRight className="size-4" />
                  </Button>
                )}
                />
                <TooltipContent>{t('home.goToPlugins')}</TooltipContent>
              </Tooltip>
            </Link>
            <Tooltip>
              <TooltipTrigger render={(
                <Button
                  variant="outline"
                  size="icon"
                  disabled={chain.length === 0}
                  onClick={clearChain}
                  className="cursor-pointer hover:!bg-red/10 hover:!text-red hover:!border-red/20 disabled:pointer-events-none disabled:opacity-50"
                  aria-label={t('home.clearChain')}
                >
                  <Paintbrush className="size-4" />
                </Button>
              )}
              />
              <TooltipContent>{t('home.clearChain')}</TooltipContent>
            </Tooltip>
          </CardAction>
        </CardHeader>
        <CardContent>
          {chain.length > 0
            ? (
                <DndContext
                  sensors={sensors}
                  modifiers={[restrictToVerticalAxis]}
                  onDragEnd={handleDragEnd}
                >
                  <SortableContext items={chain.map(p => p.id)} strategy={verticalListSortingStrategy}>
                    <div className="flex flex-col gap-2">
                      {chain.map(p => (
                        <SortableChainCard key={p.id} plugin={p} />
                      ))}
                    </div>
                  </SortableContext>
                </DndContext>
              )
            : (
                <div className="flex items-center gap-2 pt-2 text-sm text-muted-foreground">
                  <Plus className="size-4" />
                  {t('home.chainEmpty')}
                </div>
              )}
        </CardContent>
      </Card>
    </div>
  )
}
