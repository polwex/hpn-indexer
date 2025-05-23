import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

/*
If you are developing a UI outside of a Hyperware project,
comment out the following 2 lines:
*/
import manifest from "../pkg/manifest.json";
import metadata from "../metadata.json";

/*
IMPORTANT:
This must match the process name from pkg/manifest.json + pkg/metadata.json
The format is "/" + "process_name:package_name:publisher_node"
*/
const BASE_URL = `/${manifest[0].process_name}:${metadata.properties.package_name}:${metadata.properties.publisher}`;

// This is the proxy URL, it must match the node you are developing against
const PROXY_URL = (
  process.env.VITE_NODE_URL || "http://127.0.0.1:8095"
).replace("localhost", "127.0.0.1");

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: BASE_URL,
  server: {
    proxy: {
      // This route will match all other HTTP requests to the backend
      [`^${BASE_URL}/(?!(@vite/client|src/.*|node_modules/.*|@react-refresh|$))`]:
        {
          target: PROXY_URL,
          changeOrigin: true,
        },

      "/our": {
        target: PROXY_URL,
        changeOrigin: true,
      },
      [`${BASE_URL}/our.js`]: {
        target: PROXY_URL,
        changeOrigin: true,
        rewrite: (path) => path.replace(BASE_URL, ""),
      },
      "/api": {
        target: PROXY_URL,
        changeOrigin: true,
        rewrite: (path) => path.replace("/api", BASE_URL),
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        inlineDynamicImports: true, // Inline dynamic imports
        manualChunks: undefined, // Disable code splitting
        format: "iife", // Use IIFE format
        entryFileNames: "bundle.js", // Name the output file
      },
    },
  },
});
