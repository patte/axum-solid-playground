import { Card } from "~/components/ui/card";
import { Flex } from "~/components/ui/flex";
import { useColorMode } from "@kobalte/core";

export default function Home() {
  const { setColorMode } = useColorMode();
  return (
    <div>
      <button onClick={() => setColorMode("dark")}>Dark</button>
      <button onClick={() => setColorMode("light")}>Light</button>
      <h1 class="text-3xl font-bold underline">Hello world!</h1>
      <Card class="w-full max-w-xs p-6">
        <Flex>
          <div>
            <p class="text-card-foreground/70 text-sm font-normal">
              Tickets sold
            </p>
            <p class="text-card-foreground text-3xl font-semibold">9,876</p>
          </div>
          <div>
            <p class="text-card-foreground/70 text-sm font-normal">
              Average Selling Price
            </p>
            <p class="text-card-foreground text-3xl font-semibold">$ 175.20</p>
          </div>
        </Flex>
      </Card>
    </div>
  );
}
