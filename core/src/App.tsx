import "./App.css";
import Settings from "./pages/Settings";
import Search from "./pages/Search";
import { Route, Routes } from "react-router";
// import { useNavigate } from "react-router";
// import { useEffect } from "react";

function App() {
  // const navigate = useNavigate();
  // useEffect(() => {
  //   navigate("/settings");
  // }, []);
  return (
    <div
      data-tauri-drag-region
      className="h-screen w-screen p-1 flex flex-col items-center justify-start rounded-2xl bg-zinc-800 overflow-hidden"
    >
      <Routes>
        <Route path="/" element={<Search />} />
        <Route path="/settings" element={<Settings />} />
      </Routes>
    </div>
  );
}

export default App;
