import React from 'react'
import ReactDOM from 'react-dom/client'
import { AppToaster } from '@/shared/ui/sonner'
import { App } from './app/router'
import '@/shared/config/i18n'

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
    <AppToaster />
  </React.StrictMode>,
)
