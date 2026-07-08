import type { ScannedPlugin } from '@/shared/model/plugin-store'
import { Link } from '@tanstack/react-router'
import { invoke } from '@tauri-apps/api/core'
import { ArrowRight, FolderOpen, Plus, RefreshCw, RotateCcw, Settings, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useChainStore } from '@/shared/model/chain-store'
import { usePluginStore } from '@/shared/model/plugin-store'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/shared/ui/dialog'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

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
  const refreshChain = useChainStore(s => s.refresh)
  const setLoading = useChainStore(s => s.setLoading)
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [settingsOpen, setSettingsOpen] = useState(false)

  useEffect(() => {
    refreshChain()
  }, [refreshChain])

  async function scan() {
    setScanning(true)
    setError(null)
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

  async function addToChain(pluginId: string) {
    setLoading(true, t('plugins.loading'))
    try {
      await invoke('add_to_chain', { pluginId })
      await refreshChain()
    }
    catch (e) {
      setError(String(e))
    }
    finally {
      setLoading(false)
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

      {error && (
        <p className="mt-2 text-sm text-destructive">{error}</p>
      )}

      <div className="mt-3 flex flex-1 flex-col gap-2">
        {plugins.length > 0
          ? (
              plugins.map((p) => {
                const inChain = chain.some(c => c.unique_id === p.unique_id || c.name === p.name)
                return (
                  <Card key={p.unique_id} size="sm">
                    <CardHeader className="gap-0.5">
                      <div className="flex items-center gap-2">
                        <CardTitle>{p.name}</CardTitle>
                        <Badge variant="purple" className="shrink-0">
                          {p.format.toUpperCase()}
                        </Badge>
                        {inChain && (
                          <Badge variant="green" className="shrink-0">
                            {t('plugins.inChain')}
                          </Badge>
                        )}
                      </div>
                      {p.vendor && <CardDescription>{p.vendor}</CardDescription>}
                      <CardAction className="self-center">
                        <div className="flex gap-1">
                          {inChain
                            ? (
                                <Tooltip>
                                  <TooltipTrigger
                                    render={(
                                      <Link to="/">
                                        <Button
                                          variant="default"
                                          size="icon"
                                          aria-label={t('plugins.goToChain')}
                                        >
                                          <ArrowRight className="size-4" />
                                        </Button>
                                      </Link>
                                    )}
                                  />
                                  <TooltipContent>{t('plugins.goToChain')}</TooltipContent>
                                </Tooltip>
                              )
                            : (
                                <Tooltip>
                                  <TooltipTrigger
                                    render={(
                                      <Button
                                        variant="outline"
                                        size="icon"
                                        onClick={() => addToChain(p.unique_id)}
                                      >
                                        <Plus className="size-4" />
                                      </Button>
                                    )}
                                  />
                                  <TooltipContent>{t('plugins.addToChain')}</TooltipContent>
                                </Tooltip>
                              )}
                          <Tooltip>
                            <TooltipTrigger
                              render={(
                                <Button
                                  variant="outline"
                                  size="icon"
                                  onClick={() => revealPlugin(p.path)}
                                >
                                  <FolderOpen className="size-4" />
                                </Button>
                              )}
                            />
                            <TooltipContent>{t('plugins.reveal')}</TooltipContent>
                          </Tooltip>
                          <Tooltip>
                            <TooltipTrigger
                              render={(
                                <Button
                                  variant="outline"
                                  size="icon"
                                  className="hover:bg-destructive/15 hover:text-destructive"
                                  onClick={() => removePlugin(p.unique_id)}
                                >
                                  <Trash2 className="size-4" />
                                </Button>
                              )}
                            />
                            <TooltipContent>{t('plugins.remove')}</TooltipContent>
                          </Tooltip>
                        </div>
                      </CardAction>
                    </CardHeader>
                  </Card>
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

      <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
        <DialogContent className="sm:max-w-xl">
          <DialogTitle>{t('plugins.scanPathsTitle')}</DialogTitle>
          <DialogDescription>
            {t('plugins.scanPathsDescription')}
          </DialogDescription>

          <div className="flex flex-col gap-4 py-4">
            {/* VST2 Section */}
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold">VST2 Search Paths</span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={async () => {
                    try {
                      const path = await invoke<string | null>('select_directory')
                      if (path)
                        addVst2Path(path)
                    }
                    catch (e) {
                      setError(String(e))
                    }
                  }}
                  className="h-8 gap-1"
                >
                  <Plus className="size-3.5" />
                  {t('plugins.addFolder')}
                </Button>
              </div>
              <div className="rounded-md border border-border bg-muted/20 p-2 flex flex-col gap-1.5 max-h-[140px] overflow-y-auto">
                {vst2Paths.length === 0
                  ? (
                      <p className="text-xs text-muted-foreground py-1 text-center">No VST2 paths configured</p>
                    )
                  : (
                      vst2Paths.map(p => (
                        <div key={p} className="flex items-center justify-between gap-2 bg-muted/40 p-1.5 rounded text-xs select-text">
                          <span className="truncate flex-1 font-mono">{p}</span>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => removeVst2Path(p)}
                            className="size-6 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                        </div>
                      ))
                    )}
              </div>
            </div>

            {/* VST3 Section */}
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold">VST3 Search Paths</span>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={async () => {
                    try {
                      const path = await invoke<string | null>('select_directory')
                      if (path)
                        addVst3Path(path)
                    }
                    catch (e) {
                      setError(String(e))
                    }
                  }}
                  className="h-8 gap-1"
                >
                  <Plus className="size-3.5" />
                  {t('plugins.addFolder')}
                </Button>
              </div>
              <div className="rounded-md border border-border bg-muted/20 p-2 flex flex-col gap-1.5 max-h-[140px] overflow-y-auto">
                {vst3Paths.length === 0
                  ? (
                      <p className="text-xs text-muted-foreground py-1 text-center">No VST3 paths configured</p>
                    )
                  : (
                      vst3Paths.map(p => (
                        <div key={p} className="flex items-center justify-between gap-2 bg-muted/40 p-1.5 rounded text-xs select-text">
                          <span className="truncate flex-1 font-mono">{p}</span>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => removeVst3Path(p)}
                            className="size-6 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                          >
                            <Trash2 className="size-3.5" />
                          </Button>
                        </div>
                      ))
                    )}
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between gap-2 mt-2">
            <Button
              variant="outline"
              size="sm"
              onClick={resetPaths}
              className="gap-1.5 text-muted-foreground hover:text-foreground"
            >
              <RotateCcw className="size-3.5" />
              {t('plugins.resetDefaults')}
            </Button>
            <Button
              variant="default"
              size="sm"
              onClick={() => setSettingsOpen(false)}
            >
              {t('titlebar.close')}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
