import i18next from 'i18next'
import React from 'react'

interface Props {
  children: React.ReactNode
  fallback?: React.ReactNode
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends React.Component<Props, State> {
  public state: State = {
    hasError: false,
    error: null,
  }

  public static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  public componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('[ErrorBoundary] Uncaught error:', error, errorInfo)
  }

  public reset = () => {
    this.setState({ hasError: false, error: null })
  }

  public render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback
      }
      return (
        <div role="alert" className="flex h-screen w-screen flex-col items-center justify-center gap-4 bg-background p-6 text-foreground">
          <div className="flex flex-col items-center gap-2 text-center">
            <h1 className="text-xl font-bold">{i18next.t('common.errorTitle')}</h1>
            <p className="text-sm text-muted-foreground">
              {this.state.error?.message || i18next.t('common.unexpectedError')}
            </p>
          </div>
          <button
            type="button"
            aria-label={i18next.t('common.tryAgainAria')}
            className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 cursor-pointer"
            onClick={this.reset}
          >
            {i18next.t('common.tryAgain')}
          </button>
        </div>
      )
    }

    return this.props.children
  }
}
