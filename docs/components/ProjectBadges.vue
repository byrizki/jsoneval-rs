<script setup lang="ts">
const badges = [
  { alt: 'GitHub Release', src: 'https://img.shields.io/github/v/release/byrizki/jsoneval-rs?display_name=release&logo=github', href: 'https://github.com/byrizki/jsoneval-rs/releases/latest' },
  { alt: 'Crates.io', src: 'https://img.shields.io/crates/v/json-eval-rs?label=json-eval-rs&logo=rust', href: 'https://crates.io/crates/json-eval-rs' },
  { alt: 'NuGet', src: 'https://img.shields.io/nuget/v/JsonEvalRs?label=JsonEvalRs&logo=nuget', href: 'https://www.nuget.org/packages/JsonEvalRs' },
  { alt: 'RN', src: 'https://img.shields.io/npm/v/@json-eval-rs/react-native?label=@json-eval-rs/react-native&logo=npm', href: 'https://www.npmjs.com/package/@json-eval-rs/react-native' },
  { alt: 'Web Core', src: 'https://img.shields.io/npm/v/@json-eval-rs/webcore?label=@json-eval-rs/webcore&logo=npm', href: 'https://www.npmjs.com/package/@json-eval-rs/webcore' },
  { alt: 'Bundler', src: 'https://img.shields.io/npm/v/@json-eval-rs/bundler?label=@json-eval-rs/bundler&logo=npm', href: 'https://www.npmjs.com/package/@json-eval-rs/bundler' },
  { alt: 'Vanilla Web', src: 'https://img.shields.io/npm/v/@json-eval-rs/vanilla?label=@json-eval-rs/vanilla&logo=npm', href: 'https://www.npmjs.com/package/@json-eval-rs/vanilla' },
  { alt: 'Node', src: 'https://img.shields.io/npm/v/@json-eval-rs/node?label=@json-eval-rs/node&logo=npm', href: 'https://www.npmjs.com/package/@json-eval-rs/node' }
]
</script>

<template>
  <div class="flex flex-wrap items-center gap-3 my-4">
    <a
      v-for="badge in badges"
      :key="badge.alt"
      :href="badge.href"
      target="_blank"
      rel="noopener noreferrer"
      class="hover:opacity-80 transition-opacity"
    >
      <img :src="badge.src" :alt="badge.alt" class="h-[28px] w-auto" />
    </a>
  </div>
</template>
