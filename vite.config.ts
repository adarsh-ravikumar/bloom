import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  root: "client",

  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },

  resolve: {
    alias: {
      "@client": resolve(__dirname, "bloom/client"),

      "@core": resolve(__dirname, "bloom/client/core"),
      "@widgets": resolve(__dirname, "bloom/client/widgets"),
      "@style": resolve(__dirname, "bloom/client/style"),

      "@registry": resolve(__dirname, "bloom/client/registry"),
      "@serialization": resolve(__dirname, "bloom/client/serialization"),
      "@transport": resolve(__dirname, "bloom/client/transport"),

      "@app": resolve(__dirname, "src"),
    },
  },
});
