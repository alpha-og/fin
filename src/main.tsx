import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
getCurrentWindow().setSize(new LogicalSize(600, 200));
getCurrentWindow().setEffects({ radius: 25, effects: [] });
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
