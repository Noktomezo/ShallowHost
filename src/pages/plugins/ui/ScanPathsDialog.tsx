import { invoke } from '@tauri-apps/api/core'
import { Plus, RotateCcw, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'
import { Dialog, DialogContent, DialogDescription, DialogTitle } from '@/shared/ui/dialog'

interface ScanPathsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  vst2Paths: string[]
  vst3Paths: string[]
  addVst2Path: (p: string) => void
  removeVst2Path: (p: string) => void
  addVst3Path: (p: string) => void
  removeVst3Path: (p: string) => void
  resetVst2Paths: () => void
  resetVst3Paths: () => void
  setError: (err: string | null) => void
}

export function ScanPathsDialog({
  open,
  onOpenChange,
  vst2Paths,
  vst3Paths,
  addVst2Path,
  removeVst2Path,
  addVst3Path,
  removeVst3Path,
  resetVst2Paths,
  resetVst3Paths,
  setError,
}: ScanPathsDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogTitle>{t('plugins.scanPathsTitle')}</DialogTitle>
        <DialogDescription>{t('plugins.scanPathsDescription')}</DialogDescription>

        <div className="flex flex-col gap-4 py-2">
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
                          aria-label={t('plugins.remove')}
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
                          aria-label={t('plugins.remove')}
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

        <div className="flex justify-between items-center pt-2 border-t border-border">
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              resetVst2Paths()
              resetVst3Paths()
            }}
            className="text-xs text-muted-foreground hover:text-foreground gap-1"
          >
            <RotateCcw className="size-3" data-icon="inline-start" />
            Reset to Defaults
          </Button>
          <Button size="sm" onClick={() => onOpenChange(false)}>
            Done
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
