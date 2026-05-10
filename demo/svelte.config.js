import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      pages: 'dist/static',
      assets: 'dist/static',
      fallback: 'index.html',
      precompress: false,
      strict: true,
    }),
    paths: {
      relative: true,
    },
  },
};

export default config;
