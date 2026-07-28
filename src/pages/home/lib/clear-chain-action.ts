import { invoke } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import { useChainStore } from '@/shared/model/chain-store'

export async function clearChainWithUndo(t: (key: string, options?: any) => string) {
  const previousChain = [...useChainStore.getState().chain]
  if (previousChain.length === 0)
    return

  const results = await Promise.allSettled(
    previousChain.map(p => invoke('remove_from_chain', { pluginId: p.id })),
  )
  await useChainStore.getState().refresh()

  const failedCount = results.filter(r => r.status === 'rejected').length
  if (failedCount > 0) {
    toast.error(t('home.clearChainError'))
  }

  const removedCount = previousChain.length - failedCount
  if (removedCount > 0) {
    toast(t('home.chainCleared'), {
      description: `${removedCount} ${t('home.pluginsRemoved', { count: removedCount })}`,
      action: {
        label: t('home.undo'),
        onClick: async () => {
          for (const item of previousChain) {
            if (item.unique_id) {
              try {
                const restored = await useChainStore.getState().addPluginAsync({
                  unique_id: item.unique_id,
                  name: item.name,
                  vendor: item.vendor,
                  format: item.format,
                })
                if (restored && item.bypassed && !restored.bypassed) {
                  await invoke('bypass_plugin', { pluginId: restored.id, bypassed: true })
                  useChainStore.getState().updateChainItem(restored.id, { bypassed: true })
                }
              }
              catch (e) {
                console.error(e)
              }
            }
          }
        },
      },
    })
  }
}
