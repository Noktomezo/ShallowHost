import type { DragEndEvent } from '@dnd-kit/core'
import type { AudioDevices } from './AudioConfigCard'
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
import { invoke } from '@tauri-apps/api/core'
import { Plus } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAudioConfigStore } from '@/shared/model/audio-config-store'
import { useChainStore } from '@/shared/model/chain-store'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { AudioConfigCard } from './AudioConfigCard'
import { SortableChainCard } from './SortableChainCard'

export function HomePage() {
  const { t } = useTranslation()
  const [error, setError] = useState<string | null>(null)
  const chain = useChainStore(s => s.chain)
  const refreshChain = useChainStore(s => s.refresh)
  const config = useAudioConfigStore(s => s.config)
  const updateConfigStore = useAudioConfigStore(s => s.updateConfig)
  const loadFromBackend = useAudioConfigStore(s => s.loadFromBackend)
  const [devices, setDevices] = useState<AudioDevices>({ inputs: [], outputs: [] })

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
        if (!currentConfig.input_device || currentConfig.input_device === '__default') {
          patch.input_device = devs.inputs.find(d => d.default)?.name ?? '__none'
          needsUpdate = true
        }
        if (!currentConfig.output_device || currentConfig.output_device === '__default') {
          patch.output_device = devs.outputs.find(d => d.default)?.name ?? '__none'
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

  async function updateConfig(patch: Partial<AudioConfig>) {
    const nextPatch = { ...patch }
    if (patch.driver && patch.driver !== config.driver) {
      if (patch.driver === 'asio') {
        nextPatch.input_device = '__none'
        nextPatch.output_device = '__none'
        nextPatch.active_inputs = null
        nextPatch.active_outputs = null
      }
      else {
        // Auto-select defaults from the loaded devices list
        const defaultIn = devices.inputs.find(d => d.default)?.name ?? '__none'
        const defaultOut = devices.outputs.find(d => d.default)?.name ?? '__none'
        nextPatch.input_device = defaultIn
        nextPatch.output_device = defaultOut
        nextPatch.active_inputs = null
        nextPatch.active_outputs = null
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
