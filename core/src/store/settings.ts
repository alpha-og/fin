import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export type T_Metadata = {
  name: string;
  description: string;
  icon: string;
  url: string;
};

export type T_Plugin = {
  metadata: T_Metadata;
  config: Record<string, string>;
};

export type T_SettingsStore = {
  query: string;
  plugins: Record<string, T_Plugin>;
  selectedPlugin: string | null;
  setPlugins: (plugins: Record<string, T_Plugin>) => void;
  refreshPlugins: () => void;
  updatePlugin: (plugin: T_Plugin) => void;
  setSelectedPlugin: (selectedPlugin: string | null) => void;
  getSelectedPlugin: () => T_Plugin | null;
  setQuery: (query: string) => void;
};

export const useSettingsStore = create<T_SettingsStore>((set, get) => ({
  query: "",
  plugins: {},
  selectedPlugin: null,

  setPlugins: (plugins: Record<string, T_Plugin>) => set({ plugins }),
  refreshPlugins: () => {
    invoke<Record<string, T_Plugin>>("get_plugins").then((plugins) => {
      set({ plugins });
    });
  },
  updatePlugin: (plugin: T_Plugin) =>
    set((state) => ({
      plugins: {
        ...state.plugins,
        [plugin.metadata.name]: plugin,
      },
    })),
  setSelectedPlugin: (selectedPlugin: string | null) => set({ selectedPlugin }),
  getSelectedPlugin: () => {
    const { selectedPlugin, plugins } = get();
    if (selectedPlugin === null) {
      return null;
    }
    return plugins[selectedPlugin];
  },
  setQuery: (query: string) => set({ query }),
}));
