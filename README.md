# axum-solid-playground

Test your passkeys at: https://axum-solid-playground.fly.dev

Features:
- [x] Rust backend with [axum](https://github.com/tokio-rs/axum)
- [x] [SolidJS](https://www.solidjs.com) frontend with [vite](https://vitejs.dev/)
- [x] [solid-ui](https://www.solid-ui.com/) for UI components
- [x] [Dev proxy](./server/src/proxy.rs) for frontend in backend
- [x] Prod: embed client js dist in rust binary 
- [x] Discoverable [passkeys](https://www.passkeys.io/technical-details) for authentication with [webauthn-rs](https://github.com/kanidm/webauthn-rs/blob/d278c56adfa39a0723c79bdcd461644194bc5138/webauthn-rs/src/lib.rs#L1270)
- [x] Database integration (custom [tokio-rusqlite store](./server/src/rusqlite_session_store.rs) for tower-sessions) 
- [x] Client side session management (barely)
- [ ] Client side session: detect deleted on server
- [x] Deployment (fly.io)
- [x] [litefs](https://fly.io/docs/litefs/) for distributed SQLite
- [ ] PR tower-sessions-stores
- [ ] security headers
- [ ] register additional passkeys to user after first, signout all my sessions
- [ ] ui: passkey details, server info, debug network
- [ ] github action
- [ ] typed api between server and client? [rspc](https://github.com/oscartbeaumont/rspc)?
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
This deployment uses [litefs](https://fly.io/docs/litefs) for distributed SQLite.
A sidecar reverseproxy that runs inside the same VM as separate process:
- GET (and HEAD) go to local axum server (read)
- all other requests are forwarded to axum server in the primary region (write)
[Based on this line](https://github.com/superfly/litefs/blob/63eab529dc3353e8d159e097ffc4caa7badb8cb3/http/proxy_server.go#L210) only `GET` and `HEAD` requests are read all others are forwarded to the primary.
The db name `playground.db` must match in `DATABASE_URL` and litefs.yml `proxy.db`.

#### volume and litefs
Create volume initially:
```bash
fly launch --no-deploy

fly consul attach

# if no volume created during initial launch:
fly volumes create playground_litefs --region ams --size 3
```

#### envs
Set before the first deploy:
```bash
fly secrets set \
RP_ID=axum-solid-playground.fly.dev \
RP_ORIGIN=https://axum-solid-playground.fly.dev \
RP_NAME=axum-solid-playground \
LITEFS_CLOUD_TOKEN=yoursecrettoken \
DATABASE_URL=sqlite:///litefs/playground.db
```

#### deploy

```bash
fly deploy
```
*image size: 104 MB* (but as our binary is only ~8MB, this is what needs to be pushed in most cases)

#### add clones in other regions
```bash
# Add a clone in Johannesburg, South Africa
fly machine clone --select --region jnb
```
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
This cookie is only informative for the client and not used to determine if the user is authenticated on the server.

The session have a fixed lifetime of 24 hours and are not rolled over on use.


### Browsers

Chrome (local) passkeys can be managed at [chrome://settings/passkeys](chrome://settings/passkeys).

Firefox and Safari on MacOS save them in the system keychain, which can be managed in Settings -> Passwords.