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

export const updateService = {
  async check() {
    // ponytail: tauri updater has no built-in timeout — flaky network hangs
    // the check button forever. 10s ceiling; upgrade to configurable if needed.
    const update = await Promise.race([
      check(),
      new Promise<null>(resolve => setTimeout(resolve, 10000, null)),
    ])
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
  async downloadAndInstall(onProgress: (p: DownloadProgress) => void) {
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

export async function applyUpdateAndRelaunch(
  onProgress: (p: DownloadProgress) => void,
): Promise<void> {
  await updateService.downloadAndInstall(onProgress)
}
