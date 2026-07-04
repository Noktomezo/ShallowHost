import type { StateStorage } from 'zustand/middleware'
import { LazyStore } from '@tauri-apps/plugin-store'

const store = new LazyStore('settings.json')

function debounce<T extends (...args: never[]) => void>(fn: T, ms: number): T {
  let timer: ReturnType<typeof setTimeout> | null = null
  return ((...args: never[]) => {
    if (timer)
      clearTimeout(timer)
    timer = setTimeout(fn, ms, ...args)
  }) as T
}

const debouncedSet = debounce(async (key: string, value: string) => {
  await store.set(key, value)
}, 100)

export const tauriStorage: StateStorage = {
  getItem: async (key) => {
    const value = await store.get<string>(key)
    return value ?? null
  },
  setItem: async (key, value) => {
    await debouncedSet(key, value)
  },
  removeItem: async (key) => {
    await store.delete(key)
  },
}
