import { As } from "@kobalte/core";

import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTrigger,
} from "./ui/sheet";

gql`
  query MeQuery {
    me {
      id
      username
      authenticators {
        userAgentShort
        createdAt
        passkey
      }
    }
  }
`;

import jsonFormatHighlight from "~/lib/jsonFormatHighlight";
import { gql } from "@solid-primitives/graphql";
import { MeQueryDocument } from "../../graphql";
import { Show, onCleanup } from "solid-js";
import { useGraphQL } from "./GraphQLContext";
import { Card } from "./ui/card";
import { toLocaleRelativeTimeString } from "~/lib/dateTimeFormat";

export default function Account() {
  const gqlClient = useGraphQL();

  const [meGraphQL, { refetch }] = gqlClient()(MeQueryDocument);

  const timer = setInterval(() => {
    refetch();
  }, 1000 * 5);
  onCleanup(() => clearInterval(timer));

  //{new Date(authenticator.createdAt).toLocaleDateString()}
  //<h2 class="text-2xl font-medium leading-none">Account</h2>
  return (
    <Show when={meGraphQL()}>
      <div class="pt-8 text-center mx-auto flex w-full px-8 flex-col justify-center space-y-2 sm:w-[350px] sm:px-0">
        <p class="text-sm text-muted-foreground">
          You have {meGraphQL()?.me?.authenticators.length} credential
          registered:
        </p>
        {meGraphQL()?.me?.authenticators.map((authenticator) => (
          <Sheet>
            <SheetTrigger asChild>
              <As component={Card} class="py-2">
                <p class="pb-2 font-semibold underline">
                  {authenticator.passkey.cred.cred_id.slice(0, 8)}
                </p>
                <div class="text-sm text-muted-foreground">
                  <p>{authenticator.userAgentShort}</p>
                  <p>
                    {toLocaleRelativeTimeString(
                      new Date(),
                      new Date(authenticator.createdAt)
                    )}
                  </p>
                </div>
              </As>
            </SheetTrigger>
            <SheetContent position={"left"} size={"content"} class="pb-0 pt-0">
              <SheetHeader class="h-full">
                <SheetDescription class="h-full overflow-y-auto py-6">
                  <pre
                    innerHTML={jsonFormatHighlight(authenticator.passkey.cred)}
                  />
                </SheetDescription>
              </SheetHeader>
            </SheetContent>
          </Sheet>
        ))}
      </div>
    </Show>
  );
}
