import { defineConfig } from "vite";
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { resolve } from "node:path";

const ROOT = resolve(import.meta.dirname);

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    svelte({
      configFile: resolve(ROOT, "./svelte.config.js")
    })
  ],

  resolve: {
    alias: {
      "@core": resolve(ROOT, "bloom/client/src/lib/core"),
    },
  },
})
