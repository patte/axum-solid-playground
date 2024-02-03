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
import InputError from "~/components/InputError";
import { register } from "~/lib/auth";
import { createSignal } from "solid-js";

function UserAuthForm() {
  const [authForm, { Form, Field }] = createForm<AuthForm>();
  const [authError, setAuthError] = createSignal<string | null>(null);

  const handleSubmitRegister: SubmitHandler<AuthForm> = (values) => {
    return new Promise((resolve) =>
      setTimeout(() => {
        register({ username: values.username }).catch((error) => {
          setAuthError(error.message);
          throw error;
        });
        resolve(true);
      }, 800)
    );
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
            {authForm.submitting && (
              <TbLoader class="mr-2 h-4 w-4 animate-spin" />
            )}
            Create account
          </Button>
        </Grid>
      </Form>
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
      <Button variant="outline" type="button" disabled={authForm.submitting}>
        {authForm.submitting ? (
          <TbLoader class="mr-2 h-4 w-4 animate-spin" />
        ) : (
          <TbKey class="mr-2 h-4 w-4" />
        )}{" "}
        Passkey
      </Button>
      {authError() && (
        <p class="text-red-500 text-sm text-center">{authError()}</p>
      )}
    </div>
  );
}

export function SignInUp() {
  return (
    <>
      <div class="container relative h-[600px] flex-col items-center justify-center md:grid lg:max-w-none lg:px-0">
        <div class="lg:p-8">
          <div class="mx-auto flex w-full flex-col justify-center space-y-6 sm:w-[350px]">
            <div class="flex flex-col space-y-2 text-center">
              <h1 class="text-2xl font-semibold tracking-tight">
                Create an account
              </h1>
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
