import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    // Tauri serves a static SPA: prerender the shell, fall back to index.html.
    adapter: adapter({ fallback: 'index.html' }),
    alias: {
      $bindings: 'src/lib/bindings'
    }
  }
};

export default config;
