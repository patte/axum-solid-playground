import { Card } from "~/components/ui/card";
import { Flex } from "~/components/ui/flex";

export default function Home() {
  return (
    <div>
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
