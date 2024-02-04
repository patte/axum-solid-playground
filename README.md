# axum solidjs playground

My personal playground to learn rust, axum, solidjs, and passkeys.

Features:
- [x] Rust backend with axum
- [x] SolidJS frontend
- [x] Dev proxy for frontend
- [x] discoverable passkeys for authentication
- [ ] client side session management
- [ ] Database integration (now it's just a hashmap)
- [ ] Deployment

## Development

The rust backend includes a dev proxy for the frontend, so that the host and port for the frontend and backend is the same, no CORS issues arise and dev is as close to prod as possible while still delivering a good developer experience with hot reloading.

In one terminal, run vite:
```bash
npm install
npm run dev
```

In another terminal, run the backend server:
```bash
cargo watch -x "run --features dev_proxy"
```

Open [http://localhost:3000](http://localhost:3000) to view it in the browser.