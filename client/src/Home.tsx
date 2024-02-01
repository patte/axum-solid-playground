import { Card } from "~/components/ui/card";

export default function Home() {
  return (
    <div>
      <div class="flex items-center justify-center">
        <Card class="w-full max-w-xs p-6 text-center">
          <h1 class="text-3xl font-bold underline">Hello world!</h1>
        </Card>
      </div>
    </div>
  );
}
