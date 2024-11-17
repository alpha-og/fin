import {
  useEffect,
  useState,
  useRef,
  createRef,
  RefObject,
  MutableRefObject,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { Command } from "@tauri-apps/plugin-shell";
import "./App.css";
import { SearchIcon, Folder, File, FileQuestion, Copy } from "lucide-react";
import { useHotkeys, isHotkeyPressed } from "react-hotkeys-hook";

type T_Result = {
  title: string;
  description: string | null;
  icon: string | null;
  action: any;
  priority: number;
};

const Icon = ({ icon }: { icon: string | null }) => {
  if (icon === null) return <FileQuestion size={28} className="shrink-0" />;
  switch (icon.toLowerCase()) {
    case "application":
      return <File size={28} className="shrink-0" />;
    case "folder":
      return <Folder size={28} className="shrink-0" />;
    case "file":
      return <File size={28} className="shrink-0" />;
    case "copy":
      return <Copy size={28} className="shrink-0" />;
    default:
      return <FileQuestion size={28} className="shrink-0" />;
  }
};

function App() {
  const inputRef = useRef<HTMLInputElement>(null);
  const [result, setResult] = useState<T_Result[] | null>(null);
  const [query, setQuery] = useState<string>("");
  const [history, setHistory] = useState<string[]>([]);
  const listItemRefs = useRef<RefObject<HTMLLIElement>[]>([]);
  const [selectedHistory, setSelectedHistory] = useState<number | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const currentWindow = getCurrentWindow();
  const clearSearchRef = useHotkeys(
    "escape, ctrl+[",
    (e) => {
      const target = e.target as HTMLInputElement;
      if (target.value === "") {
        currentWindow.hide();
      }

      target.value = "";
      setQuery("");
      setResult(null);
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  ) as unknown as MutableRefObject<HTMLInputElement>;
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
  ) as unknown as MutableRefObject<HTMLInputElement>;

  const selectedListItemRef = useHotkeys("escape, ctrl+[", () => {
    inputRef.current?.focus();
    setSelected(null);
    setQuery("");
  }) as unknown as MutableRefObject<HTMLLIElement>;

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
  const actionRef = useHotkeys("enter", async () => {
    const selectedResult = result![selected!]!;
    if (selectedResult.action.LaunchApplication) {
      Command.create("exec-sh", [
        "-c",
        `open ${selectedResult.action.LaunchApplication}`,
      ])
        .execute()
        .then((result) => {
          console.log(result);
        });
    } else if (selectedResult.action.Open) {
      Command.create("exec-sh", [
        "-c",
        `open -R "${selectedResult.action.Open}"`,
      ])
        .execute()
        .then((result) => {
          console.log(result);
        });
    } else if (selectedResult.action.Copy) {
      await navigator.clipboard.writeText(
        selectedResult.title.toString() || "",
      );
    }
    setSelected(null);
    inputRef.current?.focus();
  }) as unknown as MutableRefObject<HTMLLIElement>;

  async function updateResults() {
    if (query.length > 0) {
      await invoke("update_search_query", {
        query,
      });
    } else setResult(null);
  }

  useEffect(() => {
    updateResults();
  }, [query]);

  useEffect(() => {
    let interval: number;
    if (query.length > 0) {
      interval = setInterval(() => {
        invoke("get_search_results").then((response) => {
          if (JSON.stringify(response) !== JSON.stringify(result)) {
            response = (response as T_Result[]).sort(
              (a, b) => b.priority - a.priority,
            );
            setResult(response as T_Result[]);
          }
        });
      }, 100);
    }

    return () => {
      clearInterval(interval);
    };
  }, [query, result]);

  useEffect(() => {
    clearSearchRef.current = inputRef.current!;
    cycleHistoryRef.current = inputRef.current!;
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
      selectedListItemRef.current = listItemRefs.current[selected].current!;
      actionRef.current = selectedListItemRef.current;
      const listItem = listItemRefs.current[selected];
      listItem?.current?.focus();
      listItem?.current?.scrollIntoView({
        behavior: "smooth",
        block: "nearest",
      });
    }
  }, [selected]);
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
              <div className="w-full overflow-hidden">
                <p className="w-full truncate text-neutral-400">
                  {item.title.toString()}
                </p>
              </div>
              <span className="rounded-lg p-1 flex flex-row items-center justify-center hover:bg-white/10 hover:cursor-pointer">
                {<Icon icon={item.icon} />}
              </span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}

export default App;
