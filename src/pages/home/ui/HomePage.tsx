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
import { ArrowRight, Plus, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/shared/lib/utils'
import { useAudioConfigStore } from '@/shared/model/audio-config-store'
import { useChainStore } from '@/shared/model/chain-store'
import { Button } from '@/shared/ui/button'
import { buttonVariants } from '@/shared/ui/button-variants'
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/shared/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/shared/ui/dialog'
import { Separator } from '@/shared/ui/separator'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import { handleAudioConfigUpdate, resolveDevice } from '../lib/audio-config-actions'
import { clearChainWithUndo } from '../lib/clear-chain-action'
import { AudioConfigCard } from './AudioConfigCard'
import { SortableChainCard } from './SortableChainCard'

let devicesCache: AudioDevices = { inputs: [], outputs: [] }

export function HomePage() {
  const { t } = useTranslation()
  const [error, setError] = useState<string | null>(null)
  const [confirmClearOpen, setConfirmClearOpen] = useState(false)
  const chain = useChainStore(s => s.chain)
  const refreshChain = useChainStore(s => s.refresh)
  const config = useAudioConfigStore(s => s.config)
  const updateConfigStore = useAudioConfigStore(s => s.updateConfig)
  const loadFromBackend = useAudioConfigStore(s => s.loadFromBackend)

  const [devices, setDevicesState] = useState<AudioDevices>(devicesCache)
  const setDevices = (d: AudioDevices) => {
    devicesCache = d
    setDevicesState(d)
  }

  const clearChain = async () => {
    setConfirmClearOpen(false)
    await clearChainWithUndo(t)
  }

  useEffect(() => {
    async function init() {
      refreshChain()
      await loadFromBackend()
      invoke('app_ready').catch(console.error)
      const devs = await invoke<AudioDevices>('get_audio_devices')
      setDevices(devs)

      const currentConfig = useAudioConfigStore.getState().config

      if (currentConfig.driver === 'wasapi') {
        const patch: Partial<AudioConfig> = {}
        let needsUpdate = false
        const store = useAudioConfigStore.getState()
        const isStaleOrEmpty = (dev: string | null, list: DeviceInfo[]) =>
          !dev
          || dev === '__default'
          || (dev !== '__none' && !list.some(d => d.name === dev))
        if (isStaleOrEmpty(currentConfig.input_device, devs.inputs)) {
          patch.input_device = resolveDevice(store.lastWasapiInput, devs.inputs)
          needsUpdate = true
        }
        if (isStaleOrEmpty(currentConfig.output_device, devs.outputs)) {
          patch.output_device = resolveDevice(store.lastWasapiOutput, devs.outputs)
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

    let active = true
    const timerId = setTimeout(async () => {
      try {
        const fresh = await invoke<AudioDevices>('get_audio_devices')
        if (active)
          setDevices(fresh)
      }
      catch {}
    }, 300)

    return () => {
      active = false
      clearTimeout(timerId)
    }
  }, [refreshChain, loadFromBackend, updateConfigStore])

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

  const updateConfig = (patch: Partial<AudioConfig>) => {
    return handleAudioConfigUpdate(patch, config, updateConfigStore, setDevices, setError)
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
            <Link
              to="/plugins"
              className={cn(buttonVariants({ variant: 'default' }), 'cursor-pointer')}
            >
              {t('home.goToPlugins')}
              <ArrowRight className="size-4" data-icon="inline-end" />
            </Link>
            <Tooltip>
              <TooltipTrigger render={(
                <Button
                  variant="outline"
                  size="icon"
                  disabled={chain.length === 0}
                  onClick={() => setConfirmClearOpen(true)}
                  className="cursor-pointer hover:!bg-red/10 hover:!text-red hover:!border-red/20 disabled:pointer-events-none disabled:opacity-50"
                  aria-label={t('home.clearChain')}
                >
                  <Trash2 className="size-4" />
                </Button>
              )}
              />
              <TooltipContent>{t('home.clearChain')}</TooltipContent>
            </Tooltip>
          </CardAction>
        </CardHeader>
        <Separator />
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

      <Dialog open={confirmClearOpen} onOpenChange={setConfirmClearOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogTitle>{t('home.clearChainTitle')}</DialogTitle>
          <DialogDescription>
            {t('home.clearChainDescription')}
          </DialogDescription>
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="outline"
              onClick={() => setConfirmClearOpen(false)}
            >
              {t('home.cancel')}
            </Button>
            <Button
              variant="destructive"
              onClick={clearChain}
            >
              {t('home.clearAll')}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
