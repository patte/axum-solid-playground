# axum solidjs playground

## Development

The rust backend includes a dev proxy for the frontend, so that the host and port for the frontend and backend is the same and no CORS issues arise and dev is as close to prod as possible.

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