import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({
  publicDir: "../../frontend/public",
  plugins: [tailwindcss()],
});
