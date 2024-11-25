import { useHotkeys } from "react-hotkeys-hook";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { useApp } from "../store/app";
import { useNavigate } from "react-router";
import { SearchIcon } from "lucide-react";
import Icon from "../components/common/Icon";
import { useSettingsStore } from "../store/settings";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function PluginList() {
  const { plugins, refreshPlugins, setSelectedPlugin, selectedPlugin } =
    useSettingsStore();
  useEffect(() => {
    refreshPlugins();
  }, []);
  return (
    <ul className="w-full h-full flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
      {Object.values(plugins)
        .sort((a, b) => a.metadata.name.localeCompare(b.metadata.name))
        .map((plugin, index) => (
          <li
            key={index}
            className={`w-full h-7 p-2 flex flex-row justify-start items-center gap-4 text-white rounded-lg outline-none focus:outline-none hover:cursor-pointer ${selectedPlugin === plugin.metadata.name && "bg-white/10"}`}
            onClick={() => setSelectedPlugin(plugin.metadata.name)}
          >
            <Icon icon={plugin.metadata.icon} size={20} />
            <span>{plugin.metadata.name}</span>
          </li>
        ))}
    </ul>
  );
}

function PluginConfig() {
  const { updatePlugin, getSelectedPlugin } = useSettingsStore();
  return (
    <>
      {getSelectedPlugin() ? (
        Object.keys(getSelectedPlugin()!.config).length > 0 ? (
          <ul className="w-full p-1 flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none border border-white/20 rounded-lg">
            {Object.entries(getSelectedPlugin()?.config ?? {}).map(
              (config, index) => (
                <li
                  key={index}
                  className="w-full h-7 px-2 flex flex-row justify-start items-center gap-4 text-white rounded-xl outline-none focus:outline-none"
                >
                  <span>{config[0]}</span>
                  <input
                    type="text"
                    placeholder={config[1]}
                    className="w-full px-2 py-[0.1rem] rounded-md bg-white/5 outline-none focus:outline-none"
                    onChange={(e) => {
                      updatePlugin({
                        ...getSelectedPlugin()!,
                        config: {
                          ...getSelectedPlugin()!.config,
                          [config[0]]: e.target.value,
                        },
                      });
                      console.log(e);
                      invoke("update_plugin_config", {
                        pluginName: getSelectedPlugin()!.metadata.name,
                        key: config[0],
                        value: e.target.value,
                      });
                      console.log(getSelectedPlugin()!.config);
                    }}
                  />
                </li>
              ),
            )}
          </ul>
        ) : (
          <div className="w-full h-full flex flex-col justify-center items-center gap-1 text-white">
            <span>No config available</span>
          </div>
        )
      ) : (
        <div className="w-full h-full flex flex-col justify-center items-center gap-1 text-white">
          <span>No plugin selected</span>
        </div>
      )}
    </>
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
      <div className="w-full h-full px-1 py-2 flex justify-evenly items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
        {/* plugin list and picker */}
        <div className="w-48 h-full pr-3 flex flex-col flex-shrink-0 flex-grow-0 justify-start items-center gap-1 text-sm text-white border-r-[0.07rem] border-white/20">
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
        <div className="w-full h-full pl-2 flex flex-col justify-start items-center gap-1 text-sm text-white">
          {/* Top bar 
          <div></div>
          */}
          {/* Settings editor */}
          <div className="w-full h-full flex flex-col justify-start items-center gap-1 overflow-y-scroll overflow-x-hidden no-scrollbar outline-none focus:outline-none">
            <PluginConfig />
          </div>
        </div>
      </div>
    </div>
  );
}

export default Settings;
