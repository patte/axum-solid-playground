# axum-solid-playground

Test your passkeys at: https://axum-solid-playground.fly.dev

Features:
- [x] Rust backend with [axum](https://github.com/tokio-rs/axum)
- [x] Database integration: rusqlite
- [x] [SolidJS](https://www.solidjs.com) frontend with [vite](https://vitejs.dev/)
- [x] [solid-ui](https://www.solid-ui.com/) for UI components
- [x] [Dev proxy](./server/src/proxy.rs) for frontend in backend
- [x] Prod: embed client js dist in rust binary 
- [x] Discoverable [passkeys](https://www.passkeys.io/technical-details) for authentication with [webauthn-rs](https://github.com/kanidm/webauthn-rs/blob/d278c56adfa39a0723c79bdcd461644194bc5138/webauthn-rs/src/lib.rs#L1270)
- [x] be: session management: [tower-sessions-rusqlite-store](https://github.com/patte/tower-sessions-rusqlite-store)
- [x] be: roll expire of session on request (max every minute)
- [x] be: session management: write informative cookie for fe
- [x] fe: session management: [AuthContext](./client/src/components/auth/AuthContext.tsx)
- [x] fe: session: detect expire and refresh ui
- [x] Deployment (fly.io)
- [x] ~~[litefs](https://fly.io/docs/litefs/) for distributed SQLite~~ removed, [no websockets](https://github.com/superfly/litefs/issues/427)
- [x] PR [maxcountryman/tower-sessions-stores#6](https://github.com/maxcountryman/tower-sessions-stores/pull/6)
- [x] publish crate [tower-sessions-rusqlite-store](https://github.com/patte/tower-sessions-rusqlite-store)
- [x] [rspc](https://github.com/oscartbeaumont/rspc)? cool idea but 🚫 [no support for axum 0.7](https://github.com/oscartbeaumont/httpz/blob/main/Cargo.toml#L50) and generally a big mess
- [x] GraphQL with [async-graphql](https://github.com/async-graphql/async-graphql) for typed api between server and client
- [x] allow users to register additional passkeys
- [x] ui: passkey details 
- [ ] security headers
- [ ] signout all my sessions
- [ ] ui: server info, debug network
- [ ] github action
- [ ] websockets?
- [ ] distributed kv?
- [ ] ssr?
- [ ] ... the possibilities are endless, the time so short

Playground to learn:
- how to combine a rust axum backend with a solidJS frontend?
- how to authenticate users with discoverable passkeys? and what's the user experience in different browsers?
- how to manage a session on the server with persistence and sync to the js client (on first render) (MVP)? - async sqlite in rust, yes please #no_orm?
- how cool is litefs and how to (ab)use it?
- how to have fun, go light and fast, with a small (<10MB) single standalone binary, that uses <100MB RAM idle.
- ... all PRs welcome 💓
- ... and all issues too 🤗

> <img width="475" alt="Screenshot 2024-02-07 at 23 10 20" src="https://github.com/patte/axum-solid-playground/assets/3500621/86e3834a-45e0-4bb4-a4fc-28d0cd7a4682"></br>
> -GitHub Copilot
> <details><summary>actually...</summary>
> <img width="882" alt="Screenshot 2024-02-07 at 23 11 58" src="https://github.com/patte/axum-solid-playground/assets/3500621/76fd47aa-2059-42a9-bfb0-6b3c9f79715a">
> </details>


## Development

Copy `.env.example` to `.env`:
```bash
cd server
cp .env.example .env
cd ..
```

The rust backend includes a [dev proxy](./server/src/proxy.rs) for the frontend, so that the host and port of the fe and be is the same, no CORS issues arise, increased dev prod parity, good dev-ex with hot reloading.

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

Open [http://localhost:3000](http://localhost:3000) to view it in your browser.

## Prod

### Local
[axum-embed](https://github.com/informationsea/axum-embed) is used to embed the frontend into the backend. For single binary niceness.

```bash
cd client
npm run build
cd ..
cargo build --release
./target/release/axum-solid-playground
```
The resulting binary is ~8MB.

### fly.io

#### volume
Create volume initially:
```bash
fly launch --no-deploy

# if no volume created during initial launch:
fly volumes create playground --region ams --size 3
```

#### envs
Set before the first deploy:
```bash
fly secrets set \
RP_ID=axum-solid-playground.fly.dev \
RP_ORIGIN=https://axum-solid-playground.fly.dev \
RP_NAME=axum-solid-playground \
DATABASE_URL=sqlite:///data/playground.db
```

#### deploy

```bash
fly deploy
```
*image size: 104 MB* (but as our binary is only ~8MB, this is what needs to be pushed in most cases)

#### add clones in other regions
Currently there is only one database on one volume (in ams). Litefs, which would enable distributed SQLite, was removed again, mainly to keep things simple and [the limitations with websockets](https://github.com/superfly/litefs/issues/427) . Only one instance can be run at a time.

remove:
```bash
fly machine ls
fly m destroy <id>
fly volumes ls
fly volumes destroy <id>
```

## Docs

### Auth
SignUp and SignIn are implemented with passkeys with [webauthn-rs](https://github.com/kanidm/webauthn-rs).

[tower-sessions](https://github.com/maxcountryman/tower-sessions/tree/52983f026f0c805598e68f82647a0865b29a60bd) with a custom [RusqliteStore](./server/src/rusqlite_session_store.rs) is used for session management.

The session is used for the passkey dance as well as to remember the authenticated user.
A cookie `authenticated_user_js` (http_only=false) is set on successful signin so that the [js frontend knows](./client/src/components/auth/AuthContext.tsx) the user is authenticated and can render appropriatly on first load.
This cookie is only informative for the client and not used to determine if the user is authenticated on the server. No auth decision on the server is based on the cookie.

The session are rolled every minute (see: roll_expiry_mw). This also keeps the informative cookie fresh.


### Browsers

Chrome (local) passkeys can be managed at [chrome://settings/passkeys](chrome://settings/passkeys).

Firefox and Safari on MacOS save them in the system keychain, which can be managed in Settings -> Passwords.
