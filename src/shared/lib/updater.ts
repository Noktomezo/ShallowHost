import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'

export interface UpdateInfo {
  version: string
  date?: string
  body?: string
  releaseUrl?: string
}

export type DownloadStatus = 'downloading' | 'installing' | 'error'

export interface DownloadProgress {
  status: DownloadStatus
  percent: number
}

export interface UpdateService {
  check: () => Promise<UpdateInfo | null>
  downloadAndInstall: (onProgress: (p: DownloadProgress) => void) => Promise<void>
}

// ponytail: mock in DEV so UI is testable without a signed endpoint.
// Add real updater config (pubkey + endpoints) to tauri.conf.json plugins.updater
// before shipping; remove this flag or set USE_MOCK=false to test the real path.
const USE_MOCK = import.meta.env.DEV

function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms))
}

const mockService: UpdateService = {
  async check() {
    await delay(900)
    return {
      version: '0.2.0',
      date: new Date().toISOString(),
      releaseUrl: 'https://github.com/Noktomezo/ShallowHost/releases/tag/v0.2.0',
    }
  },
  async downloadAndInstall(onProgress) {
    let percent = 0
    while (percent < 100) {
      await delay(70)
      percent = Math.min(100, percent + Math.random() * 14 + 5)
      onProgress({ status: 'downloading', percent: Math.round(percent) })
    }
    onProgress({ status: 'installing', percent: 100 })
    await delay(700)
  },
}

const realService: UpdateService = {
  async check() {
    const update = await check()
    if (!update)
      return null
    return {
      version: update.version,
      date: update.date,
      body: update.body,
      releaseUrl: (update.rawJson.releaseUrl as string | undefined)
        ?? (update.rawJson.url as string | undefined),
    }
  },
  async downloadAndInstall(onProgress) {
    const update = await check()
    if (!update)
      throw new Error('No update available')
    let downloaded = 0
    let contentLength = 0
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength ?? 0
          break
        case 'Progress': {
          downloaded += event.data.chunkLength
          const percent = contentLength > 0
            ? Math.min(100, Math.round((downloaded / contentLength) * 100))
            : 0
          onProgress({ status: 'downloading', percent })
          break
        }
        case 'Finished':
          onProgress({ status: 'installing', percent: 100 })
          break
      }
    })
    await relaunch()
  },
}

export const updateService: UpdateService = USE_MOCK ? mockService : realService

export async function applyUpdateAndRelaunch(
  onProgress: (p: DownloadProgress) => void,
): Promise<void> {
  await updateService.downloadAndInstall(onProgress)
}
