import type { Component } from "solid-js";
import { ColorModeProvider } from "@kobalte/core";
import Home from "~/components/Home";
import { ColorModeToggle } from "~/components/ColorModeToggle";
import { cn } from "~/lib/utils";
import { AuthProvider } from "./components/auth/AuthContext";
import SignOutButton from "./components/auth/SignOutButton";
import { GraphQLProvider } from "./components/GraphQLContext";

const App: Component = () => {
  return (
    <ColorModeProvider>
      <AuthProvider>
        <div class={cn("flex items-end justify-end p-2")}>
          <SignOutButton />
          <ColorModeToggle />
        </div>
        <GraphQLProvider>
          <Home />
        </GraphQLProvider>
      </AuthProvider>
    </ColorModeProvider>
  );
};

export default App;
