import React from 'react'
import ReactDOM from 'react-dom/client'
import { ThemeProvider } from '@/shared/providers/theme-provider'
import { ErrorBoundary } from '@/shared/ui/error-boundary'
import { AppToaster } from '@/shared/ui/sonner'
import { App } from './app/router'
import '@/shared/config/i18n'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <ThemeProvider defaultTheme="dark">
        <App />
        <AppToaster />
      </ThemeProvider>
    </ErrorBoundary>
  </React.StrictMode>,
)
