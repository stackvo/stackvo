import { i18n } from '@/i18n';

/**
 * The prose that comes out of a Rust catalogue, in the reader's language.
 *
 * Five catalogues carry sentences into a bilingual window: `quickcmd.rs` writes
 * what each quick command does (26 of them), `oauth.rs` what each identity
 * provider accepts as a callback URL (7), `tooling.rs` why each required tool
 * is required (4) and what this repository's own two binaries are for, and
 * `provider.rs` one line per shipped recipe plus the five things a person has
 * to change before one will run. Forty-eight sentences, all of them English
 * literals printed raw, so a Turkish user got a translated screen with English
 * inside it.
 *
 * The roadmap counted 39 across three catalogues. Measured, it is 37 across
 * those three — `oauth.rs` carries 7 notes, not 9 — and eleven more in two
 * catalogues it did not look at.
 *
 * `hints.rs` had already written down why this class of string is the worst one
 * to leave untranslated — *"It is the sentence that tells someone what to do"* —
 * and "Run pending migrations." under `php artisan migrate` is that sentence
 * exactly, sitting under a command somebody is about to run in their container.
 *
 * ## Keyed by the id the catalogue already has
 *
 * No second name to typo and no mapping to keep level: `Spec.id` is stable —
 * it is what the IPC surface sends and what the tests name — so it is the key.
 * `hint_translations.rs` holds the two sides equal from the Rust end, failing
 * on a row nobody translated and on a translation nothing offers any more.
 *
 * ## English is still carried, and still wins nothing
 *
 * The catalogue keeps its English and the back end keeps sending it: that is
 * what the CLI prints, what an MCP client reads, and what the log records. Here
 * it is the fallback, for the one case the gate cannot prevent — an older
 * back end offering a row this build's locales have never heard of.
 */
export function catalogueText(namespace, id, english) {
  if (!id) return english ?? '';

  const key = `${namespace}.${id}`;
  const { t, te } = i18n.global;
  return te(key) ? t(key) : (english ?? '');
}

/** What a quick command does, under the command itself. */
export const quickCommandAbout = (command) =>
  catalogueText('quickCommands', command?.id, command?.about);

/** What an identity provider accepts as a callback URL. */
export const oauthNote = (provider) => catalogueText('oauthNotes', provider?.id, provider?.note);

/** Why a required tool is required. */
export const toolingWhy = (tool) => catalogueText('toolingWhy', tool?.id, tool?.why);

/** What one of this repository's own binaries is for. */
export const toolingOwnAbout = (own) => catalogueText('toolingOwn', own?.id, own?.about);

/** One line describing a shipped provider recipe, keyed by its name. */
export const providerRecipeAbout = (recipe) =>
  catalogueText('providerRecipes', recipe?.name, recipe?.about);

/**
 * One thing a recipe needs changed.
 *
 * Takes the `{ key, english }` pair `provider.rs` sends rather than a bare
 * string, which is what this used to be — see the module note above, and
 * `hints.rs`, whose shape this is.
 */
export const providerRecipeEdit = (edit) =>
  catalogueText('providerRecipeEdits', edit?.key, edit?.english);
