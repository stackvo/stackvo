import './styles/global.css';
import './styles/project-panes.css';
import './styles/settings-panes.css';
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import router from './router';
import vuetify from './plugins/vuetify';
import { i18n } from './i18n';
import App from './App.vue';

const app = createApp(App);

// Uncaught component errors go to the console with a trace. A user-facing
// snackbar lands with the feedback composable in Phase 2, alongside the
// mutating commands that can actually fail mid-action.
//
// And they are also kept, which is three lines and closes a real hole. An error
// thrown in `App.vue`'s `setup()` is caught here, printed, and the component
// renders a comment node — so `#app` is empty and the reason is in a console
// nothing reads afterwards. `tests/driver/boot.driver.js` runs the built
// application in the real webview and could only report *"the app root never
// rendered any children"*, four times, with the cause one frame away and
// unreachable. A console line is not a diagnostic if the only thing that can
// fail is a program that cannot read consoles.
//
// Bounded at ten and holding strings rather than `Error` objects: this must not
// become a leak, and it must not keep a component instance alive through a
// reference in a stack. Nothing sends it anywhere — `PRIVACY.md` is the promise
// and this stays in memory, readable by a driver attached to this window.
const RECENT_ERRORS = [];
globalThis.__STACKVO_ERRORS__ = RECENT_ERRORS;

app.config.errorHandler = (err, _instance, info) => {
  console.error('[Vue error]', err, info);
  RECENT_ERRORS.push(`${info}: ${err?.stack || err?.message || String(err)}`);
  if (RECENT_ERRORS.length > 10) RECENT_ERRORS.shift();
};

app.use(createPinia());
app.use(router);
app.use(i18n);
app.use(vuetify);

app.mount('#app');
