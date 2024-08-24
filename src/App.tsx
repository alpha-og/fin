import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

function App() {
  const [result, setResult] = useState<[{ name: string; path: string }] | null>(
    null,
  );

  async function updateQuery(e: React.ChangeEvent<HTMLInputElement>) {
    setResult(await invoke("get_files", { filter: e.target.value }));
    if (result !== null && result.length > 0) {
      getCurrentWindow().setSize(new LogicalSize(600, 400));
    }
  }
  useEffect(() => {}, []);
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen flex flex-col items-center justify-start rounded-2xl bg-zinc-800 overflow-hidden"
    >
      <input
        onChange={updateQuery}
        autoCapitalize="off"
        autoComplete="off"
        autoCorrect="off"
        spellCheck="false"
        placeholder="Query..."
        className="w-full p-2 rounded-md bg-zinc-800 text-white outline-none focus:outline-none"
      />

      <ul className="w-full h-full flex flex-col justify-start items-center gap-[0.1rem] overflow-y-scroll">
        {result?.map((item, index) => (
          <li key={index} className="w-full p-2 text-white">
            {item.name}
          </li>
        ))}
      </ul>
    </div>
  );
}

export default App;
