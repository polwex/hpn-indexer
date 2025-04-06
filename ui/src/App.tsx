import React from "react";
import { ConnectButton } from "@rainbow-me/rainbowkit";
import SearchPage from "./components/HpnSearch.tsx";

function App() {
  return (
    <div className="app-container">
      <header className="app-header">
        <h1 className="app-title">Hyperware Provider Network</h1>
        <ConnectButton />
      </header>
      <SearchPage />
    </div>
  );
}

export default App;
