import type { ScannedPlugin } from '@/shared/model/plugin-store'
import { invoke } from '@tauri-apps/api/core'
import { FolderOpen, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useChainStore } from '@/shared/model/chain-store'
import { usePluginStore } from '@/shared/model/plugin-store'
import { Badge } from '@/shared/ui/badge'
import { Button } from '@/shared/ui/button'
import { Card, CardAction, CardDescription, CardHeader, CardTitle } from '@/shared/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

export function PluginsPage() {
  const { t } = useTranslation()
  const plugins = usePluginStore(s => s.plugins)
  const setPlugins = usePluginStore(s => s.setPlugins)
  const removePlugin = usePluginStore(s => s.removePlugin)
  const chain = useChainStore(s => s.chain)
  const refreshChain = useChainStore(s => s.refresh)
  const setLoading = useChainStore(s => s.setLoading)
  const [scanning, setScanning] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    refreshChain()
  }, [refreshChain])

  async function scan() {
    setScanning(true)
    setError(null)
    try {
      const result = await invoke<ScannedPlugin[]>('scan_plugins')
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
      <div className="flex items-start justify-between gap-2">
        <div className="flex flex-col gap-0.5">
          <h1 className="text-xl font-semibold">{t('plugins.title')}</h1>
          <p className="text-sm text-muted-foreground">
            {t('plugins.description')}
          </p>
        </div>
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

      {error && (
        <p className="mt-2 text-sm text-destructive">{error}</p>
      )}

      <div className="mt-3 flex flex-1 flex-col gap-2">
        {plugins.length > 0
          ? (
              plugins.map((p) => {
                const inChain = chain.some(c => c.id === p.unique_id)
                return (
                  <Card key={p.unique_id} size="sm">
                    <CardHeader className="gap-0.5">
                      <div className="flex items-center gap-2">
                        <CardTitle>{p.name}</CardTitle>
                        <Badge variant="secondary" className="shrink-0">
                          {p.format.toUpperCase()}
                        </Badge>
                      </div>
                      {p.vendor && <CardDescription>{p.vendor}</CardDescription>}
                      <CardAction className="self-center">
                        <div className="flex gap-1">
                          <Tooltip>
                            <TooltipTrigger
                              render={(
                                <Button
                                  variant="outline"
                                  size="icon"
                                  disabled={inChain}
                                  onClick={() => addToChain(p.unique_id)}
                                >
                                  <Plus className="size-4" />
                                </Button>
                              )}
                            />
                            <TooltipContent>
                              {inChain ? t('plugins.alreadyInChain') : t('plugins.addToChain')}
                            </TooltipContent>
                          </Tooltip>
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
    </div>
  )
}
