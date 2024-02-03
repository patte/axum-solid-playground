import { Base64 } from "js-base64";

function check_credentials_support() {
  if (!navigator.credentials) {
    throw new Error(
      "Credentials API not supported, please use a different browser."
    );
  }
}

// parse json (into required uint8array from base64)
function publicKeyCredentialCreationOptionsFromJSON(
  publicKey: any,
  addPrfExtension: boolean = false
): PublicKeyCredentialCreationOptions {
  const firstSalt = addPrfExtension
    ? new Uint8Array(new Array(32).fill(1)).buffer
    : undefined;
  return {
    ...publicKey,
    challenge: Base64.toUint8Array(publicKey.challenge),
    user: {
      ...publicKey.user,
      id: Base64.toUint8Array(publicKey.user.id),
    },
    excludeCredentials: publicKey.excludeCredentials?.map((cred: any) => {
      return {
        ...cred,
        id: Base64.toUint8Array(cred.id),
      };
    }),
    extensions: {
      ...publicKey.extensions,
      ...(addPrfExtension
        ? {
            prf: {
              eval: {
                first: firstSalt,
              },
            },
          }
        : {}),
    },
  };
}

// stringify credential (convert uint8array to base64)
function publicKeyCredentialToJSON(credential: PublicKeyCredential): string {
  return JSON.stringify({
    id: credential.id,
    rawId: Base64.fromUint8Array(new Uint8Array(credential.rawId), true),
    response: {
      attestationObject: Base64.fromUint8Array(
        // @ts-ignore
        new Uint8Array(credential.response.attestationObject),
        true
      ),
      clientDataJSON: Base64.fromUint8Array(
        new Uint8Array(credential.response.clientDataJSON),
        true
      ),
    },
    type: credential.type,
  });
}

const includePrfExtension = true;

export async function register({ username }: { username: string }) {
  check_credentials_support();

  const ccr = await fetch(`/register_start/${username}`, {
    method: "POST",
  }).then((res) => {
    if (!res.ok) {
      throw new Error(res.statusText);
    }
    return res.json() as Promise<any>;
  });
  if (!ccr.publicKey) {
    throw new Error("Registration failed - no publicKey");
  }
  const publicKey = publicKeyCredentialCreationOptionsFromJSON(
    ccr.publicKey,
    includePrfExtension
  );

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

  const crr = await fetch(`/register_finish`, {
    method: "POST",
    body: publicKeyCredentialToJSON(regCredential),
    headers: {
      "Content-Type": "application/json",
    },
  });

  if (!crr.ok) {
    throw new Error(crr.statusText);
  }
  console.log("Registration complete");
  return true;
}
