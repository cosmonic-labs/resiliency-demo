import { defineConfig } from "vite";

export default defineConfig({
  build: {
    assetsInlineLimit: Infinity,
  },
});