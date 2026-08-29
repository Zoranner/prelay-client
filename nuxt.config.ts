import tailwindcss from "@tailwindcss/vite";

const shikiLite = decodeURIComponent(
  new URL("./app/utils/shikiLite.ts", import.meta.url).pathname,
).slice(1);

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
    resolve: {
      alias: {
        shiki: shikiLite,
      },
    },
    build: {
      rolldownOptions: {
        checks: {
          pluginTimings: false,
        },
        output: {
          codeSplitting: {
            groups: [
              {
                name: "vendor",
                test: /node_modules[\\/]/,
                priority: -1,
                maxSize: 400 * 1024,
                entriesAware: true,
              },
            ],
          },
        },
      },
    },
    server: {
      strictPort: true,
    },
  },
  typescript: {
    strict: true,
  },
  nitro: {
    externals: {
      inline: ["@vue/shared"],
    },
  },
  devtools: {
    enabled: true,
  },
});
