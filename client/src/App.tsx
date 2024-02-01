import type { Component } from "solid-js";
import { ColorModeProvider } from "@kobalte/core";
import Home from "./Home";

const App: Component = () => {
  return (
    <ColorModeProvider>
      <Home />
    </ColorModeProvider>
  );
};

export default App;
