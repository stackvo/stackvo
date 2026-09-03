import { describe, it, expect } from 'vitest';
import { createHash } from 'node:crypto';
import { Buffer } from 'node:buffer';
import { checksumLine, parseArtifactPaths } from '../tools/checksum-artifacts.mjs';

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
