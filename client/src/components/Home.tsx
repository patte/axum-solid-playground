import { SignInUp } from "~/components/auth/SignInUp";
import { useAuth } from "~/components/auth/AuthContext";
import { Show } from "solid-js";
import { Card } from "~/components/ui/card";

function Welcome() {
  const { me } = useAuth();
  return (
    <div>
      <div class="flex items-center justify-center">
        <h1 class="text-3xl font-bold">Hello {me()?.username}</h1>
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
