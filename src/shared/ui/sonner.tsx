import { Toaster } from 'sonner'

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
