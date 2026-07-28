import type { DownloadProgress, UpdateInfo } from '@/shared/lib/updater'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Download, ExternalLink, X } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { applyUpdateAndRelaunch } from '@/shared/lib/updater'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'

const TOAST_ID = 'shallow-update'
let toastVisible = false

export function isUpdateToastVisible() {
  return toastVisible
}

export function dismissUpdateToast() {
  toastVisible = false
  toast.dismiss(TOAST_ID)
}

type UpdateState
  = | { kind: 'available' }
    | { kind: 'progress', progress: DownloadProgress }
    | { kind: 'error', message: string }

function UpdateToastView({ info }: { info: UpdateInfo }) {
  const { t } = useTranslation()
  const [state, setState] = useState<UpdateState>({ kind: 'available' })

  async function handleUpdate() {
    setState({
      kind: 'progress',
      progress: { status: 'downloading', percent: 0 },
    })
    try {
      await applyUpdateAndRelaunch((p) => {
        setState({ kind: 'progress', progress: p })
      })
      dismissUpdateToast()
    }
    catch (e) {
      setState({ kind: 'error', message: String(e) })
    }
  }

  if (state.kind !== 'available') {
    const percent = state.kind === 'progress' ? state.progress.percent : 100
    const isError = state.kind === 'error'
    const label = isError
      ? t('update.failed')
      : state.progress.status === 'installing'
        ? t('update.installing')
        : t('update.downloading')
    return (
      <div className="flex w-80 flex-col gap-2 p-4">
        <div className="flex items-center justify-between gap-2">
          <div className="text-sm font-medium text-foreground">{label}</div>
          <div className="text-xs text-muted-foreground tabular-nums">
            {percent}
            %
          </div>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
          <div
            className={cn(
              'h-full rounded-full transition-all duration-150',
              isError ? 'bg-destructive' : 'bg-primary',
            )}
            style={{ width: `${percent}%` }}
          />
        </div>
        {isError && (
          <div className="flex justify-start gap-2 pt-7">
            <Button variant="outline" size="sm" onClick={dismissUpdateToast}>
              {t('update.close')}
            </Button>
          </div>
        )}
      </div>
    )
  }

  const releaseUrl = info.releaseUrl || `https://github.com/Noktomezo/ShallowHost/releases/tag/v${info.version}`

  return (
    <div className="flex w-80 flex-col gap-2 p-4">
      <div className="text-sm font-medium text-foreground">
        {t('update.available')}
        {' '}
        v
        {info.version}
      </div>
      <button
        type="button"
        onClick={() => {
          openUrl(releaseUrl).catch(console.error)
        }}
        className="inline-flex w-fit items-center gap-1 text-xs text-primary hover:underline cursor-pointer"
      >
        {t('update.viewRelease')}
        <ExternalLink className="size-3" />
      </button>
      <div className="flex gap-2 pt-1">
        <Button variant="default" size="sm" className="flex-1 cursor-pointer" onClick={handleUpdate}>
          <Download className="size-3.5" />
          {t('update.update')}
        </Button>
        <Button variant="outline" size="sm" className="cursor-pointer" onClick={dismissUpdateToast}>
          <X className="size-3.5" />
          {t('update.notNow')}
        </Button>
      </div>
    </div>
  )
}

export function showUpdateToast(info: UpdateInfo) {
  toastVisible = true
  toast.custom(() => <UpdateToastView info={info} />, {
    id: TOAST_ID,
    duration: Infinity,
    dismissible: true,
    onDismiss: () => { toastVisible = false },
  })
}
