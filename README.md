# axum solidjs playground

My personal playground to learn rust, axum, solidjs, and passkeys.

Features:
- [x] Rust backend with [axum](https://github.com/tokio-rs/axum)
- [x] [SolidJS](https://www.solidjs.com) frontend with [vite](https://vitejs.dev/)
- [x] [solid-ui](https://www.solid-ui.com/) for UI components
- [x] Dev proxy for frontend in backend
- [x] Discoverable passkeys for authentication with [webauthn-rs](https://github.com/kanidm/webauthn-rs/blob/d278c56adfa39a0723c79bdcd461644194bc5138/webauthn-rs/src/lib.rs#L1270)
- [x] Database integration (custom [tokio-rusqlite store](./server/src/rusqlite_session_store.rs) for tower-sessions)
- [x] Client side session management
- [x] Prod: embed client js app in rust binary 
- [ ] Deployment

## Development

The rust backend includes a dev proxy for the frontend, so that the host and port for the frontend and backend is the same, no CORS issues arise and dev is as close to prod as possible while still delivering a good developer experience with hot reloading.

In one terminal, run vite:
```bash
cd client
npm install
npm run dev
```

In another terminal, run the backend server:
```bash
cd server
cargo watch -x "run --features dev_proxy"
```

Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

## Prod
[axum-embed](https://github.com/informationsea/axum-embed) is used to embed the frontend into the backend.

```bash
npm run build
cargo run --release
```

## Docs

### Auth
SignUp and SignIn are implemented with passkeys with [webauthn-rs](https://github.com/kanidm/webauthn-rs).

[tower-sessions](https://github.com/maxcountryman/tower-sessions/tree/52983f026f0c805598e68f82647a0865b29a60bd) with a custom [RusqliteStore](./server/src/rusqlite_session_store.rs) is used for session management.

The session is used for the passkey dance as well as to remember the authenticated user.
A cookie `authenticated_user_js` (http_only=false) is set on successful signin so that the [js frontend knows](./client/src/components/auth/AuthContext.tsx) the user is authenticated and can render appropriatly on first load.
This cookie is only informative for the client and not used to determine if the user is authenticated on the server.

The session have a fixed lifetime of 24 hours and are not rolled over on use.


### Browsers

Chrome (local) passkeys can be managed at [chrome://settings/passkeys](chrome://settings/passkeys).

Firefox and Safari on MacOS save them in the system keychain, which can be managed in Settings -> Passwords.