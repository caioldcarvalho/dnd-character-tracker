import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    // Tauri serves a static SPA: prerender the shell, fall back to index.html.
    adapter: adapter({ fallback: 'index.html' }),
    // Base path: empty for Tauri & local dev; set BASE_PATH (e.g.
    // "/dnd-character-tracker") in the GitHub Pages build so all assets — incl.
    // the lazily-imported WASM — resolve under the project-site subpath.
    paths: {
      base: process.env.BASE_PATH ?? ''
    },
    alias: {
      $bindings: 'src/lib/bindings'
    }
  }
};

export default config;
