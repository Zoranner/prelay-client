import tailwindcss from "@tailwindcss/vite";

const stellarStyles = "@stellar/ui/styles";

export default defineNuxtConfig({
  srcDir: "app/",
  ssr: false,
  modules: [
    "@nuxt/icon",
    "@stellar/ui/nuxt",
    (_options, nuxt) => {
      nuxt.hook("modules:done", () => {
        const index = nuxt.options.css.lastIndexOf(stellarStyles);
        if (index === -1) return;

        nuxt.options.css.splice(index, 1);
        nuxt.options.css.unshift(stellarStyles);
      });
    },
  ],
  icon: {
    provider: "none",
    componentName: "NuxtIcon",
    clientBundle: {
      scan: true,
      icons: ["ph:arrows-in", "ph:arrows-out", "ph:spinner-gap"],
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
