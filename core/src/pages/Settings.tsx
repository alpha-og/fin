import { useHotkeys } from "react-hotkeys-hook";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useApp } from "../store/app";
import { useNavigate } from "react-router";
import { SearchIcon } from "lucide-react";
import Icon from "../components/common/Icon";
import { useSettingsStore } from "../store/settings";
import { useEffect } from "react";

function PluginList() {
  const { plugins, refreshPlugins } = useSettingsStore();
  useEffect(() => {
    refreshPlugins();
  }, []);
  return (
    <ul className="w-full h-full flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
      {Object.values(plugins).map((plugin, index) => (
        <li
          key={index}
          className="w-full h-7 flex flex-row justify-start items-center gap-4 text-white rounded-xl outline-none focus:outline-none"
        >
          <Icon icon={plugin.metadata.icon} size={20} />
          <span>{plugin.metadata.name}</span>
        </li>
      ))}
    </ul>
  );
}

function Settings() {
  const currentWindow = getCurrentWindow();
  const { setCurrentPage } = useApp();
  const navigate = useNavigate();

  useHotkeys("escape, ctrl+[", () => {
    currentWindow.setSize(new LogicalSize(600, 50));
    navigate("/");
    setCurrentPage("search");
  });

  return (
    <div
      data-tauri-drag-region
      className="w-full h-full p-1 flex flex-col justify-evenly items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none"
    >
      <div className="h-3 flex flex-row items-center gap-1 rounded-md bg-zinc-800 text-white">
        <span className="text-white">Settings</span>
      </div>
      <div className="w-full h-full p-1 flex justify-evenly items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
        {/* plugin list and picker */}
        <div className="w-48 h-full p-1 pr-3 flex flex-col flex-shrink-0 flex-grow-0 justify-start items-center gap-1 text-sm text-white border-r-[0.07rem] border-white/20">
          {/* search bar */}
          <div className="w-full px-2 py-1 flex flex-row items-center gap-1 rounded-md bg-zinc-800 border border-white/20">
            <SearchIcon size={20} />
            <input
              type="text"
              placeholder="Search..."
              className="w-full px-1 rounded-md bg-transparent outline-none focus:outline-none"
            />
          </div>
          {/* list of plugins */}
          <div className="w-full h-full p-1 flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
            <PluginList />
          </div>
        </div>
        {/* plugin config */}
        <div className="w-full h-full flex flex-col justify-start items-center gap-1">
          {/* Top bar */}
          <div></div>
          {/* Settings editor */}
          <div></div>
        </div>
      </div>
    </div>
  );
}

export default Settings;
