import {
  authenticationPublicKeyCredentialToJSON,
  parsePublicKeyCreationOptionsFromJSON,
  parsePublicKeyRequestOptionsFromJSON,
  registrationPublicKeyCredentialToJSON,
} from "./webauthn";

function check_credentials_support() {
  if (!navigator.credentials) {
    throw new Error(
      "Credentials API not supported, please use a different browser."
    );
  }
}

const includePrfExtension = true;

export async function register({ username }: { username: string }) {
  check_credentials_support();

  const creationChallengeResponse = await fetch(`/register_start/${username}`, {
    method: "POST",
  }).then((res) => {
    if (!res.ok) {
      throw new Error(`register_start failed: ${res.statusText}`);
    }
    return res.json() as Promise<any>;
  });
  if (!creationChallengeResponse.publicKey) {
    throw new Error("Registration failed - no publicKey");
  }
  const publicKey = parsePublicKeyCreationOptionsFromJSON(
    creationChallengeResponse.publicKey
  );

  if (includePrfExtension) {
    publicKey.extensions = {
      ...publicKey.extensions,
      // @ts-ignore
      prf: {
        eval: {
          first: new Uint8Array(new Array(32).fill(1)).buffer,
        },
      },
    };
  }

  const regCredential = (await navigator.credentials.create({
    publicKey,
  })) as PublicKeyCredential | null;
  if (!regCredential) {
    throw new Error("Registration failed - navigator.credentials.create");
  }

  if (includePrfExtension) {
    const extensionResults = regCredential?.getClientExtensionResults();
    console.log(extensionResults);
    // @ts-ignore
    console.log(`PRF supported: ${!!extensionResults?.prf?.enabled}`);
  }

  const creationResult = await fetch(`/register_finish`, {
    method: "POST",
    body: registrationPublicKeyCredentialToJSON(regCredential),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!creationResult.ok) {
    throw new Error(`register_finish failed: ${creationResult.statusText}`);
  }
  console.log("Registration complete");
  return true;
}

export type User = {
  id: string;
  username: string;
};

export async function authenticate() {
  check_credentials_support();

  const requestChallengeResponse = await fetch(`/authenticate_start`, {
    method: "POST",
  }).then((res) => {
    if (!res.ok) {
      throw new Error(`authenticate_start failed: ${res.statusText}`);
    }
    return res.json() as Promise<any>;
  });
  if (!requestChallengeResponse.publicKey) {
    throw new Error("Authentication failed - no publicKey");
  }
  const publicKey = parsePublicKeyRequestOptionsFromJSON(
    requestChallengeResponse.publicKey
  );

  const authCredential = (await navigator.credentials.get({
    publicKey,
  })) as PublicKeyCredential | null;
  if (!authCredential) {
    throw new Error("Authentication failed - navigator.credentials.get");
  }

  const authResult = await fetch(`/authenticate_finish`, {
    method: "POST",
    body: authenticationPublicKeyCredentialToJSON(authCredential),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!authResult.ok) {
    // TODO cleanup error handling: this is the way:
    throw new Error(`authenticate_finish failed: ${await authResult.text()}`);
  }

  const user = await authResult.json();

  console.log("Authentication complete");
  return user;
}
