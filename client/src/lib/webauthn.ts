import { Base64 } from "js-base64";

// parsing and stringifying functions for webauthn

// parse json (into required uint8array from base64)
// because parseCreationOptionsFromJSON is not yet supported in all browsers
// https://developer.mozilla.org/en-US/docs/Web/API/PublicKeyCredential/parseCreationOptionsFromJSON_static
export function parsePublicKeyCreationOptionsFromJSON(
  publicKey: any
): PublicKeyCredentialCreationOptions {
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
    },
  };
}

// stringify credential (convert uint8array to base64)
// because PublicKeyCredential.toJSON is not yet supported in all browsers
// https://developer.mozilla.org/en-US/docs/Web/API/PublicKeyCredential/toJSON
export function registrationPublicKeyCredentialToJSON(
  credential: PublicKeyCredential
): string {
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

// parse json (into required uint8array from base64)
// because parseRequestOptionsFromJSON is not yet supported in all browsers
// https://developer.mozilla.org/en-US/docs/Web/API/PublicKeyCredential/parseRequestOptionsFromJSON_static
export function parsePublicKeyRequestOptionsFromJSON(
  publicKey: any
): PublicKeyCredentialRequestOptions {
  return {
    ...publicKey,
    challenge: Base64.toUint8Array(publicKey.challenge),
    allowCredentials: publicKey.allowCredentials?.map((cred: any) => {
      return {
        ...cred,
        id: Base64.toUint8Array(cred.id),
      };
    }),
  };
}

// because PublicKeyCredential.toJSON is not yet supported in all browsers
// https://developer.mozilla.org/en-US/docs/Web/API/PublicKeyCredential/toJSON
export function authenticationPublicKeyCredentialToJSON(
  credential: PublicKeyCredential
): string {
  return JSON.stringify({
    id: credential.id,
    rawId: Base64.fromUint8Array(new Uint8Array(credential.rawId), true),
    response: {
      authenticatorData: Base64.fromUint8Array(
        // @ts-ignore
        new Uint8Array(credential.response.authenticatorData),
        true
      ),
      clientDataJSON: Base64.fromUint8Array(
        new Uint8Array(credential.response.clientDataJSON),
        true
      ),
      signature: Base64.fromUint8Array(
        // @ts-ignore
        new Uint8Array(credential.response.signature),
        true
      ),
      userHandle: Base64.fromUint8Array(
        // @ts-ignore
        new Uint8Array(credential.response.userHandle),
        true
      ),
    },
    type: credential.type,
  });
}
