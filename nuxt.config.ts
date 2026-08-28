import tailwindcss from "@tailwindcss/vite";

export default defineNuxtConfig({
  srcDir: "app/",
  ssr: false,
  modules: ["@nuxt/icon", "@stellar/ui/nuxt"],
  icon: {
    provider: "none",
    componentName: "NuxtIcon",
    clientBundle: {
      scan: true,
    },
  },
  compatibilityDate: "2026-08-19",
  devServer: {
    port: 18081,
  },
  css: ["~/assets/css/main.css"],
  vite: {
    plugins: [tailwindcss()],
    server: {
      strictPort: true,
    },
  },
  typescript: {
    strict: true,
  },
  devtools: {
    enabled: true,
  },
});
