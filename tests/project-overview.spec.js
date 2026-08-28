import { describe, it, expect, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createVuetify } from 'vuetify';
import * as components from 'vuetify/components';
import * as directives from 'vuetify/directives';

/**
 * The Configuration pane's read-only summary of what a project *is*.
 *
 * One row on it is worth a test on its own. "Container path" was the string
 * `/var/www/html`, with no `v-if`, directly under two rows that had one — so
 * every project claimed the PHP web root, including the ones the generator
 * writes `WORKDIR /app` for. The copy button put that path on the clipboard,
 * and a `docker exec … cd` with it fails inside the project's own container.
 *
 * What these hold is the pairing rather than either value: the page has to say
 * what `render_dockerfile` did, and the two runtimes below are the two sides of
 * that dispatch. The Rust side owns the paths; this owns the agreement.
 */

globalThis.visualViewport = undefined;

vi.mock('@/lib/ipc', () => ({
  StackvoError: class extends Error {},
  call: vi.fn(),
  asList: (value) => (Array.isArray(value) ? value : []),
  api: new Proxy({}, { get: () => () => Promise.resolve(null) }),
}));

const { i18n } = await import('@/i18n');
const OverviewPane = (await import('@/components/project/OverviewPane.vue')).default;

const vuetify = createVuetify({ components, directives });

const project = (runtime) => ({
  name: 'shop',
  domain: 'shop.stackvo.loc',
  runtime,
  manifest: { name: 'shop', runtime },
});

const mountPane = (runtime) =>
  mount(
    {
      components: { OverviewPane },
      props: ['project'],
      template: '<v-app><OverviewPane :project="project" /></v-app>',
    },
    { props: { project: project(runtime) }, global: { plugins: [vuetify, i18n] } }
  );

describe('the container path a project page offers', () => {
  it('is the web root for a PHP project', () => {
    const text = mountPane('php').text();
    expect(text).toContain('/var/www/html');
    expect(text).not.toContain('/app');
  });

  // node and the six language runtimes all reach `WORKDIR /app`, by two
  // different generator functions. Both are listed so a change to either one
  // shows up here.
  it.each(['node', 'python', 'go', 'ruby', 'rust', 'bun', 'deno'])(
    'is /app for a %s project, which has no /var/www/html to cd into',
    (runtime) => {
      const text = mountPane(runtime).text();
      expect(text).toContain('/app');
      expect(text).not.toContain('/var/www/html');
    }
  );

  // A project loaded before its runtime is known must not guess `/app`: the
  // manifest contract defaults a missing `runtime` to `php`, and the page has
  // to default the same way or it disagrees with the file on disk.
  it('falls back to the web root when the runtime is not loaded yet', () => {
    expect(mountPane(undefined).text()).toContain('/var/www/html');
  });
});
