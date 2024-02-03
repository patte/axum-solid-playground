import {
  authenticationPublicKeyCredentialToJSON,
  parsePublicKeyCreationOptionsFromJSON,
  parsePublicKeyRequestOptionsFromJSON,
  registrationPublicKeyCredentialToJSON,
} from "./webauthn";

function ensure_credentials_support() {
  if (!navigator.credentials) {
    throw new Error(
      "Credentials API not supported, please use a different browser."
    );
  }
}

// use text body if available, otherwise status and statusText
async function getErrorMessage(
  response: Response,
  location: string | undefined
) {
  return response
    .text()
    .catch(
      () =>
        `${response.status} - ${response.statusText}${
          location ? " at" + location : ""
        }`
    );
}

const includePrfExtension = true;

export type User = {
  id: string;
  username: string;
};

export async function register({
  username,
}: {
  username: string;
}): Promise<User> {
  ensure_credentials_support();

  // get challenge from server
  const creationChallengeResponse = await fetch(`/register_start/${username}`, {
    method: "POST",
  }).then(async (res) => {
    if (!res.ok) {
      throw new Error(await getErrorMessage(res, "register_start"));
    }
    return res.json() as Promise<any>;
  });

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

  // let authenticator create credential
  const _regCredential = await navigator.credentials
    .create({
      publicKey,
    })
    .catch((e) => {
      throw new Error(e.message + " - Please try again or another browser.");
    });
  if (!_regCredential) {
    throw new Error(
      "Registration failed: no credential - this should't happen!"
    );
  }
  const regCredential = _regCredential as PublicKeyCredential;

  if (includePrfExtension) {
    const extensionResults = regCredential.getClientExtensionResults();
    console.log(extensionResults);
    // @ts-ignore
    console.log(`PRF supported: ${!!extensionResults?.prf?.enabled}`);
  }

  // send credential to server
  const creationResult = await fetch(`/register_finish`, {
    method: "POST",
    body: registrationPublicKeyCredentialToJSON(regCredential),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!creationResult.ok) {
    throw new Error(await getErrorMessage(creationResult, "register_finish"));
  }
  console.log("Registration complete");

  const user = await creationResult.json();
  return user;
}

export async function authenticate() {
  ensure_credentials_support();

  // get challenge from server
  const requestChallengeResponse = await fetch(`/authenticate_start`, {
    method: "POST",
  }).then(async (res) => {
    if (!res.ok) {
      throw new Error(await getErrorMessage(res, "authenticate_start"));
    }
    return res.json() as Promise<any>;
  });

  const publicKey = parsePublicKeyRequestOptionsFromJSON(
    requestChallengeResponse.publicKey
  );

  // let authenticator create credential
  const _authCredential = await navigator.credentials
    .get({
      publicKey,
    })
    .catch((e) => {
      throw new Error(e.message + " - Please try again or another browser.");
    });
  if (!_authCredential) {
    throw new Error(
      "Authentication failed: no credential - this should't happen!"
    );
  }
  const authCredential = _authCredential as PublicKeyCredential;

  // send credential to server
  const authResult = await fetch(`/authenticate_finish`, {
    method: "POST",
    body: authenticationPublicKeyCredentialToJSON(authCredential),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!authResult.ok) {
    throw new Error(`authenticate_finish failed: ${await authResult.text()}`);
  }

  console.log("Authentication complete");

  const user = await authResult.json();
  return user;
}
