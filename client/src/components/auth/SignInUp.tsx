type AuthForm = {
  username: string;
};

import type { SubmitHandler } from "@modular-forms/solid";
import { createForm, required, minLength } from "@modular-forms/solid";
import { Button } from "~/components/ui/button";
import { Grid } from "~/components/ui/grid";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import { TbKey, TbLoader } from "solid-icons/tb";
import { InputError, GenericError } from "~/components/InputError";
import { register, authenticate } from "~/lib/auth";
import { createSignal } from "solid-js";
import { useAuth } from "./AuthContext";

function UserAuthForm() {
  const [authForm, { Form, Field }] = createForm<AuthForm>();
  const [registrationError, setRegistrationError] = createSignal<string | null>(
    null
  );
  const [authenticationError, setAuthenticationError] = createSignal<
    string | null
  >(null);
  const { signIn } = useAuth();

  const handleSubmitRegister: SubmitHandler<AuthForm> = (values) => {
    setRegistrationError(null);
    setAuthenticationError(null);
    return new Promise((resolve) =>
      setTimeout(() => {
        const username = values.username;
        register({ username })
          .then((user) => {
            signIn(user);
          })
          .catch((error) => {
            setRegistrationError(error.message);
            throw error;
          });
        resolve(true);
      }, 800)
    );
  };

  const handleClickSignIn: (e: Event) => void = (e) => {
    setRegistrationError(null);
    setAuthenticationError(null);
    authenticate()
      .then((user) => {
        signIn(user);
      })
      .catch((error) => {
        setAuthenticationError(error.message);
        throw error;
      });
  };

  return (
    <div class="grid gap-6">
      <Form onSubmit={handleSubmitRegister}>
        <Grid class="gap-4">
          <Field
            name="username"
            validate={[
              required("Username is required"),
              minLength(3, "Username must be at least 3 characters long"),
            ]}
          >
            {(field, props) => (
              <Grid class="gap-1">
                <Label class="sr-only" for="username">
                  Username
                </Label>
                <Input {...props} type="text" placeholder="yourname" />
                {field.touched && field.error && (
                  <InputError error={field.error} />
                )}
              </Grid>
            )}
          </Field>
          <Button type="submit" disabled={authForm.submitting}>
            {authForm.submitting ? (
              <TbLoader class="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <TbKey class="mr-2 h-4 w-4" />
            )}{" "}
            Create account
          </Button>
        </Grid>
      </Form>
      {registrationError() && <GenericError error={registrationError()} />}
      <div class="relative">
        <div class="absolute inset-0 flex items-center">
          <span class="w-full border-t" />
        </div>
        <div class="relative flex justify-center text-xs uppercase">
          <span class="bg-background text-muted-foreground px-2">
            Or continue with
          </span>
        </div>
      </div>

      <Button
        variant="outline"
        type="button"
        disabled={authForm.submitting}
        onClick={handleClickSignIn}
      >
        {authForm.submitting ? (
          <TbLoader class="mr-2 h-4 w-4 animate-spin" />
        ) : (
          <TbKey class="mr-2 h-4 w-4" />
        )}{" "}
        Login
      </Button>

      {authenticationError() && <GenericError error={authenticationError()} />}
    </div>
  );
}

export function SignInUp() {
  return (
    <>
      <h1 class="text-6xl font-bold text-center">Playground</h1>
      <div class="container relative h-[400px] flex-col items-center justify-center sm:grid sm:px-0 pt-[58px] sm:pt-0">
        <div class="lg:p-8">
          <div class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
            <div class="flex flex-col space-y-2 text-center">
              <h2 class="text-2xl font-semibold tracking-tight">
                Create an account
              </h2>
              <p class="text-muted-foreground text-sm">
                Enter a username below to create an account.
              </p>
            </div>
            <UserAuthForm />
          </div>
        </div>
      </div>
    </>
  );
}
