/**
 * The contract's named types against the Rust structs that produce them.
 *
 * Suite E in `validate-contracts.mjs` keeps the *list* of commands in step
 * across three files. Nothing read a type's fields: add one to a
 * `#[derive(Serialize)]` struct and forget `contracts/ipc.json`, and the front
 * end is generated a `.d.ts` that does not know the field exists — the editor
 * marks a correct read as an error, and the review counts a payload the app no
 * longer sends. Issue #100.
 *
 * Everything here is pure: it takes the contract as a parsed object and the
 * Rust sources as text, and returns findings. `validate-contracts.mjs` feeds it
 * the real tree; `tests/contract-fields.spec.js` feeds it fixtures in which one
 * thing is wrong at a time, which is the only way to know a check fires.
 *
 * ## Matching a struct to a contract type — the rule
 *
 * The one hard part. A contract type and its struct are not required to share
 * a name — `DnsStatus` is `dns::Status`, `PresetPlan` is `preset::Plan` — and
 * the same short name exists in several modules (`Port` in `engine` and `pkg`),
 * so "look for a struct called that" is neither complete nor safe. What *is*
 * unambiguous is the command that returns the value: the contract names the
 * command and its return type, the Rust function of the same name declares a
 * return type of its own, and suite E already proves the two commands are the
 * same one. So the link is derived, in this order, and the first rule that
 * answers wins:
 *
 *   1. `_rust` on the contract type, written as `module::Name`. An explicit pin
 *      for the cases the rules below cannot reach, and the underscore keeps it
 *      out of the generated `.d.ts` — `generate-types.mjs` treats `_`-prefixed
 *      keys as documentation.
 *   2. A command whose `returns` names the type, paired with the Rust function's
 *      return type; or an `args` entry that names it, paired with the parameter
 *      of the same name. `Result<Vec<Option<T>>>` and `T[]? | null` both mean T.
 *      A return is believed before an argument, and a struct before a
 *      `serde_json::Value` — `project_clone` hand-builds JSON whose prose names
 *      `Manifest`, and `project_requirements_declare` returns the struct.
 *   3. A field of an already-matched pair whose contract side names a type and
 *      whose Rust side names a struct. `Project.ports: Port[]` beside
 *      `ports: Vec<Port>` links the two `Port`s, and the one in `engine` wins
 *      over the one in `pkg` because that is the one `Project` carries.
 *   4. A struct with exactly the same name, when the crate has exactly one.
 *
 * A type none of these reach is reported, not skipped; a type whose only
 * producer returns `serde_json::Value` is *listed* rather than warned about,
 * because it is hand-built by design and a warning that fires on every run
 * until somebody types it is a warning nobody reads.
 */

import { headOf } from './generate-types.mjs';

// ---------------------------------------------------------------- Rust source

/**
 * The one kind of key `contracts/ipc.json` uses for prose inside a type.
 * Deliberately the same predicate `generate-types.mjs` applies, so the two
 * readers agree about what a field is.
 */
const documentation = (field) =>
  field.startsWith('_') || field === '$ref' || /(^|[a-z])[Nn]ote$/.test(field);

/**
 * Split `a, b<c, d>, e` on the commas that are not inside brackets.
 *
 * Both sides need it: a Rust field list carries `BTreeMap<String, String>`, and
 * a serde attribute carries `rename(serialize = "a", deserialize = "b")`.
 */
function splitTopLevel(text, separator = ',') {
  const parts = [];
  let depth = 0;
  let current = '';
  let quote = null;
  for (const ch of text) {
    if (quote) {
      current += ch;
      if (ch === quote) quote = null;
      continue;
    }
    if (ch === '"') quote = ch;
    if ('<([{'.includes(ch)) depth += 1;
    if ('>)]}'.includes(ch)) depth -= 1;
    if (ch === separator && depth === 0) {
      parts.push(current);
      current = '';
    } else current += ch;
  }
  parts.push(current);
  return parts.map((p) => p.trim()).filter(Boolean);
}

/** The text between the brace at `open` and the one that closes it. */
function braced(text, open) {
  let depth = 0;
  for (let i = open; i < text.length; i += 1) {
    if (text[i] === '{') depth += 1;
    else if (text[i] === '}') {
      depth -= 1;
      if (depth === 0) return text.slice(open + 1, i);
    }
  }
  return null;
}

/**
 * One `#[serde(…)]` attribute, as a map.
 *
 * Bare words become `true`; `key = "value"` keeps the string. `rename` in its
 * long form keeps only the serialising spelling, because the wire is what the
 * contract describes.
 */
export function serdeAttribute(text) {
  const out = {};
  for (const entry of splitTopLevel(text)) {
    const long = entry.match(/^(\w+)\s*\((.*)\)$/s);
    if (long) {
      const inner = serdeAttribute(long[2]);
      if (long[1] === 'rename' && inner.serialize) out.rename = inner.serialize;
      else out[long[1]] = inner;
      continue;
    }
    const pair = entry.match(/^(\w+)\s*=\s*"([^"]*)"$/);
    if (pair) out[pair[1]] = pair[2];
    else if (/^\w+$/.test(entry)) out[entry] = true;
  }
  return out;
}

/**
 * A field's name on the wire, under `rename_all`.
 *
 * Only the spellings serde offers, from a snake_case identifier — which is the
 * only shape a Rust field can have without the linter complaining.
 */
export function wireName(field, renameAll) {
  const words = field.split('_').filter(Boolean);
  const cap = (w) => w[0].toUpperCase() + w.slice(1);
  switch (renameAll) {
    case 'camelCase':
      return words.map((w, i) => (i ? cap(w) : w)).join('');
    case 'PascalCase':
      return words.map(cap).join('');
    case 'kebab-case':
      return words.join('-');
    case 'SCREAMING-KEBAB-CASE':
      return words.join('-').toUpperCase();
    case 'SCREAMING_SNAKE_CASE':
    case 'UPPERCASE':
      return field.toUpperCase();
    case 'lowercase':
      return field.toLowerCase();
    default:
      return field;
  }
}

/**
 * A Rust type, reduced to what the wire sees.
 *
 * `Option<T>` is the nullable half of optionality and is recorded; `Vec<T>`,
 * `Box<T>`, `&'static T` and the rest are wrappers the JSON does not show, so
 * they are stripped until a name is left. `path` keeps `crate::dns::Status`'s
 * module so the struct can be found in the right file.
 */
export function rustType(text) {
  let type = text.trim().replace(/,\s*$/, '');
  let optional = false;
  let list = false;
  let map = false;

  for (;;) {
    type = type.replace(/^&\s*(?:'\w+\s+)?(?:mut\s+)?/, '').trim();
    // A slice or an array literal is a list.
    const slice = type.match(/^\[\s*(.*?)\s*(?:;\s*[^\]]+)?\]$/s);
    if (slice) {
      list = true;
      type = slice[1];
      continue;
    }
    const wrapped = type.match(/^([\w:]+)\s*<\s*(.*)\s*>$/s);
    if (!wrapped) break;
    const [, outer, inner] = wrapped;
    const name = outer.split('::').pop();
    if (name === 'Option') optional = true;
    else if (['Vec', 'VecDeque', 'BTreeSet', 'HashSet', 'IndexSet'].includes(name)) list = true;
    else if (['BTreeMap', 'HashMap', 'IndexMap'].includes(name)) {
      map = true;
      type = splitTopLevel(inner).pop();
      continue;
    } else if (name === 'Cow') {
      type = splitTopLevel(inner).pop();
      continue;
    } else if (!['Box', 'Arc', 'Rc'].includes(name)) break;
    type = inner;
  }

  const segments = type.split('::').map((s) => s.trim());
  const name = segments.pop();
  const generic = name.match(/^(\w+)\s*<.*>$/s);
  return {
    name: generic ? generic[1] : name,
    path: segments.filter((s) => s && s !== 'crate'),
    optional,
    list,
    map,
  };
}

/**
 * A struct or enum body as fields.
 *
 * Attributes and doc comments arrive on their own lines before the field, so
 * each top-level comma-separated chunk is read as "some attributes, then
 * `[pub] name: Type`". Anything else — a tuple variant, a unit variant — is
 * not a field and is left out.
 */
function fieldsOf(body) {
  const fields = [];
  for (const chunk of splitTopLevel(body)) {
    const serde = {};
    let cfg = false;
    let rest = chunk;
    for (;;) {
      const attr = rest.match(/^\s*#\[\s*(\w+)\s*(?:\((.*?)\))?\s*\]\s*/s);
      if (!attr) break;
      if (attr[1] === 'serde') Object.assign(serde, serdeAttribute(attr[2] ?? ''));
      if (attr[1] === 'cfg') cfg = true;
      rest = rest.slice(attr[0].length);
    }
    const field = rest.match(/^\s*(?:pub(?:\([^)]*\))?\s+)?(?:r#)?(\w+)\s*:\s*([\s\S]+)$/);
    if (!field) continue;
    fields.push({ name: field[1], type: field[2].trim(), serde, cfg });
  }
  return fields;
}

/** Line comments gone, so a doc comment cannot be read as code. */
const uncommented = (source) =>
  source.replace(/^[ \t]*\/\/[^\n]*$/gm, '').replace(/[ \t]+\/\/[^\n]*$/gm, '');

/**
 * What a file's `use crate::…` lines bring into scope, by the name they are
 * used under: `{ Port: ['engine', 'Port'], manifest: ['manifest'] }`. A bare
 * `Port` in a field or a signature is traced through this to the file that
 * defines it, which is how the `Port` in `engine` is told from the one in `pkg`.
 */
export function rustImports(source) {
  const imports = {};
  const line = /^\s*(?:pub(?:\([^)]*\))?\s+)?use\s+(?:crate|super)::([\w:]+)(?:::\{([^}]*)\})?;/gm;
  for (const m of uncommented(source).matchAll(line)) {
    const base = m[1].split('::');
    if (m[2]) {
      for (const item of splitTopLevel(m[2])) {
        const [original, alias] = item.split(/\s+as\s+/).map((s) => s.trim());
        if (original === 'self') imports[base[base.length - 1]] = base;
        else imports[alias ?? original] = [...base, original];
      }
    } else {
      imports[base[base.length - 1]] = base;
    }
  }
  return imports;
}

/**
 * Every struct and enum in one Rust file that derives `Serialize` or
 * `Deserialize`, with what it puts on the wire.
 *
 * Comments are removed first, because a doc comment is free to contain
 * `#[serde(skip)]` as an example of what not to do, and this reads text, not
 * a syntax tree.
 */
export function rustShapes(source, file = '') {
  const text = uncommented(source);

  const imports = rustImports(text);

  const shapes = [];
  const declaration = /(?:pub(?:\([^)]*\))?\s+)?(struct|enum)\s+(\w+)(?:<[^>{]*>)?\s*([{(;])/y;
  for (const m of text.matchAll(/#\[derive\(([^)]*)\)\]/g)) {
    const derives = m[1];
    const serialize = /\bSerialize\b/.test(derives);
    const deserialize = /\bDeserialize\b/.test(derives);
    if (!serialize && !deserialize) continue;

    // The attributes between the derive and the item, one `#[…]` at a time
    // and by bracket depth rather than by regex — a `[^\n]*` over a long file
    // with hundreds of attributes backtracks for minutes.
    let at = m.index + m[0].length;
    const serde = {};
    for (;;) {
      const gap = text.slice(at).match(/^\s*/)[0].length;
      if (!text.startsWith('#[', at + gap)) break;
      let depth = 0;
      let end = at + gap;
      for (; end < text.length; end += 1) {
        if (text[end] === '[') depth += 1;
        if (text[end] === ']') {
          depth -= 1;
          if (depth === 0) break;
        }
      }
      const attr = text.slice(at + gap, end + 1).match(/^#\[serde\((.*)\)\]$/s);
      if (attr) Object.assign(serde, serdeAttribute(attr[1]));
      at = end + 1;
    }
    declaration.lastIndex = at + text.slice(at).match(/^\s*/)[0].length;
    const item = declaration.exec(text);
    if (!item) continue;

    const shape = {
      name: item[2],
      kind: item[1],
      file,
      line: text.slice(0, m.index).split('\n').length,
      serialize,
      deserialize,
      serde,
      imports,
      fields: [],
    };
    if (item[3] === '{') {
      const body = braced(text, declaration.lastIndex - 1);
      if (body !== null && item[1] === 'struct') {
        shape.fields = fieldsOf(body).map((f) => ({
          ...f,
          wire: f.serde.rename ?? wireName(f.name, serde.rename_all),
          skipped: !!(f.serde.skip || f.serde.skip_serializing),
          flatten: !!f.serde.flatten,
          // Optional on the wire when it can be `null` OR absent. `default`
          // only matters on the way *in*: a struct that is never serialised is
          // an argument, and `default` there means the caller may leave the
          // key out — on a struct that goes out it changes nothing the front
          // end can see.
          optional:
            rustType(f.type).optional ||
            !!f.serde.skip_serializing_if ||
            (!serialize && !!f.serde.default),
          rust: rustType(f.type),
        }));
      }
    }
    shapes.push(shape);
  }
  return shapes;
}

// ---------------------------------------------------------------- Rust commands

/**
 * Every `#[tauri::command]` function: its return type and its parameters.
 *
 * The same rule `contract_agreement.rs` applies — the next `fn` after the
 * attribute — with the signature read up to its opening brace, because Prettier
 * has no say over Rust and rustfmt wraps a long one across lines.
 */
export function rustCommands(source, file = '') {
  const out = {};
  const text = uncommented(source);
  const imports = rustImports(text);
  const attr = /#\[tauri::command(?:\([^)]*\))?\]/g;
  for (const m of text.matchAll(attr)) {
    const after = text.slice(m.index + m[0].length);
    const fn = after.match(/(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(/);
    if (!fn || fn.index > 4000) continue;
    const open = fn.index + fn[0].length - 1;
    let depth = 0;
    let close = open;
    for (; close < after.length; close += 1) {
      if (after[close] === '(') depth += 1;
      if (after[close] === ')') {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    const params = {};
    for (const param of splitTopLevel(after.slice(open + 1, close))) {
      const p = param.match(/^(?:mut\s+)?(\w+)\s*:\s*([\s\S]+)$/);
      if (p) params[p[1]] = p[2].trim();
    }
    const tail = after.slice(close + 1).match(/^\s*->\s*([\s\S]*?)\s*(?:\{|where\b)/);
    out[fn[1]] = { returns: tail ? tail[1].replace(/\s+/g, ' ') : null, params, file, imports };
  }
  return out;
}

// ---------------------------------------------------------------- the contract

/**
 * The contract's spelling of a field, reduced to the two facts compared here:
 * whether it may be absent or null, and which named type it refers to.
 *
 * `string?`, `Port[]`, `Checkout | null`, `TunnelCredentials | null` and
 * `Array<Port>` all appear. A nested object written as an object is a shape of
 * its own and is compared recursively; a JSON array with one object in it is
 * the list-of-shapes spelling `Migration.services` uses.
 */
export function contractField(value, known) {
  if (Array.isArray(value)) {
    const shape = value.find((v) => v && typeof v === 'object');
    return { optional: false, ref: null, inline: shape ?? null, list: true };
  }
  if (value && typeof value === 'object') return { optional: false, ref: null, inline: value };

  const head = headOf(String(value).trim());
  let optional = /\?$/.test(head);
  const parts = splitTopLevel(head.replace(/\?$/, ''), '|').map((p) => p.trim());
  if (parts.includes('null')) optional = true;
  const names = parts
    .filter((p) => p !== 'null')
    .map((p) =>
      p
        .replace(/^Array<(.*)>$/s, '$1')
        .replace(/^Partial<(.*)>$/s, '$1')
        .replace(/\[\]$/, '')
        .trim()
    );
  const ref = names.length === 1 && known.has(names[0]) ? names[0] : null;
  return { optional, ref, inline: null };
}

/**
 * A contract type's fields, `...` spread included.
 *
 * `"...": "every field of Worktree, flattened"` is inheritance written in
 * prose and `generate-types.mjs` reads it as `extends`; here it is the fields
 * themselves, because the wire carries them flat.
 */
export function contractFields(shape, types, seen = new Set()) {
  const out = [];
  for (const [field, value] of Object.entries(shape)) {
    if (field === '...') {
      // Two spellings in the contract: `every field of Worktree, flattened`
      // and a bare `LogFile`. Both mean the same wire.
      const spread =
        typeof value === 'string'
          ? (value.match(/every field of (\w+)/) ?? [null, value.trim()])
          : null;
      const parent = spread && types[spread[1]];
      if (parent && typeof parent === 'object' && !seen.has(spread[1])) {
        seen.add(spread[1]);
        out.push(...contractFields(parent, types, seen));
      }
      continue;
    }
    if (documentation(field) || field === 'oneOf') continue;
    out.push({ name: field, ...contractField(value, new Set(Object.keys(types))) });
  }
  return out;
}

// ---------------------------------------------------------------- linking

/**
 * The struct a Rust type name means, from where it is written.
 *
 * A path (`crate::dns::Status`, `engine::Port`) names the file. A bare name is
 * the struct in the same file, then the one a `use` line brought in, then the
 * only one in the crate by that name. Two files defining it and no path to
 * choose by is an answer this refuses to guess.
 */
export function resolveShape(type, from, shapes) {
  const { name, path } = typeof type === 'string' ? rustType(type) : type;
  const byName = shapes.filter((s) => s.name === name);
  if (!byName.length) return { shape: null, reason: `no struct or enum named ${name}` };

  const inFile = (file) => byName.find((s) => s.file === file);
  if (path.length) {
    const module = path[path.length - 1];
    const hit = inFile(`${module}.rs`);
    return hit ? { shape: hit } : { shape: null, reason: `${module}.rs defines no ${name}` };
  }
  if (from) {
    const local = inFile(from.file);
    if (local) return { shape: local };
    const imported = from.imports?.[name];
    if (imported) {
      const module = imported[imported.length - 2];
      const hit = inFile(`${module}.rs`);
      if (hit) return { shape: hit };
    }
  }
  if (byName.length === 1) return { shape: byName[0] };
  return {
    shape: null,
    reason: `${byName.length} definitions of ${name} (${byName.map((s) => s.file).join(', ')}) and nothing to choose by`,
  };
}

const VALUE = /(^|::)Value$/;

/**
 * Which struct each contract type is, by the rule at the top of this file.
 *
 * Returns the links, the types that could not be linked with the reason, and
 * the ones that are hand-built JSON on the Rust side.
 */
export function linkTypes(contract, shapes, commands) {
  const types = contract.types ?? {};
  const known = new Set(Object.keys(types));
  const links = new Map();
  const handBuilt = new Map();
  const problems = [];

  const isObject = (name) => {
    const t = types[name];
    return (
      t && typeof t === 'object' && !Array.isArray(t) && !(Object.keys(t).length === 1 && t.$ref)
    );
  };

  const link = (name, shape, via) => {
    if (!isObject(name) || !shape) return;
    const existing = links.get(name);
    if (existing && existing.shape !== shape) {
      problems.push({
        type: name,
        code: 'TYPE_CONFLICT',
        message: `${name} is ${existing.shape.file}:${existing.shape.name} via ${existing.via} and ${shape.file}:${shape.name} via ${via} — pin one with "_rust"`,
      });
      return;
    }
    if (!existing) links.set(name, { shape, via });
  };

  // 1. The explicit pin. A pin that names nothing is an error on its own;
  // falling through to the name would then warn about the same type twice.
  const pinned = new Set();
  for (const [name, spec] of Object.entries(types)) {
    if (!isObject(name) || typeof spec._rust !== 'string') continue;
    pinned.add(name);
    if (VALUE.test(spec._rust)) {
      handBuilt.set(name, `pinned as ${spec._rust}`);
      continue;
    }
    const { shape, reason } = resolveShape(spec._rust, null, shapes);
    if (shape) link(name, shape, '_rust');
    else
      problems.push({
        type: name,
        code: 'RUST_PIN_UNRESOLVED',
        message: `"_rust": "${spec._rust}" — ${reason}`,
      });
  }

  // 2. The command that returns it, or takes it.
  const rustSide = (rust, spec) => {
    const out = [];
    if (rust.returns && typeof spec.returns === 'string') {
      const inner = rust.returns.match(/^Result\s*<\s*(.*)\s*>$/s);
      out.push([spec.returns, inner ? splitTopLevel(inner[1])[0] : rust.returns, '()']);
    }
    for (const [arg, text] of Object.entries(spec.args ?? {})) {
      const snake = arg.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`);
      const param = rust.params[snake] ?? rust.params[arg];
      if (param) out.push([text, param, `(${arg})`]);
    }
    return out;
  };
  const pairs = Object.entries(contract.commands ?? {}).flatMap(([command, spec]) =>
    commands[command]
      ? rustSide(commands[command], spec).map(([contractText, rustText, suffix]) => ({
          contractText,
          type: rustType(rustText),
          from: commands[command],
          via: `${command}${suffix}`,
        }))
      : []
  );
  // The same exemptions suite E applies to the command list: a command the
  // contract says lives in the front end, or is deferred with a reason, has no
  // Rust function — so a type only it returns has no struct, by design rather
  // than by omission.
  const notRust = Object.entries(contract.commands ?? {})
    .filter(([, spec]) => spec.kind === 'frontend-plugin' || spec.status === 'deferred')
    .map(([command, spec]) => ({
      contractText: String(spec.returns ?? ''),
      type: null,
      via: `${command}() is ${spec.status === 'deferred' ? 'deferred' : 'a front-end plugin'}`,
      rank: 1,
    }));
  // What comes OUT is what the type table describes, so a return type is
  // believed before an argument type, and a struct before hand-built JSON:
  // `routes_list` assembles a `UserRoute` from `routes::Checked` plus an
  // error branch, and `routes_save` reads the same name back as the smaller
  // `routes::Route` — the row the front end receives is the one to describe.
  const rank = ({ via, type }) => (via.endsWith('()') ? 0 : 2) + (VALUE.test(type.name) ? 1 : 0);
  const ordered = [...pairs.map((p) => ({ ...p, rank: rank(p) })), ...notRust].sort(
    (a, b) => a.rank - b.rank
  );
  for (const { contractText, type, from, via } of ordered) {
    if (!type || VALUE.test(type.name)) {
      // `serde_json::Value` is assembled by hand, and a `returns` written as
      // an inline object can name several contract types inside it — every
      // one of those is hand-built too, and none has a struct to be checked
      // against.
      for (const name of contractText.match(/[A-Z]\w*/g) ?? []) {
        if (isObject(name) && !links.has(name) && !handBuilt.has(name)) {
          handBuilt.set(name, type ? `${via} builds it as serde_json::Value` : via);
        }
      }
      continue;
    }
    const { ref } = contractField(contractText, known);
    if (!ref || !isObject(ref) || links.has(ref) || handBuilt.has(ref)) continue;
    const { shape } = resolveShape(type, from, shapes);
    if (shape) link(ref, shape, via);
  }

  // 3. Fields of what is already linked, until nothing new appears.
  for (let before = -1; before !== links.size;) {
    before = links.size;
    for (const [name, { shape }] of [...links]) {
      const rustFields = new Map(wireFields(shape, shapes).map((f) => [f.wire, f]));
      for (const field of contractFields(types[name], types)) {
        const rust = rustFields.get(field.name);
        if (!field.ref || !rust || links.has(field.ref) || handBuilt.has(field.ref)) continue;
        const { shape: target } = resolveShape(rust.rust, shape, shapes);
        if (!target) continue;
        link(field.ref, target, `${name}.${field.name}`);
      }
    }
  }

  // 4. The same name, when there is exactly one.
  for (const name of Object.keys(types)) {
    if (!isObject(name) || links.has(name) || handBuilt.has(name) || pinned.has(name)) continue;
    const { shape, reason } = resolveShape(name, null, shapes);
    if (shape) link(name, shape, 'its name');
    else problems.push({ type: name, code: 'TYPE_UNMATCHED', message: reason });
  }

  return { links, handBuilt, problems };
}

// ---------------------------------------------------------------- comparing

/**
 * A struct's wire fields, `#[serde(flatten)]` expanded and `skip` removed.
 *
 * A flattened field's own name never reaches the wire; its struct's fields do,
 * so those are what the contract has to list. A flattened type this cannot
 * resolve is reported once, because every field it would have contributed is
 * about to be missing.
 */
export function wireFields(shape, shapes, out = [], seen = new Set()) {
  seen.add(shape);
  for (const field of shape.fields) {
    if (field.skipped) continue;
    if (field.flatten) {
      const { shape: inner, reason } = resolveShape(field.rust, shape, shapes);
      if (!inner) out.push({ ...field, unresolvedFlatten: reason });
      else if (!seen.has(inner)) wireFields(inner, shapes, out, seen);
      continue;
    }
    out.push(field);
  }
  return out;
}

/**
 * One contract type against one struct, field by field.
 *
 * Three questions, each its own code: does the contract list every field the
 * struct serialises, does the struct have every field the contract lists, and
 * do the two agree about whether it can be missing. Nested shapes written
 * inline in the contract recurse into the struct the Rust field names.
 */
export function compareType(name, contractShape, shape, shapes, types, subject = name) {
  const findings = [];
  const where = `${shape.file}:${shape.line} ${shape.name}`;

  if (shape.kind === 'enum') {
    findings.push({
      level: 'warn',
      code: 'TYPE_UNCHECKED',
      subject,
      message: `${where} is an enum; only structs are compared field by field`,
    });
    return findings;
  }

  const rust = wireFields(shape, shapes);
  const contractSide = contractFields(contractShape, types);
  const declared = new Map(contractSide.map((f) => [f.name, f]));
  const serialised = new Map(rust.map((f) => [f.wire, f]));

  for (const field of rust) {
    if (field.unresolvedFlatten) {
      findings.push({
        level: 'error',
        code: 'FLATTEN_UNRESOLVED',
        subject,
        message: `${where} flattens \`${field.name}: ${field.type}\` — ${field.unresolvedFlatten}`,
      });
      continue;
    }
    if (!declared.has(field.wire)) {
      findings.push({
        level: 'error',
        code: 'FIELD_UNDECLARED',
        subject,
        message: `${where} serialises \`${field.wire}\` (${field.type}) and the contract does not list it`,
      });
    }
  }

  for (const field of contractSide) {
    const match = serialised.get(field.name);
    if (!match) {
      findings.push({
        level: 'error',
        code: 'FIELD_PHANTOM',
        subject,
        message: `the contract lists \`${field.name}\` and ${where} has no such field`,
      });
      continue;
    }
    if (field.optional !== match.optional) {
      findings.push({
        level: 'error',
        code: 'FIELD_OPTIONALITY',
        subject,
        message: field.optional
          ? `the contract says \`${field.name}\` may be absent or null; ${where} always sends \`${match.type}\``
          : `${where} sends \`${field.name}\` as \`${match.type}\`, which can be absent or null; the contract says it is always there`,
      });
    }
    if (field.inline) {
      const { shape: inner } = resolveShape(match.rust, shape, shapes);
      if (inner && inner !== shape) {
        findings.push(
          ...compareType(name, field.inline, inner, shapes, types, `${subject}.${field.name}`)
        );
      }
    }
  }

  return findings;
}

// ---------------------------------------------------------------- entry point

/**
 * The whole check: link, then compare.
 *
 * `sources` is `[{ file, text }]` for every Rust file that can carry a
 * command or a shape — flat under `src-tauri/src/`, as the crate is.
 */
export function checkTypeFields(contract, sources) {
  const shapes = sources.flatMap(({ file, text }) => rustShapes(text, file));
  const commands = Object.assign({}, ...sources.map(({ file, text }) => rustCommands(text, file)));
  const types = contract.types ?? {};

  const { links, handBuilt, problems } = linkTypes(contract, shapes, commands);
  const findings = problems.map((p) => ({
    level: p.code === 'RUST_PIN_UNRESOLVED' ? 'error' : 'warn',
    code: p.code,
    subject: p.type,
    message: p.message,
  }));

  for (const [name, { shape }] of links) {
    findings.push(...compareType(name, types[name], shape, shapes, types));
  }

  return {
    findings,
    checked: [...links.keys()].sort(),
    handBuilt: [...handBuilt].sort(),
    links: new Map(
      [...links].map(([name, { shape, via }]) => [
        name,
        { shape: `${shape.file}:${shape.name}`, via },
      ])
    ),
  };
}
