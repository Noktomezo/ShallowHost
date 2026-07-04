import { invoke } from '@tauri-apps/api/core'
import { SlidersHorizontal } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
} from '@/shared/ui/dialog'
import { ScrollArea } from '@/shared/ui/scroll-area'
import { Slider } from '@/shared/ui/slider'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

interface ParamInfo {
  index: number
  name: string
  unit: string
  min: number
  max: number
  default: number
  step_count: number
  value: number
}

export function ChainParamsButton({ pluginId, name }: { pluginId: string, name: string }) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [params, setParams] = useState<ParamInfo[]>([])

  async function loadParams() {
    setParams([])
    try {
      const result = await invoke<ParamInfo[]>('get_plugin_parameters', {
        pluginId,
      })
      setParams(result)
    }
    catch (e) {
      console.error(e)
    }
  }

  async function handleParamChange(index: number, value: number) {
    setParams(prev =>
      prev.map(p => (p.index === index ? { ...p, value } : p)),
    )
    try {
      await invoke('set_plugin_parameter', {
        pluginId,
        paramIndex: index,
        value,
      })
    }
    catch (e) {
      console.error(e)
    }
  }

  return (
    <>
      <Tooltip>
        <TooltipTrigger
          render={(
            <Button
              variant="outline"
              size="icon"
              aria-label={t('home.parameters')}
              onClick={() => {
                setOpen(true)
                loadParams()
              }}
            >
              <SlidersHorizontal className="size-4" />
            </Button>
          )}
        />
        <TooltipContent>{t('home.parameters')}</TooltipContent>
      </Tooltip>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="sm:max-w-lg">
          <DialogTitle>{name}</DialogTitle>
          <DialogDescription>{t('home.parameters')}</DialogDescription>
          <ScrollArea className="max-h-[60vh]">
            <div className="flex flex-col gap-3 pr-3">
              {params.length === 0
                ? (
                    <p className="text-sm text-muted-foreground">
                      {t('home.noParameters')}
                    </p>
                  )
                : (
                    params.map((p) => {
                      const step
                        = p.step_count > 0
                          ? (p.max - p.min) / p.step_count
                          : (p.max - p.min) / 1000
                      return (
                        <div key={p.index} className="flex flex-col gap-1">
                          <div className="flex justify-between text-sm">
                            <span>{p.name}</span>
                            <span className="text-muted-foreground tabular-nums">
                              {p.value.toFixed(2)}
                              {p.unit && ` ${p.unit}`}
                            </span>
                          </div>
                          <Slider
                            value={[p.value]}
                            min={p.min}
                            max={p.max}
                            step={step}
                            onValueChange={(v) => {
                              const val = Array.isArray(v) ? v[0] : v
                              handleParamChange(p.index, val)
                            }}
                          />
                        </div>
                      )
                    })
                  )}
            </div>
          </ScrollArea>
        </DialogContent>
      </Dialog>
    </>
  )
}
