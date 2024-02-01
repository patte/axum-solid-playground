import { Show } from "solid-js";
import { useAuth } from "./AuthContext";
import { TbLogout } from "solid-icons/tb";
import { Button } from "../ui/button";

export default function SignOutButton() {
  const { isSignedIn, signOut } = useAuth();
  return (
    <Show when={isSignedIn()}>
      <Button variant={"ghost"} size="sm" class="w-9 mx-1" onClick={signOut}>
        <TbLogout class="w-6 h-6" />
      </Button>
    </Show>
  );
}
