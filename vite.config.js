import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import vuetify from 'vite-plugin-vuetify';
import { aliases as vuetifyIcons } from 'vuetify/iconsets/mdi';
import { SOURCE_ROOTS, glyphRule, iconsUsed } from './tools/mdi-icons.mjs';
import { fileURLToPath, URL } from 'node:url';

// Tauri drives the dev server, so the port is fixed and HMR must be reachable
// from the webview. Failing loudly on a taken port beats Vite silently moving
// to 1421 while tauri.conf.json still points at 1420.
const host = process.env.TAURI_DEV_HOST;

/**
 * Ship one font format instead of four.
 *
 * The Material Design Icons stylesheet declares eot, woff2, woff and ttf, and
 * Vite bundles every file a stylesheet references. That is 3.4 MB of fonts in
 * the app bundle to deliver one typeface: .eot exists for Internet Explorer,
 * and .woff/.ttf are fallbacks for browsers that predate woff2.
 *
 * This app runs in exactly three engines — WKWebView, WebView2 and WebKitGTK —
 * and all three have supported woff2 for years. There is no browser to fall
 * back for, so the fallbacks are pure weight in a bundle that gets written to
 * the user's disk.
 *
 * Rewriting the src descriptor rather than deleting files: unreferenced assets
 * are simply never emitted.
 */
function mdiWoff2Only() {
  return {
    name: 'mdi-woff2-only',
    enforce: 'pre',
    transform(code, id) {
      if (!id.includes('materialdesignicons.css')) return null;

      // Two src declarations, not one: the stylesheet emits a bare
      // `src: url(...eot)` for IE8 and then a second `src:` listing eot?#iefix,
      // woff2, woff and ttf. Replacing only the first leaves every file still
      // referenced, so both have to go.
      let replaced = 0;
      const trimmed = code.replace(/src:\s*url\([^)]*\)[^;]*;/g, () => {
        replaced += 1;
        return replaced === 1
          ? 'src: url("../fonts/materialdesignicons-webfont.woff2") format("woff2");'
          : '';
      });

      if (replaced !== 2) {
        // The stylesheet changed shape upstream. Failing loudly beats silently
        // shipping four formats again while the comment above claims otherwise.
        this.error(`mdi-woff2-only: expected 2 src declarations to rewrite, found ${replaced}`);
      }
      return { code: trimmed, map: null };
    },
  };
}

/**
 * Ship the icon rules this application uses, not the seven thousand it does not.
 *
 * The same argument as `mdiWoff2Only` one step further along. That one dropped
 * three font formats no engine here needs; this one drops the class rules for
 * icons no screen here names. Material Design Icons declares 7,447 of them —
 * 408 KB of `.mdi-x::before { content: "\Fxxxx" }` — and this application names
 * 313. The stylesheet was the single largest thing in the bundle and almost all
 * of it described glyphs nothing could reach.
 *
 * **Only the glyph rules.** They are recognised by their exact shape — a
 * `::before` with a body that is nothing but `content` — which is what separates
 * them from the modifiers that share the prefix: `.mdi-rotate-45`, `.mdi-flip-h`
 * and `.mdi-spin` are written with a single colon and set a transform, so they
 * do not match and are never dropped. Nor is `@font-face`, nor `.mdi:before`,
 * nor the keyframes.
 *
 * **The font itself is untouched**, and that is a stated limit rather than an
 * oversight: subsetting a woff2 needs a font toolchain, and the 394 KB it would
 * save is the next piece of this work. The stylesheet needed no dependency at
 * all.
 *
 * Which icons count as used is `tools/mdi-icons.mjs`, not a scan written here:
 * the built stylesheet is held against the same list after the build, and the
 * list is held against the icon set by a test. Three readers, one answer.
 *
 * ## `apply: 'build'`, and it is a bug fix rather than an optimisation
 *
 * This used to run in dev as well, and it produced a failure with no symptom
 * anybody could act on: **an icon added while the dev server was running never
 * appeared.** The scan happens inside `transform`, so it reads the source trees
 * once — the first time Vite asks for this stylesheet — and Vite then caches
 * the result. Adding `mdi-source-branch-check` to a new pane changes a `.vue`
 * file, and nothing in the module graph connects a `.vue` file to
 * `materialdesignicons.css`, so the cached stylesheet is served unchanged for
 * the rest of the session. The pane draws a blank square, the name is a real
 * icon, the scan does collect it, and every check anybody would run says the
 * icon is fine — because it is.
 *
 * Subsetting exists to shrink the **shipped** bundle. In dev, bundle size is
 * not a thing anybody is measuring and correctness is, so the whole stylesheet
 * is served and the trap goes with it. Nothing is lost by the change: an icon
 * name that is not an icon is caught by `tests/mdi-icons.spec.js`, which holds
 * every name against the icon set, and a rule wrongly stripped is caught by
 * `tools/check-bundle.mjs` after the build — two gates that fail loudly, where
 * the dev-server square failed silently.
 */
function mdiUsedIconsOnly() {
  return {
    name: 'mdi-used-icons-only',
    enforce: 'pre',
    apply: 'build',
    transform(code, id) {
      if (!id.includes('materialdesignicons.css')) return null;

      // Both trees: `src-tauri/src` carries the terminal, editor and browser
      // catalogues, and eighteen of their icons are named nowhere else.
      const used = iconsUsed(
        SOURCE_ROOTS.map((dir) => fileURLToPath(new URL(`./${dir}`, import.meta.url))),
        vuetifyIcons
      );
      let seen = 0;
      let kept = 0;

      const trimmed = code.replace(glyphRule(), (rule, name) => {
        seen += 1;
        if (!used.has(`mdi-${name}`)) return '';
        kept += 1;
        return rule;
      });

      // The stylesheet changed shape upstream, or the scan stopped finding
      // anything. Both are silent failures otherwise — the first ships the
      // whole file again while this comment claims it does not, the second
      // ships an application with no icons at all.
      if (seen < 5000) {
        this.error(`mdi-used-icons-only: expected thousands of glyph rules, matched ${seen}`);
      }
      if (kept < 100) {
        this.error(`mdi-used-icons-only: only ${kept} icons survived — the scan found nothing`);
      }

      return { code: trimmed, map: null };
    },
  };
}

export default defineConfig({
  plugins: [
    mdiWoff2Only(),
    mdiUsedIconsOnly(),
    vue(),
    /**
     * No `styles.configFile`.
     *
     * It bought three compile-time tokens — the body font, the root corner
     * radius and the transition duration — and every one of them is now a
     * runtime setting under Appearance, written as a custom property that
     * overrides the compiled value anyway.
     *
     * What it cost was the whole style pipeline: with a config file the plugin
     * rewrites every component's stylesheet to a virtual id
     * (`virtual:plugin-vuetify:components/VBtn/VBtn.sass`) that only it can
     * resolve. When that resolution misses — a restarted server, a cached
     * bundle from another session — the browser gets a 404 for every component
     * at once and the app renders as unstyled boxes on white.
     *
     * Without it, the plugin imports Vuetify's own precompiled CSS: ordinary
     * files, served by Vite like any other.
     */
    vuetify({ autoImport: { labs: true } }),
  ],

  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },

  /**
   * Vuetify is never pre-bundled in dev.
   *
   * Two failures come from letting the optimiser near it, and both take out a
   * whole route rather than a corner of one — a lazy chunk that fails to load
   * aborts the navigation, so the click just does nothing.
   *
   * The first: `vite-plugin-vuetify` rewrites each component's style import to
   * a virtual id (`virtual:plugin-vuetify:…/VTextarea.sass`). Pre-bundling
   * freezes that id into a cached file, and any session where the plugin does
   * not resolve it exactly as before answers 404.
   *
   * The second: the plugin's auto-import runs during transform, after the
   * cold-start scan, so components are discovered one view at a time. Each
   * discovery re-bundles and reloads, and whatever `import()` was in flight
   * fails — which is how the settings page became unreachable.
   *
   * Excluding costs a slower first paint in dev, where the modules are served
   * one by one. It buys a dependency graph that cannot change under a running
   * page. `optimizeDeps` is dev-only; the production build is untouched.
   */
  optimizeDeps: { exclude: ['vuetify'] },

  css: {
    preprocessorOptions: {
      // Vuetify's entry stylesheet is main.sass (indented syntax), so both the
      // scss and sass keys must be configured or half the compilation keeps
      // emitting deprecation noise.
      scss: { api: 'modern-compiler', quietDeps: true },
      sass: { api: 'modern-compiler', quietDeps: true },
    },
  },

  // Tauri expects a fixed port and treats a failure to bind as fatal.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },

  // Match the Rust side's target so we don't ship polyfills the webview
  // (WKWebView / WebView2 / WebKitGTK) does not need.
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      output: {
        manualChunks: { vue: ['vue', 'vue-router', 'pinia', 'vue-i18n'] },
      },
    },
  },
});
