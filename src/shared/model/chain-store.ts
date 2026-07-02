import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface ChainItem {
  id: string;
  name: string;
  vendor: string;
  format: string;
  bypassed: boolean;
}

interface ChainStore {
  chain: ChainItem[];
  setChain: (c: ChainItem[]) => void;
  refresh: () => Promise<void>;
  remove: (id: string) => void;
}

export const useChainStore = create<ChainStore>((set) => ({
  chain: [],
  setChain: (chain) => set({ chain }),
  refresh: async () => {
    try {
      const result = await invoke<ChainItem[]>("get_chain");
      set({ chain: result });
    } catch {
      // backend not ready
    }
  },
  remove: (id) =>
    set((s) => ({ chain: s.chain.filter((p) => p.id !== id) })),
}));

export type { ChainItem };
