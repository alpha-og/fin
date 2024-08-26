import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { SearchIcon } from "lucide-react";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useHotkeys } from "react-hotkeys-hook";

function App() {
  const [result, setResult] = useState<[{ name: string; path: string }] | null>(
    null,
  );
  const [selected, setSelected] = useState<number | null>(null);
  const currentWindow = getCurrentWindow();
  const clearSearchRef = useHotkeys<HTMLInputElement>(
    "escape, ctrl+[",
    (e) => {
      const target = e.target as HTMLInputElement;
      if (target.value === "") {
        currentWindow.hide();
      }
      target.value = "";
      setResult(null);
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );
  useHotkeys(
    "ArrowDown, ctrl+j",
    () => {
      if (result !== null) {
        if (selected === null) {
          setSelected(0);
        } else {
          setSelected((selected + 1) % result.length);
        }
      }
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );

  useHotkeys(
    "ArrowUp, ctrl+k",
    () => {
      if (result !== null) {
        if (selected === null) {
          setSelected(result.length - 1);
        } else {
          setSelected((selected + result.length - 1) % result.length);
        }
      }
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );

  const inputRef = useRef<HTMLInputElement>(null);
  async function updateQuery(e: React.ChangeEvent<HTMLInputElement>) {
    if (e.target.value.length > 0)
      setResult(await invoke("get_files", { filter: e.target.value }));
    else setResult(null);
  }

  useEffect(() => {
    currentWindow.setSize(new LogicalSize(600, 50));
    currentWindow.setEffects({ radius: 25, effects: [] });
  }, []);
  useEffect(() => {
    clearSearchRef.current = inputRef.current;
  }, [inputRef]);
  useEffect(() => {
    if (result !== null && result.length > 0) {
      currentWindow.setSize(new LogicalSize(600, 400));
      setSelected(null);
    } else {
      currentWindow.setSize(new LogicalSize(600, 50));
    }
  }, [result]);
  useEffect(() => {
    if (selected !== null) {
      const element = document.querySelectorAll("li")[selected];
      element.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [selected]);
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen flex flex-col items-center justify-start rounded-2xl bg-zinc-800 overflow-hidden"
    >
      <div
        data-tauri-drag-region
        className="w-full p-3 flex flex-row items-center gap-1 rounded-md bg-zinc-800 text-white"
      >
        <SearchIcon />
        <input
          ref={inputRef}
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
        <ul className="w-full h-full p-2 flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar">
          {result?.map((item, index) => (
            <li
              key={index}
              className={`w-full p-2 text-white rounded-xl text-ellipsis overflow-x-clip ${index === selected && "bg-blue-400/40"}`}
              onClick={() => {
                setSelected(index);
              }}
            >
              {item.name}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default App;
