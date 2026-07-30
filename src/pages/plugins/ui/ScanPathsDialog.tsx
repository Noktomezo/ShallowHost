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
  onReset: () => void
  setError: (err: string | null) => void
}

interface PathListSectionProps {
  title: string
  emptyText: string
  paths: string[]
  onAdd: (path: string) => void
  onRemove: (path: string) => void
  setError: (err: string | null) => void
}

function PathListSection({
  title,
  emptyText,
  paths,
  onAdd,
  onRemove,
  setError,
}: PathListSectionProps) {
  const { t } = useTranslation()
  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">{title}</span>
        <Button
          variant="outline"
          size="sm"
          onClick={async () => {
            try {
              const path = await invoke<string | null>('select_directory')
              if (path)
                onAdd(path)
            }
            catch (e) {
              setError(String(e))
            }
          }}
          className="h-8 gap-1"
        >
          <Plus className="size-3.5" data-icon="inline-start" />
          {t('plugins.addFolder')}
        </Button>
      </div>
      <div className="rounded-md border border-border bg-muted/20 p-2 flex flex-col gap-1.5 max-h-[140px] overflow-y-auto">
        {paths.length === 0
          ? (
              <p className="text-xs text-muted-foreground py-1 text-center">{emptyText}</p>
            )
          : (
              paths.map(p => (
                <div key={p} className="flex items-center justify-between gap-2 bg-muted/40 p-1.5 rounded text-xs select-text">
                  <span className="truncate flex-1 font-mono">{p}</span>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={t('plugins.removePath', { path: p })}
                    onClick={() => onRemove(p)}
                    className="size-6 text-muted-foreground hover:text-destructive hover:bg-destructive/10"
                  >
                    <Trash2 className="size-3.5" />
                  </Button>
                </div>
              ))
            )}
      </div>
    </div>
  )
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
  onReset,
  setError,
}: ScanPathsDialogProps) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogTitle>{t('plugins.scanPathsTitle')}</DialogTitle>
        <DialogDescription>{t('plugins.scanPathsDescription')}</DialogDescription>

        <div className="flex flex-col gap-4 py-2">
          <PathListSection
            title={t('plugins.vst2SearchPaths')}
            emptyText={t('plugins.noVst2Paths')}
            paths={vst2Paths}
            onAdd={addVst2Path}
            onRemove={removeVst2Path}
            setError={setError}
          />
          <PathListSection
            title={t('plugins.vst3SearchPaths')}
            emptyText={t('plugins.noVst3Paths')}
            paths={vst3Paths}
            onAdd={addVst3Path}
            onRemove={removeVst3Path}
            setError={setError}
          />
        </div>

        <div className="flex justify-between items-center pt-2 border-t border-border">
          <Button
            variant="ghost"
            onClick={onReset}
            className="text-sm text-muted-foreground hover:text-foreground gap-1"
          >
            <RotateCcw className="size-4" data-icon="inline-start" />
            {t('plugins.resetDefaults')}
          </Button>
          <Button onClick={() => onOpenChange(false)}>
            {t('plugins.done')}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
