import type { ScannedPlugin } from '@/shared/model/plugin-store'
import { invoke } from '@tauri-apps/api/core'
import { RefreshCw, Settings } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useChainStore } from '@/shared/model/chain-store'
import { usePluginStore } from '@/shared/model/plugin-store'
import { Button } from '@/shared/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import { PluginItemCard } from './PluginItemCard'
import { ScanPathsDialog } from './ScanPathsDialog'

export function PluginsPage() {
  const { t } = useTranslation()
  const plugins = usePluginStore(s => s.plugins)
  const setPlugins = usePluginStore(s => s.setPlugins)
  const removePlugin = usePluginStore(s => s.removePlugin)

  const {
    vst2Paths,
    vst3Paths,
    addVst2Path,
    removeVst2Path,
    addVst3Path,
    removeVst3Path,
    resetPaths,
  } = usePluginStore()

  const chain = useChainStore(s => s.chain)
  const chainError = useChainStore(s => s.error)
  const refreshChain = useChainStore(s => s.refresh)
  const addPluginAsync = useChainStore(s => s.addPluginAsync)
  const checkInitializing = useChainStore(s => s.isInitializing)
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [settingsOpen, setSettingsOpen] = useState(false)

  const displayError = error || chainError

  useEffect(() => {
    refreshChain()
  }, [refreshChain])

  async function scan() {
    setScanning(true)
    setError(null)
    useChainStore.getState().setError(null)
    try {
      const result = await invoke<ScannedPlugin[]>('scan_plugins', {
        vst2Paths,
        vst3Paths,
      })
      setPlugins(result)
    }
    catch (e) {
      setError(String(e))
    }
    finally {
      setScanning(false)
    }
  }

  async function revealPlugin(path: string) {
    try {
      await invoke('reveal_plugin', { path })
    }
    catch (e) {
      setError(String(e))
    }
  }

  return (
    <div className="flex flex-1 flex-col gap-0.5">
      <div className="flex items-center justify-between gap-2">
        <div className="flex flex-col gap-0.5">
          <h1 className="text-xl font-semibold">{t('plugins.title')}</h1>
          <p className="text-sm text-muted-foreground">
            {t('plugins.description')}
          </p>
        </div>
        <div className="flex gap-2">
          <Tooltip>
            <TooltipTrigger
              render={(
                <Button
                  variant="outline"
                  size="icon"
                  aria-label={t('plugins.scanPathsTitle')}
                  onClick={() => setSettingsOpen(true)}
                  disabled={scanning}
                >
                  <Settings className="size-4" />
                </Button>
              )}
            />
            <TooltipContent>{t('plugins.scanPathsTitle')}</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger
              render={(
                <Button
                  variant="default"
                  size="icon"
                  aria-label={t('plugins.scan')}
                  onClick={scan}
                  disabled={scanning}
                >
                  <RefreshCw className={scanning ? 'size-4 animate-spin' : 'size-4'} />
                </Button>
              )}
            />
            <TooltipContent>{t('plugins.scan')}</TooltipContent>
          </Tooltip>
        </div>
      </div>

      {displayError && (
        <p className="mt-2 text-sm text-destructive">{displayError}</p>
      )}

      <div className="mt-3 flex flex-1 flex-col gap-2">
        {plugins.length > 0
          ? (
              plugins.map((p) => {
                const isInitializing = checkInitializing(p.unique_id)
                const inChain = chain.some(c => c.unique_id === p.unique_id)
                return (
                  <PluginItemCard
                    key={p.unique_id}
                    plugin={p}
                    inChain={inChain}
                    isInitializing={isInitializing}
                    onAdd={addPluginAsync}
                    onReveal={revealPlugin}
                    onRemove={removePlugin}
                  />
                )
              })
            )
          : (
              !scanning && (
                <div className="flex flex-1 items-center justify-center">
                  <p className="text-sm text-muted-foreground">
                    {t('plugins.empty')}
                  </p>
                </div>
              )
            )}
      </div>

      <ScanPathsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        vst2Paths={vst2Paths}
        vst3Paths={vst3Paths}
        addVst2Path={addVst2Path}
        removeVst2Path={removeVst2Path}
        addVst3Path={addVst3Path}
        removeVst3Path={removeVst3Path}
        onReset={resetPaths}
        setError={setError}
      />
    </div>
  )
}
