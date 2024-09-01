import { useEffect, useState, useRef, createRef, RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { Command } from "@tauri-apps/plugin-shell";
import "./App.css";
import { SearchIcon, Folder, File, FileQuestion } from "lucide-react";
import { useHotkeys, isHotkeyPressed } from "react-hotkeys-hook";

function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const listItemRefs = useRef<RefObject<HTMLLIElement>[]>([]);
  const [result, setResult] = useState<
    | [
        {
          name: string;
          path: string;
          kind: string;
        },
      ]
    | null
  >(null);
  const [query, setQuery] = useState<string>("");
  const [history, setHistory] = useState<string[]>([]);
  const [selectedHistory, setSelectedHistory] = useState<number | null>(null);
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
  const cycleHistoryRef = useHotkeys(
    "ctrl+p, ctrl+n",
    () => {
      if (history.length === 0) return;
      if (isHotkeyPressed("p")) {
        if (selectedHistory === null) {
          setSelectedHistory(history.length - 1);
        } else
          setSelectedHistory(
            (selectedHistory + history.length - 1) % history.length,
          );
      } else {
        if (selectedHistory === null) return;
        else setSelectedHistory((selectedHistory + 1) % history.length);
      }
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );

  const selectedListItemRef = useHotkeys("escape, ctrl+[", () => {
    inputRef.current?.focus();
    setSelected(null);
  });

  useHotkeys(
    "ArrowDown, ctrl+j",
    () => {
      if (result !== null) {
        if (selected === null) {
          setHistory([...history, query]);
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
          setHistory([...history, query]);
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
  const openFileRef = useHotkeys("enter", (e) => {
    const target = e.target as HTMLLIElement;
    const pathElement = target.children[1].children[1] as HTMLSpanElement;
    const path = pathElement.innerText;
    Command.create("exec-sh", ["-c", `open -R ${path}`])
      .execute()
      .then(() => {});
  });

  async function updateResults() {
    if (query.length > 0)
      setResult(await invoke("get_files", { filter: query }));
    else setResult(null);
  }

  useEffect(() => {
    clearSearchRef.current = inputRef.current;
    cycleHistoryRef.current = inputRef.current;
  }, [inputRef]);
  useEffect(() => {
    if (result !== null && result.length > 0) {
      currentWindow.setSize(new LogicalSize(600, 400));
      listItemRefs.current = Array(result.length)
        .fill(null)
        .map(() => createRef<HTMLLIElement>());
      setSelected(null);
    } else {
      currentWindow.setSize(new LogicalSize(600, 50));
    }
  }, [result]);
  useEffect(() => {
    if (selected !== null) {
      selectedListItemRef.current = listItemRefs.current[selected].current;
      openFileRef.current = selectedListItemRef.current;
      const listItem = listItemRefs.current[selected];
      listItem?.current?.focus();
      listItem?.current?.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, [selected]);
  useEffect(() => {
    updateResults();
  }, [query]);
  useEffect(() => {
    if (selectedHistory !== null) {
      setQuery(history[selectedHistory]);
    }
  }, [selectedHistory]);
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen p-1 flex flex-col items-center justify-start rounded-2xl bg-zinc-800 overflow-hidden"
    >
      <div
        data-tauri-drag-region
        className="w-full px-3 py-2 flex flex-row items-center gap-1 rounded-md bg-zinc-800 text-white"
      >
        <SearchIcon />
        <input
          ref={inputRef}
          tabIndex={0}
          onChange={(e) => setQuery(e.target.value)}
          value={query}
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
        <ul
          tabIndex={1}
          className="w-full h-full p-1 flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none"
        >
          {result?.map((item, index) => (
            <li
              ref={listItemRefs.current[index]}
              tabIndex={index + 2}
              key={index}
              className={`w-full p-2 flex flex-row justify-between items-center gap-4 text-white rounded-xl outline-none focus:outline-none ${index === selected && "bg-white/20"}`}
              onClick={() => {
                setHistory([...history, query]);
                setSelected(index);
              }}
            >
              <span>
                {item.kind === "File" ? (
                  <File size={28} className="shrink-0" />
                ) : item.kind === "Directory" ? (
                  <Folder size={28} className="shrink-0" />
                ) : (
                  <FileQuestion size={28} className="shrink-0" />
                )}
              </span>
              <div className="w-11/12 flex flex-col justify-evenly items-start">
                <span className="w-full">
                  <p className="truncate">{item.name}</p>
                </span>{" "}
                <span className="w-full">
                  <p className="w-full truncate text-neutral-400">
                    {item.path}
                  </p>
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default App;
