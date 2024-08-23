import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [result, setResult] = useState<String[]>([]);

  async function updateQuery(e: React.ChangeEvent<HTMLInputElement>) {
    setResult(await invoke("get_dirs", { query: e.currentTarget.value }));
  }
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen-lg bg-zinc-800 flex flex-col items-center justify-center rounded-lg overflow-y-scroll"
    >
      <div className="w-2/3 flex flex-col items-center justify-center">
        <input
          onChange={updateQuery}
          placeholder="Query..."
          className="w-full mt-5 p-2 border-2 border-gray-300 rounded-md"
        />
        <ul className="w-full h-96 flex flex-col justify-between items-center overflow-y-scroll">
          {result.map((item, index) => (
            <li
              key={index}
              className="w-1/2 p-2 border-2 border-gray-300 rounded-md"
            >
              {item}
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

export default App;
