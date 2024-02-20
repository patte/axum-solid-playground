import { Sheet, SheetContent, SheetDescription, SheetHeader } from "./ui/sheet";

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
import { Show, createSignal, onCleanup } from "solid-js";
import { useGraphQL } from "./GraphQLContext";
import { Card } from "./ui/card";
import { toLocaleRelativeTimeString } from "~/lib/dateTimeFormat";
import { RegisterButton } from "./auth/SignInUp";
import { GenericError } from "./InputError";
import { register } from "~/lib/auth";
import { useAuth } from "./auth/AuthContext";

function AddCredential({ cb }: { cb: () => void }) {
  const [error, setError] = createSignal<string | null>(null);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const { me } = useAuth();

  const handleClick: (e: Event) => void = (e) => {
    setError(null);
    setIsSubmitting(true);
    register({ username: me()?.username || "" })
      .then((_user) => {
        setIsSubmitting(false);
        cb();
      })
      .catch((error) => {
        setError(error.message);
        throw error;
      });
  };

  return (
    <>
      <RegisterButton
        buttonText="add another credential"
        submitting={isSubmitting()}
        onClick={handleClick}
      />
      {error() && <GenericError error={error()} />}
    </>
  );
}

export default function Account() {
  const gqlClient = useGraphQL();

  const [meGraphQL, { refetch }] = gqlClient()(MeQueryDocument);

  const [showAuthId, setShowAuthId] = createSignal<string | null>(null);

  const timer = setInterval(() => {
    refetch();
  }, 1000 * 15);
  onCleanup(() => clearInterval(timer));

  return (
    <Show when={meGraphQL()}>
      <div class="pt-8 text-center mx-auto flex w-full px-8 flex-col justify-center space-y-2 sm:w-[350px] sm:px-0">
        <p class="text-sm text-muted-foreground">
          You have {meGraphQL()?.me?.authenticators.length} credential
          registered:
        </p>
        {meGraphQL()?.me?.authenticators.map((authenticator) => (
          <div
            role="button"
            onClick={() => setShowAuthId(authenticator.passkey.cred.cred_id)}
          >
            <Card class="py-2">
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
            </Card>
          </div>
        ))}
        <Sheet
          open={showAuthId() === null ? false : true}
          onOpenChange={(isOpen) => {
            if (!isOpen) setShowAuthId(null);
          }}
        >
          <SheetContent
            position={"left"}
            size={"content"}
            class="w-screen md:w-auto h-full pb-0 pt-0"
          >
            <SheetHeader class="h-full">
              <SheetDescription class="text-left h-full overflow-y-auto py-6">
                <pre
                  innerHTML={jsonFormatHighlight(
                    meGraphQL()?.me?.authenticators.find(
                      (auth) => auth.passkey.cred.cred_id === showAuthId()
                    )?.passkey.cred
                  )}
                />
              </SheetDescription>
            </SheetHeader>
          </SheetContent>
        </Sheet>
        <AddCredential cb={refetch} />
      </div>
    </Show>
  );
}
