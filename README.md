# axum solidjs playground

My personal playground to learn rust, axum, solidjs, and passkeys.

Features:
- [x] Rust backend with [axum](https://github.com/tokio-rs/axum)
- [x] [SolidJS](https://www.solidjs.com) frontend with [vite](https://vitejs.dev/)
- [x] [solid-ui](https://www.solid-ui.com/) for UI components
- [x] Dev proxy for frontend in backend
- [x] Discoverable passkeys for authentication with [webauthn-rs](https://github.com/kanidm/webauthn-rs/blob/d278c56adfa39a0723c79bdcd461644194bc5138/webauthn-rs/src/lib.rs#L1270)
- [ ] Client side session management
- [ ] Database integration (now it's just a hashmap)
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