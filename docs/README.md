# json-eval-rs Documentation Site

> Documentation site for [json-eval-rs](https://github.com/byrizki/jsoneval-rs) — a high-performance JSON Logic evaluation library with schema validation and multi-platform bindings.

Built with [Nuxt](https://nuxt.com) and [Nuxt Content](https://content.nuxt.com/), deployed to [GitHub Pages](https://byrizki.github.io/jsoneval-rs/).

## 🚀 Quick Start

```bash
# Install dependencies
yarn install

# Start development server
yarn dev
```

The documentation site will be running at `http://localhost:3000`.

## 📁 Project Structure

```
docs/
├── content/
│   └── en/                     # English documentation
│       ├── index.md            # Landing page
│       ├── 01.getting-started/ # Getting started guide
│       ├── 02.how-to/          # How-to guides per platform
│       ├── 03.advance-guide/   # Advanced topics
│       └── 04.operators/       # Operator reference docs
├── components/                 # Vue components
├── public/                     # Static assets
├── nuxt.config.ts              # Nuxt configuration
└── package.json
```

## ⚡ Built With

- [Nuxt 4](https://nuxt.com) — Web framework
- [Nuxt Content](https://content.nuxt.com/) — File-based CMS
- [Nuxt i18n](https://i18n.nuxt.com/) — Internationalization

## 🚀 Deployment

Deploys automatically to GitHub Pages on every push to `main` that touches the `docs/**` path via the [deploy-docs workflow](../.github/workflows/deploy-docs.yml).

To build manually:

```bash
NUXT_APP_BASE_URL=/jsoneval-rs/ yarn nuxt build --preset github_pages
```

Output will be in `.output/public/`.

## 📄 License

[MIT License](https://opensource.org/licenses/MIT)