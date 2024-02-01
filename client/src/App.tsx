import type { Component } from "solid-js";
import { ColorModeProvider } from "@kobalte/core";
import Home from "./Home";
import { ColorModeToggle } from "~/components/ColorModeToggle";
import { cn } from "~/lib/utils";

const App: Component = () => {
  return (
    <ColorModeProvider>
      <div class={cn("flex items-end justify-end p-2")}>
        <ColorModeToggle />
      </div>
      <Home />
    </ColorModeProvider>
  );
};

export default App;
