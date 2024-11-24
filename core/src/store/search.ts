import { create } from "zustand";

export type T_Result = {
  title: string;
  description: string | null;
  icon: string | null;
  action: any;
  priority: number;
};

export type T_SearchStore = {
  query: string;
  results: T_Result[];
  history: string[];
  selected: number | null;
  selectedHistory: number | null;
  setQuery: (query: string) => void;
  setResults: (results: T_Result[]) => void;
  setHistory: (history: string[]) => void;
  setSelected: (selected: number | null) => void;
  setSelectedHistory: (selectedHistory: number | null) => void;
};

export const useSearchStore = create<T_SearchStore>((set) => ({
  query: "",
  results: [],
  history: [],
  selected: null,
  selectedHistory: null,
  setQuery: (query: string) => set({ query }),
  setResults: (results: T_Result[]) => set({ results }),
  setHistory: (history: string[]) => set({ history }),
  setSelected: (selected: number | null) => set({ selected }),
  setSelectedHistory: (selectedHistory: number | null) =>
    set({ selectedHistory }),
}));
