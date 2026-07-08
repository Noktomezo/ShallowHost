import type { DownloadProgress, UpdateInfo } from '@/shared/lib/updater'
import { openUrl } from '@tauri-apps/plugin-opener'
import { Download, ExternalLink, X } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast, Toaster } from 'sonner'
import { applyUpdateAndRelaunch } from '@/shared/lib/updater'
import { cn } from '@/shared/lib/utils'
import { buttonVariants } from '@/shared/ui/button-variants'

export { toast }

const TOAST_ID = 'shallow-update'

// ponytail: track toast visibility so the poller (router.tsx) can skip re-showing
// while it's on screen, but re-show after the user dismisses it.
let toastVisible = false
export function isUpdateToastVisible() {
  return toastVisible
}

// ponytail: theme follows the app via CSS variables (bg-card/text-card-foreground/border-border
// all switch with the .dark class). No need to drive sonner's theme prop.
export function AppToaster() {
  return (
    <Toaster
      position="bottom-right"
      duration={4000}
      toastOptions={{
        classNames: {
          toast:
            'bg-card text-card-foreground border border-border rounded-lg shadow-md',
          title: 'text-sm font-medium text-foreground',
          description: 'text-xs text-muted-foreground',
          actionButton:
            'bg-primary text-primary-foreground hover:bg-primary-hover rounded-lg',
          cancelButton:
            'bg-transparent text-muted-foreground hover:bg-accent rounded-lg',
        },
      }}
    />
  )
}

type UpdateState
  = | { kind: 'available' }
    | { kind: 'progress', progress: DownloadProgress }
    | { kind: 'error', message: string }

function UpdateButton({
  variant = 'default',
  size = 'sm',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: 'default' | 'outline' | 'ghost' | 'secondary' | 'destructive'
  size?: 'default' | 'sm' | 'xs' | 'lg'
}) {
  return (
    <button
      type="button"
      data-slot="button"
      className={cn(buttonVariants({ variant, size }), 'cursor-pointer', className)}
      {...props}
    />
  )
}

function UpdateToastView({ info }: { info: UpdateInfo }) {
  const { t } = useTranslation()
  const [state, setState] = useState<UpdateState>({ kind: 'available' })

  const dismiss = () => toast.dismiss(TOAST_ID)

  async function handleUpdate() {
    setState({
      kind: 'progress',
      progress: { status: 'downloading', percent: 0 },
    })
    try {
      await applyUpdateAndRelaunch((p) => {
        setState({ kind: 'progress', progress: p })
      })
      // ponytail: real path relaunches inside downloadAndInstall (app exits, line unreached).
      // Mock resolves without relaunch — dismiss the toast silently.
      dismiss()
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
            <UpdateButton variant="outline" onClick={dismiss}>
              {t('update.close')}
            </UpdateButton>
          </div>
        )}
      </div>
    )
  }

  return (
    <div className="flex w-80 flex-col gap-2 p-4">
      <div className="text-sm font-medium text-foreground">
        {t('update.available')}
        {' '}
        v
        {info.version}
      </div>
      {info.releaseUrl && (
        <button
          type="button"
          onClick={() => openUrl(info.releaseUrl!)}
          className="inline-flex w-fit items-center gap-1 text-xs text-primary hover:underline"
        >
          {t('update.viewRelease')}
          <ExternalLink className="size-3" />
        </button>
      )}
      <div className="flex gap-2 pt-1">
        <UpdateButton variant="default" className="flex-1" onClick={handleUpdate}>
          <Download className="size-3.5" />
          {t('update.update')}
        </UpdateButton>
        <UpdateButton variant="outline" onClick={dismiss}>
          <X className="size-3.5" />
          {t('update.notNow')}
        </UpdateButton>
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
