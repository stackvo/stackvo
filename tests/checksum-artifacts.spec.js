import { describe, it, expect, beforeAll, afterAll } from 'vitest';
import { createHash } from 'node:crypto';
import { Buffer } from 'node:buffer';
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  checksumLine,
  checksumLines,
  isEntryPoint,
  isRegularFile,
  parseArtifactPaths,
} from '../tools/checksum-artifacts.mjs';

describe('checksumLine', () => {
  it('hashes the bytes and names the file by its basename', () => {
    const buffer = Buffer.from('hello');
    const expected = createHash('sha256').update(buffer).digest('hex');
    expect(checksumLine('/tmp/StackVo_0.2.0_x64-setup.exe', buffer)).toBe(
      `${expected}  StackVo_0.2.0_x64-setup.exe`
    );
  });

  it('drops the runner path, Windows backslashes included', () => {
    const buffer = Buffer.from('hello');
    const line = checksumLine(
      'D:\\a\\stackvo\\stackvo\\src-tauri\\target\\x86_64-pc-windows-msvc\\release\\bundle\\nsis\\StackVo_0.2.0_x64-setup.exe',
      buffer
    );
    expect(line.endsWith('StackVo_0.2.0_x64-setup.exe')).toBe(true);
    expect(line).not.toContain('D:\\');
  });

  it('is two spaces between hash and name, the format `sha256sum -c` reads', () => {
    const line = checksumLine('/tmp/a.dmg', Buffer.from('x'));
    const [hash, rest] = line.split('  ');
    expect(hash).toMatch(/^[0-9a-f]{64}$/);
    expect(rest).toBe('a.dmg');
  });

  it('gives different files different hashes', () => {
    const a = checksumLine('/tmp/a.exe', Buffer.from('one'));
    const b = checksumLine('/tmp/b.exe', Buffer.from('two'));
    expect(a.split('  ')[0]).not.toBe(b.split('  ')[0]);
  });
});

describe('parseArtifactPaths', () => {
  it('parses the JSON array tauri-action prints', () => {
    expect(parseArtifactPaths('["/a.exe","/b.dmg"]')).toEqual(['/a.exe', '/b.dmg']);
  });

  it('refuses an unset ARTIFACT_PATHS rather than silently checksumming nothing', () => {
    expect(() => parseArtifactPaths(undefined)).toThrow('ARTIFACT_PATHS is not set');
    expect(() => parseArtifactPaths('')).toThrow('ARTIFACT_PATHS is not set');
  });

  it('names the value when it parses but is not an array', () => {
    expect(() => parseArtifactPaths('{"not":"an array"}')).toThrow(
      'ARTIFACT_PATHS is not a JSON array'
    );
  });

  it('lets a malformed JSON error speak for itself rather than swallowing it', () => {
    expect(() => parseArtifactPaths('not json')).toThrow();
  });
});

// A layout shaped like what `tauri-action` reports on macOS: one `.dmg`, one
// `.app` (a directory), one `.app.tar.gz`. The `.app` is what release run #6
// tripped over.
describe('isRegularFile and checksumLines', () => {
  let root;
  let dmg;
  let app;
  let tarball;

  beforeAll(() => {
    root = mkdtempSync(join(tmpdir(), 'stackvo-checksums-'));
    dmg = join(root, 'StackVo_0.2.0_aarch64.dmg');
    app = join(root, 'StackVo.app');
    tarball = join(root, 'StackVo.app.tar.gz');
    writeFileSync(dmg, 'dmg bytes');
    mkdirSync(app);
    writeFileSync(join(app, 'Contents'), 'inside');
    writeFileSync(tarball, 'tar bytes');
  });

  afterAll(() => {
    rmSync(root, { recursive: true, force: true });
  });

  it('is true for a file, false for a directory, false for nothing', () => {
    expect(isRegularFile(dmg)).toBe(true);
    expect(isRegularFile(app)).toBe(false);
    expect(isRegularFile(join(root, 'never-built.msi'))).toBe(false);
  });

  it('skips the .app directory instead of dying on it, and keeps the order', () => {
    const lines = checksumLines([dmg, app, tarball]);
    expect(lines).toHaveLength(2);
    expect(lines[0].endsWith('StackVo_0.2.0_aarch64.dmg')).toBe(true);
    expect(lines[1].endsWith('StackVo.app.tar.gz')).toBe(true);
  });

  it('refuses to publish an empty checksums file', () => {
    expect(() => checksumLines([app, join(root, 'never-built.msi')])).toThrow(
      'named no regular file to checksum'
    );
  });
});

describe('isEntryPoint', () => {
  // The round trip on whichever OS runs this — `ci.yml` includes
  // `windows-latest`, where `argv[1]` is `D:\\a\\...` and a pasted `file://`
  // prefix never matched the module URL.
  it('recognises this very file as its own entry point, on this OS', () => {
    const argv1 = fileURLToPath(import.meta.url);
    expect(isEntryPoint(argv1, import.meta.url)).toBe(true);
  });

  it('is false for another script, and for a REPL with no script at all', () => {
    const argv1 = fileURLToPath(import.meta.url);
    expect(isEntryPoint(join(argv1, '..', 'other.mjs'), import.meta.url)).toBe(false);
    expect(isEntryPoint(undefined, import.meta.url)).toBe(false);
  });
});
