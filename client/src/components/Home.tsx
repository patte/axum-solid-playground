import { SignInUp } from "~/components/auth/SignInUp";
import { useAuth } from "~/components/auth/AuthContext";
import { Show } from "solid-js";
import ChatComponent from "./Chat";

function Welcome() {
  const { me } = useAuth();
  return (
    <div>
      <div class="flex items-center justify-center flex-col">
        <h1 class="text-3xl font-bold">Hello {me()?.username}</h1>
        <ChatComponent />
      </div>
    </div>
  );
}

export default function Home() {
  const { isSignedIn } = useAuth();
  return (
    <Show when={isSignedIn()} fallback={<SignInUp />}>
      <Welcome />
    </Show>
  );
}
