import { SignInUp } from "~/components/auth/SignInUp";
import { useAuth } from "~/components/auth/AuthContext";
import { Show } from "solid-js";
import { Card } from "~/components/ui/card";

function Welcome() {
  const { me } = useAuth();
  return (
    <div>
      <div class="flex items-center justify-center">
        <Card class="w-full max-w-xs p-6 text-center">
          <h1 class="text-3xl font-bold">Hello {me()?.username}</h1>
        </Card>
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
