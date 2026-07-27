import type { ChainItem } from '@/shared/model/chain-store'
import { invoke } from '@tauri-apps/api/core'
import { AppWindow, Ban, Circle, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useChainStore } from '@/shared/model/chain-store'
import { Button } from '@/shared/ui/button'
import { CardAction } from '@/shared/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import { ChainParamsButton } from './ChainParamsButton'

async function openGui(id: string, titlePrefix: string) {
  try {
    await invoke('open_plugin_gui', { pluginId: id, titlePrefix })
  }
  catch (e) {
    console.error(e)
  }
}

function bypassPlugin(id: string, bypassed: boolean) {
  invoke('bypass_plugin', { pluginId: id, bypassed: !bypassed })
    .then(() => {
      useChainStore.getState().updateChainItem(id, { bypassed: !bypassed })
    })
    .catch(console.error)
}

function removeFromChain(p: ChainItem) {
  invoke('remove_from_chain', { pluginId: p.id })
    .then(() => {
      useChainStore.getState().remove(p.id)
      toast(p.name, {
        description: 'Plugin removed from chain',
        action: p.unique_id
          ? {
              label: 'Undo',
              onClick: () => {
                useChainStore.getState().addPluginAsync({
                  unique_id: p.unique_id!,
                  name: p.name,
                  vendor: p.vendor,
                  format: p.format,
                })
              },
            }
          : undefined,
      })
    })
    .catch(console.error)
}

export function ChainCardActions({ plugin: p }: { plugin: ChainItem }) {
  const { t } = useTranslation()
  const titlePrefix = `ShallowHost → ${t('sidebar.plugins')}`
  return (
    <CardAction className="self-center">
      <div className="flex gap-1">
        <Tooltip>
          <TooltipTrigger
            render={(
              <Button
                variant="outline"
                size="icon"
                aria-label={t('home.openGui')}
                onClick={() => openGui(p.id, titlePrefix)}
                disabled={p.initializing}
              >
                <AppWindow className="size-4" />
              </Button>
            )}
          />
          <TooltipContent>{t('home.openGui')}</TooltipContent>
        </Tooltip>
        <ChainParamsButton pluginId={p.id} name={p.name} disabled={p.initializing} />
        <Tooltip>
          <TooltipTrigger
            render={(
              <Button
                variant="outline"
                size="icon"
                aria-label={t('home.bypass')}
                className={
                  p.bypassed
                    ? 'border-amber-600/30 bg-amber-600/15 text-amber-600 hover:bg-amber-600/25 dark:text-amber-500'
                    : ''
                }
                onClick={() => bypassPlugin(p.id, p.bypassed)}
                disabled={p.initializing}
              >
                {p.bypassed
                  ? (
                      <Circle className="size-4" />
                    )
                  : (
                      <Ban className="size-4" />
                    )}
              </Button>
            )}
          />
          <TooltipContent>{t('home.bypass')}</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger
            render={(
              <Button
                variant="outline"
                size="icon"
                aria-label={t('home.removeFromChain')}
                className="hover:bg-destructive/15 hover:text-destructive hover:border-destructive/30"
                onClick={() => removeFromChain(p)}
                disabled={p.initializing}
              >
                <Trash2 className="size-4" />
              </Button>
            )}
          />
          <TooltipContent>{t('home.removeFromChain')}</TooltipContent>
        </Tooltip>
      </div>
    </CardAction>
  )
}
