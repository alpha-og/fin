import { create } from "zustand";

export type T_Page = "search" | "settings" | "about";

export type T_App = {
  currentPage: T_Page;
  setCurrentPage: (page: T_Page) => void;
};

export const useApp = create<T_App>((set) => ({
  currentPage: "search",
  setCurrentPage: (page: T_Page) => set({ currentPage: page }),
}));
