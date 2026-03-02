export default defineNuxtConfig({
  modules: ["@nuxtjs/i18n"],
  app: {
    head: {
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }
      ]
    },
    baseURL: '/jsoneval-rs/',
  },
  i18n: {
    defaultLocale: "en",
    locales: [
      {
        code: "en",
        name: "English",
      },
      {
        code: "id",
        name: "Indonesia",
      },
    ],
  },
  content: {
    build: {
      markdown: {
        highlight: {
          theme: {
            default: "github-light",
            dark: "github-dark",
            sepia: "monokai",
          },
          langs: [
            "js",
            "ts",
            "json",
            "bash",
            "sh",
            "toml",
            "csharp",
            "rust",
            "html",
          ],
        },
      },
    },
  },
});
