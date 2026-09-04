import { describe, it, expect } from 'vitest';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve, join } from 'node:path';
import {
  checkTypeFields,
  contractFields,
  rustCommands,
  rustShapes,
  rustType,
  wireName,
} from '../tools/contract-fields.mjs';

/**
 * Suite H of the contract validator, shown something wrong.
 *
 * `tools/validate-contracts.mjs` keeps three lists of commands in step and,
 * until issue #100, read no type's fields — a field added to a
 * `#[derive(Serialize)]` struct and missed in `contracts/ipc.json` passed every
 * suite. The comparison now lives in `tools/contract-fields.mjs`, and this file
 * is the reason it can be trusted: every finding it can produce is produced
 * here, from a fixture built to produce it, and nothing else.
 *
 * In-memory fixtures rather than edited sources, because a test that breaks a
 * real struct to see the check fire is a test somebody will one day forget to
 * put back. The last block runs the real tree too, so the fixture and the
 * repository cannot quietly become two different programs.
 */

const ROOT = resolve(import.meta.dirname, '..');

/** A contract with one type and one command returning it. */
const contract = (types, commands = {}) => ({ commands, types });

/** The findings for one fixture, as `CODE subject` strings sorted for comparison. */
const codes = (result) => result.findings.map((f) => `${f.code} ${f.subject}`).sort();

const STATUS = `
use serde::Serialize;

/// What is set up and what is not.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub host_name: String,
    /// The file this app writes, where the mechanism is a file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub reload: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stale: Vec<String>,
    #[serde(skip)]
    pub secret: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[tauri::command]
pub async fn dns_status(state: State<'_, AppState>) -> Result<Status> {
    todo!()
}
`;

/** The contract that agrees with STATUS exactly. */
const STATUS_TYPE = {
  hostName: 'string',
  file: 'string? — the file this mechanism writes, where it has one',
  reload: 'string | null',
  stale: 'string[]? — omitted when empty',
  type: 'string',
  note: 'prose that happens to live in an object',
};

describe('reading a struct out of Rust source', () => {
  it('applies rename_all, rename, skip and the two optional spellings', () => {
    const [status] = rustShapes(STATUS, 'dns.rs');
    expect(status.name).toBe('Status');
    expect(status.serialize).toBe(true);
    const wire = Object.fromEntries(
      status.fields.map((f) => [f.wire, { optional: f.optional, skipped: f.skipped }])
    );
    expect(wire).toEqual({
      hostName: { optional: false, skipped: false },
      file: { optional: true, skipped: false },
      reload: { optional: true, skipped: false },
      stale: { optional: true, skipped: false },
      secret: { optional: false, skipped: true },
      type: { optional: false, skipped: false },
    });
  });

  it('spells every rename_all serde offers', () => {
    expect(wireName('host_name', 'camelCase')).toBe('hostName');
    expect(wireName('host_name', 'PascalCase')).toBe('HostName');
    expect(wireName('host_name', 'kebab-case')).toBe('host-name');
    expect(wireName('host_name', 'SCREAMING_SNAKE_CASE')).toBe('HOST_NAME');
    expect(wireName('host_name', 'lowercase')).toBe('host_name');
    expect(wireName('host_name', undefined)).toBe('host_name');
  });

  it('reduces a type to the name the wire sees, and keeps the module path', () => {
    expect(rustType('Option<Vec<crate::git::Checkout>>')).toMatchObject({
      name: 'Checkout',
      path: ['git'],
      optional: true,
      list: true,
    });
    expect(rustType("&'static Provider")).toMatchObject({ name: 'Provider', optional: false });
    expect(rustType('BTreeMap<String, engine::Port>')).toMatchObject({
      name: 'Port',
      path: ['engine'],
      map: true,
    });
  });

  it('does not read a doc comment as code', () => {
    // A comment is free to quote `#[serde(skip)]` as an example. Reading the
    // attribute out of it would skip the field that follows.
    const source = `
#[derive(Serialize)]
pub struct Row {
    /// Never write #[serde(skip)] here — the front end reads it.
    pub kept: String,
}`;
    const [row] = rustShapes(source, 'x.rs');
    expect(row.fields.map((f) => [f.wire, f.skipped])).toEqual([['kept', false]]);
  });

  it('reads `default` as optional only on a struct that is never serialised', () => {
    const source = `
#[derive(Deserialize)]
pub struct Input { #[serde(default)] pub enabled: bool }
#[derive(Serialize, Deserialize)]
pub struct Both { #[serde(default)] pub enabled: bool }
`;
    const [input, both] = rustShapes(source, 'x.rs');
    expect(input.fields[0].optional).toBe(true);
    expect(both.fields[0].optional).toBe(false);
  });

  it('reads a command signature across rustfmt line breaks', () => {
    const source = `
#[tauri::command(async)]
pub async fn project_create(
    app: AppHandle,
    state: State<'_, AppState>,
    spec: manifest::Manifest,
) -> Result<String> {
    todo!()
}`;
    expect(rustCommands(source, 'commands.rs').project_create).toMatchObject({
      returns: 'Result<String>',
      params: { spec: 'manifest::Manifest' },
      file: 'commands.rs',
    });
  });
});

describe('reading a type out of the contract', () => {
  it('finds the named type under every spelling the contract uses', () => {
    const types = { Port: {}, Checkout: {}, T: {} };
    const fields = contractFields(
      {
        ports: 'Port[]',
        git: 'Checkout | null',
        list: 'Array<Port> — newest first',
        maybe: 'Port? (absent when unknown)',
        plain: 'string',
        note: 'documentation',
        _rust: 'documentation too',
      },
      types
    );
    expect(fields.map((f) => [f.name, f.ref, f.optional])).toEqual([
      ['ports', 'Port', false],
      ['git', 'Checkout', true],
      ['list', 'Port', false],
      ['maybe', 'Port', true],
      ['plain', null, false],
    ]);
  });

  it('flattens a `...` spread, in both spellings', () => {
    const types = { LogFile: { id: 'string' }, Worktree: { name: 'string' } };
    expect(
      contractFields({ '...': 'LogFile', project: 'string' }, types).map((f) => f.name)
    ).toEqual(['id', 'project']);
    expect(
      contractFields({ '...': 'every field of Worktree, flattened', exists: 'bool' }, types).map(
        (f) => f.name
      )
    ).toEqual(['name', 'exists']);
  });
});

describe('matching a contract type to its struct', () => {
  it('goes through the command that returns it, whatever the struct is called', () => {
    const result = checkTypeFields(
      contract({ DnsStatus: STATUS_TYPE }, { dns_status: { returns: 'DnsStatus' } }),
      [{ file: 'dns.rs', text: STATUS }]
    );
    expect(result.links.get('DnsStatus')).toEqual({ shape: 'dns.rs:Status', via: 'dns_status()' });
    expect(result.findings).toEqual([]);
  });

  it('chooses the struct the module path names when two share a name', () => {
    const engine = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Port { pub container: u16, pub host: Option<u16> }
#[tauri::command]
pub fn ports() -> Result<Vec<engine::Port>> { todo!() }`;
    const pkg = `
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Port { pub handle: String, pub preferred: u16 }`;
    const result = checkTypeFields(
      contract({ Port: { container: 'u16', host: 'u16?' } }, { ports: { returns: 'Port[]' } }),
      [
        { file: 'engine.rs', text: engine },
        { file: 'pkg.rs', text: pkg },
      ]
    );
    expect(result.links.get('Port').shape).toBe('engine.rs:Port');
    expect(result.findings).toEqual([]);
  });

  it('follows the fields of a matched struct to the types they carry', () => {
    const source = `
use crate::git::Checkout;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project { pub name: String, pub git: Option<Checkout> }
#[tauri::command]
pub fn project_get(name: String) -> Result<Project> { todo!() }`;
    const git = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checkout { pub remote: Option<String>, pub branch: String }`;
    const result = checkTypeFields(
      contract(
        {
          Project: { name: 'string', git: 'Checkout | null' },
          Checkout: { remote: 'string | null' },
        },
        { project_get: { returns: 'Project' } }
      ),
      [
        { file: 'commands.rs', text: source },
        { file: 'git.rs', text: git },
      ]
    );
    expect(result.links.get('Checkout')).toEqual({ shape: 'git.rs:Checkout', via: 'Project.git' });
    // And the struct reached that way is compared like any other.
    expect(codes(result)).toEqual(['FIELD_UNDECLARED Checkout']);
  });

  it('falls back to the name only when the crate has exactly one', () => {
    const one = `
#[derive(Serialize)]
pub struct FinishedEvent { pub ok: bool }`;
    const result = checkTypeFields(contract({ FinishedEvent: { ok: 'bool' } }), [
      { file: 'events.rs', text: one },
    ]);
    expect(result.links.get('FinishedEvent').via).toBe('its name');
    expect(result.findings).toEqual([]);

    const two = `
#[derive(Serialize)]
pub struct Service { pub id: String }`;
    const ambiguous = checkTypeFields(contract({ Service: { id: 'string' } }), [
      { file: 'commands.rs', text: two },
      { file: 'agentctx.rs', text: two },
    ]);
    expect(ambiguous.links.has('Service')).toBe(false);
    expect(ambiguous.findings).toMatchObject([
      { level: 'warn', code: 'TYPE_UNMATCHED', subject: 'Service' },
    ]);
    expect(ambiguous.findings[0].message).toContain('agentctx.rs');
  });

  it('honours an explicit `_rust` pin, and refuses one that names nothing', () => {
    const source = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Raised { pub project: String }`;
    const pinned = checkTypeFields(
      contract({ SupervisorAlarm: { _rust: 'supervisor::Raised', project: 'string' } }),
      [{ file: 'supervisor.rs', text: source }]
    );
    expect(pinned.links.get('SupervisorAlarm')).toEqual({
      shape: 'supervisor.rs:Raised',
      via: '_rust',
    });
    expect(pinned.findings).toEqual([]);

    const dangling = checkTypeFields(
      contract({ SupervisorAlarm: { _rust: 'supervisor::Alarm', project: 'string' } }),
      [{ file: 'supervisor.rs', text: source }]
    );
    expect(dangling.findings).toMatchObject([
      { level: 'error', code: 'RUST_PIN_UNRESOLVED', subject: 'SupervisorAlarm' },
    ]);
  });

  it('lists a hand-built or deferred type instead of warning about it', () => {
    const source = `
#[tauri::command]
pub fn prefs_get() -> Result<serde_json::Value> { todo!() }
#[tauri::command]
pub fn policy_status() -> serde_json::Value { todo!() }`;
    const result = checkTypeFields(
      contract(
        {
          Preferences: { theme: 'string' },
          PolicyImage: { repository: 'string' },
          UpdateInfo: { available: 'bool' },
        },
        {
          prefs_get: { returns: 'Preferences' },
          policy_status: { returns: '{ images: PolicyImage[], active: boolean }' },
          updates_check: { returns: 'UpdateInfo', status: 'deferred' },
        }
      ),
      [{ file: 'commands.rs', text: source }]
    );
    expect(result.findings).toEqual([]);
    expect(result.handBuilt.map(([name]) => name)).toEqual([
      'PolicyImage',
      'Preferences',
      'UpdateInfo',
    ]);
  });

  it('prefers the command returning the struct over one hand-building the same type', () => {
    const source = `
#[derive(Serialize)]
pub struct Manifest { pub name: String }
#[tauri::command]
pub fn project_clone() -> Result<serde_json::Value> { todo!() }
#[tauri::command]
pub fn project_requirements_declare() -> Result<Manifest> { todo!() }`;
    const result = checkTypeFields(
      contract(
        { Manifest: { name: 'string' } },
        {
          // Alphabetically first, and the wrong answer.
          project_clone: { returns: '{ manifest: Manifest }' },
          project_requirements_declare: { returns: 'Manifest' },
        }
      ),
      [{ file: 'commands.rs', text: source }]
    );
    expect(result.links.get('Manifest').via).toBe('project_requirements_declare()');
    expect(result.handBuilt).toEqual([]);
  });

  it('believes what comes out over what goes in', () => {
    // `routes_list` hand-builds each row from `Checked` plus an error branch;
    // `routes_save` reads the same contract name back as the smaller `Route`.
    // Linking through the argument would report the row's own fields as
    // phantoms, so the row wins and the type is listed as hand-built.
    const source = `
#[derive(Deserialize)]
pub struct Route { pub domain: String }
#[tauri::command]
pub fn routes_list() -> Result<Vec<serde_json::Value>> { todo!() }
#[tauri::command]
pub fn routes_save(routes: Vec<crate::routes::Route>) -> Result<Vec<serde_json::Value>> { todo!() }`;
    const result = checkTypeFields(
      contract(
        { UserRoute: { domain: 'string', error: 'string?' } },
        {
          routes_list: { returns: 'UserRoute[]' },
          routes_save: { args: { routes: 'UserRoute[]' }, returns: 'UserRoute[]' },
        }
      ),
      [{ file: 'routes.rs', text: source }]
    );
    expect(result.findings).toEqual([]);
    expect(result.handBuilt).toEqual([
      ['UserRoute', 'routes_list() builds it as serde_json::Value'],
    ]);
  });

  it('says so when a matched type is an enum rather than a struct', () => {
    const source = `
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PtyTarget { Container { name: String }, Host { cwd: Option<String> } }
#[tauri::command]
pub fn pty_open(target: PtyTarget) -> Result<String> { todo!() }`;
    const result = checkTypeFields(
      contract({ PtyTarget: { oneOf: [] } }, { pty_open: { args: { target: 'PtyTarget' } } }),
      [{ file: 'pty.rs', text: source }]
    );
    expect(result.findings).toMatchObject([{ level: 'warn', code: 'TYPE_UNCHECKED' }]);
  });
});

describe('what the comparison catches', () => {
  const run = (type) =>
    checkTypeFields(contract({ DnsStatus: type }, { dns_status: { returns: 'DnsStatus' } }), [
      { file: 'dns.rs', text: STATUS },
    ]);

  it('nothing, when the two agree', () => {
    expect(run(STATUS_TYPE).findings).toEqual([]);
  });

  it('a field the struct serialises and the contract does not list', () => {
    const { hostName: _dropped, ...without } = STATUS_TYPE;
    const result = run(without);
    expect(result.findings).toMatchObject([
      { level: 'error', code: 'FIELD_UNDECLARED', subject: 'DnsStatus' },
    ]);
    expect(result.findings[0].message).toContain('`hostName`');
  });

  it('a field the contract lists and the struct does not have', () => {
    const result = run({ ...STATUS_TYPE, ghost: 'bool' });
    expect(result.findings).toMatchObject([
      { level: 'error', code: 'FIELD_PHANTOM', subject: 'DnsStatus' },
    ]);
    expect(result.findings[0].message).toContain('`ghost`');
  });

  it('a renamed field, from both sides at once', () => {
    // A rename is a field gone and a field arrived. Reporting both is right:
    // the reader cannot tell a rename from two unrelated edits, and either
    // half alone would be a wrong description of the wire.
    const { hostName: _old, ...rest } = STATUS_TYPE;
    const result = run({ ...rest, hostname: 'string' });
    expect(codes(result)).toEqual(['FIELD_PHANTOM DnsStatus', 'FIELD_UNDECLARED DnsStatus']);
  });

  it('a field the contract calls optional that Rust always sends', () => {
    const result = run({ ...STATUS_TYPE, hostName: 'string?' });
    expect(result.findings).toMatchObject([
      { level: 'error', code: 'FIELD_OPTIONALITY', subject: 'DnsStatus' },
    ]);
    expect(result.findings[0].message).toContain('always sends');
  });

  it('a field the contract calls required that Rust can leave out or null', () => {
    for (const [field, spelling] of [
      ['file', 'string'], // skip_serializing_if
      ['reload', 'string'], // Option<T>, sent as null
      ['stale', 'string[]'], // skip_serializing_if = "Vec::is_empty"
    ]) {
      const result = run({ ...STATUS_TYPE, [field]: spelling });
      expect(codes(result), field).toEqual(['FIELD_OPTIONALITY DnsStatus']);
      expect(result.findings[0].message).toContain(`\`${field}\``);
    }
  });

  it('a `#[serde(skip)]` field is not expected in the contract', () => {
    // `secret` is skipped in STATUS and absent from STATUS_TYPE, and the clean
    // run above already passes. This is the other direction: listing it would
    // be a phantom, because nothing ever sends it.
    const result = run({ ...STATUS_TYPE, secret: 'string' });
    expect(codes(result)).toEqual(['FIELD_PHANTOM DnsStatus']);
  });

  it('the fields of a `#[serde(flatten)]` struct, under the outer type', () => {
    const source = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider { pub id: String, pub session_minutes: Option<u32> }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus { #[serde(flatten)] pub provider: &'static Provider, pub has_token: bool }
#[tauri::command]
pub fn tunnel_providers() -> Result<Vec<ProviderStatus>> { todo!() }`;
    const clean = checkTypeFields(
      contract(
        { TunnelProviderStatus: { id: 'string', sessionMinutes: 'number?', hasToken: 'bool' } },
        { tunnel_providers: { returns: 'TunnelProviderStatus[]' } }
      ),
      [{ file: 'tunnel.rs', text: source }]
    );
    expect(clean.findings).toEqual([]);

    // The flattened field's own name is not on the wire.
    const named = checkTypeFields(
      contract(
        { TunnelProviderStatus: { provider: 'object', hasToken: 'bool' } },
        { tunnel_providers: { returns: 'TunnelProviderStatus[]' } }
      ),
      [{ file: 'tunnel.rs', text: source }]
    );
    expect(codes(named)).toEqual([
      'FIELD_PHANTOM TunnelProviderStatus',
      'FIELD_UNDECLARED TunnelProviderStatus',
      'FIELD_UNDECLARED TunnelProviderStatus',
    ]);
  });

  it('a nested shape written inline, against the struct the field names', () => {
    const source = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeOption { pub id: String, pub default: Option<String> }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog { pub runtimes: Vec<RuntimeOption> }
#[tauri::command]
pub fn catalog_get() -> Result<Catalog> { todo!() }`;
    const result = checkTypeFields(
      contract(
        { Catalog: { runtimes: [{ id: 'string', default: 'string' }] } },
        { catalog_get: { returns: 'Catalog' } }
      ),
      [{ file: 'commands.rs', text: source }]
    );
    expect(result.findings).toMatchObject([
      { code: 'FIELD_OPTIONALITY', subject: 'Catalog.runtimes' },
    ]);
  });
});

describe('the repository itself', () => {
  const dir = join(ROOT, 'src-tauri', 'src');
  const sources = readdirSync(dir)
    .filter((f) => f.endsWith('.rs'))
    .map((file) => ({ file, text: readFileSync(join(dir, file), 'utf8') }));
  const ipc = JSON.parse(readFileSync(join(ROOT, 'contracts', 'ipc.json'), 'utf8'));
  const result = checkTypeFields(ipc, sources);

  it('matches most of the contract to a struct, which is what makes the check mean anything', () => {
    // Not the exact count — that moves with every command. A floor well below
    // it, because the failure a source scraper actually has is matching
    // nothing and reporting a clean tree.
    expect(result.checked.length).toBeGreaterThan(100);
    expect(result.handBuilt.length).toBeLessThan(result.checked.length / 5);
  });

  it('agrees with the contract field for field', () => {
    // The gate `node tools/validate-contracts.mjs` applies in CI, stated here
    // so a failing field names itself in the test output rather than only in
    // the validator's table.
    const errors = result.findings.filter((f) => f.level === 'error');
    expect(errors.map((f) => `${f.code} ${f.subject}: ${f.message}`)).toEqual([]);
  });
});
