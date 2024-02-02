import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import tsconfigPaths from "vite-tsconfig-paths";
// import devtools from 'solid-devtools/vite';

export default defineConfig({
  plugins: [
    /* 
    Uncomment the following line to enable solid-devtools.
    For more info see https://github.com/thetarnav/solid-devtools/tree/main/packages/extension#readme
    */
    // devtools(),
    solidPlugin(),
    tsconfigPaths(),
  ],
  server: {
    host: "localhost",
    port: 3001,
    // this is necessary for hmr to not use the axum proxy
    // but still connect directly to vite
    hmr: {
      path: "/socket.io",
      clientPort: 3001,
    },
  },
  build: {
    target: "esnext",
  },
});
