import { SearchIcon } from "lucide-react";
import Icon from "../components/common/Icon";
import {
  useEffect,
  useRef,
  createRef,
  RefObject,
  MutableRefObject,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { Command } from "@tauri-apps/plugin-shell";
import { useHotkeys, isHotkeyPressed } from "react-hotkeys-hook";
import { useApp } from "../store/app";
import { useNavigate } from "react-router";
import { T_Result, useSearchStore } from "../store/search";

function Search() {
  const inputRef = useRef<HTMLInputElement>(null);
  const {
    query,
    results,
    history,
    selected,
    selectedHistory,
    setQuery,
    setResults,
    setHistory,
    setSelected,
    setSelectedHistory,
  } = useSearchStore();
  const listItemRefs = useRef<RefObject<HTMLLIElement>[]>([]);
  const currentWindow = getCurrentWindow();
  const { setCurrentPage } = useApp();
  const navigate = useNavigate();

  const clearSearchRef = useHotkeys(
    "escape, ctrl+[",
    (e) => {
      const target = e.target as HTMLInputElement;
      if (target.value === "") {
        currentWindow.hide();
      }

      target.value = "";
      setQuery("");
      setResults([]);
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
      if (results.length > 0) {
        if (selected === null) {
          setHistory([...history, query]);
          setSelected(0);
        } else {
          setSelected((selected + 1) % results.length);
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
      if (results.length > 0) {
        if (selected === null) {
          setHistory([...history, query]);
          setSelected(results.length - 1);
        } else {
          setSelected((selected + results.length - 1) % results.length);
        }
      }
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );

  const actionRef = useHotkeys("enter", async () => {
    const selectedResult = results![selected!]!;
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

  useHotkeys(
    "meta+comma",
    async () => {
      currentWindow.setSize(new LogicalSize(600, 400));
      navigate("/settings");
      setCurrentPage("settings");
    },
    {
      preventDefault: true,
      enableOnFormTags: ["INPUT"],
    },
  );

  async function updateResults() {
    if (query.length > 0) {
      await invoke("update_search_query", {
        query,
      });
    } else setResults([]);
  }

  useEffect(() => {
    updateResults();
  }, [query]);

  useEffect(() => {
    let interval: number;
    if (query.length > 0) {
      interval = setInterval(() => {
        invoke("get_search_results").then((response) => {
          if (JSON.stringify(response) !== JSON.stringify(results)) {
            response = (response as T_Result[]).sort(
              (a, b) => b.priority - a.priority,
            );
            console.log(response);
            setResults(response as T_Result[]);
          }
        });
      }, 100);
    }

    return () => {
      clearInterval(interval);
    };
  }, [query, results]);

  useEffect(() => {
    clearSearchRef.current = inputRef.current!;
    cycleHistoryRef.current = inputRef.current!;
  }, [inputRef]);

  useEffect(() => {
    if ((results.length > 0 && results.length > 0) || query) {
      currentWindow.setSize(new LogicalSize(600, 400));
      listItemRefs.current = Array(results.length)
        .fill(null)
        .map(() => createRef<HTMLLIElement>());
      setSelected(null);
    } else {
      currentWindow.setSize(new LogicalSize(600, 50));
    }
  }, [query, results]);
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
    <>
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
      {results.length > 0 && (
        <ul
          tabIndex={1}
          className="w-full h-full p-1 flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none"
        >
          {results?.map((item, index) => (
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
                {<Icon icon={item.icon} size={20} />}
              </span>
            </li>
          ))}
        </ul>
      )}
    </>
  );
}
export default Search;
