import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { SearchIcon } from "lucide-react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

function App() {
  const [result, setResult] = useState<[{ name: string; path: string }] | null>(
    null,
  );

  async function updateQuery(e: React.ChangeEvent<HTMLInputElement>) {
    if (e.target.value.length > 0) {
      setResult(await invoke("get_files", { filter: e.target.value }));
      if (result !== null && result.length > 0) {
        getCurrentWindow().setSize(new LogicalSize(600, 400));
      }
    } else {
      setResult(null);
      getCurrentWindow().setSize(new LogicalSize(600, 50));
    }
  }
  useEffect(() => {
    getCurrentWindow().setSize(new LogicalSize(600, 50));
    getCurrentWindow().setEffects({ radius: 25, effects: [] });
    window.addEventListener("keydown", async (e) => {
      if (e.key === "Escape") {
        await invoke("hide_app");
      }
    });
  }, []);
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen flex flex-col items-center justify-start rounded-2xl bg-zinc-800 overflow-hidden"
    >
      <div className="w-full p-3 flex flex-row items-center gap-1 rounded-md bg-zinc-800 text-white">
        <SearchIcon />
        <input
          data-tauri-drag-region
          onChange={updateQuery}
          autoCapitalize="off"
          autoComplete="off"
          autoCorrect="off"
          spellCheck="false"
          placeholder="Fin"
          autoFocus
          className="w-full px-1 rounded-md text-white text-lg bg-transparent outline-none focus:outline-none"
        />
      </div>
      {result !== null && (
        <ul className="w-full h-full flex flex-col justify-start items-center gap-[0.1rem] overflow-y-scroll">
          {result?.map((item, index) => (
            <li key={index} className="w-full p-2 text-white">
              {item.name}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default App;
