import { describe, it, expect } from 'vitest';
import { inspect, osOf, archOf, collect, FORMATS } from '../tools/check-installers.mjs';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

/**
 * The judgement half of `npm run installers:check`, exercised without a bundler.
 *
 * §3 #22's open question is whether the bundler produces a package on ARM, and
 * nothing on this machine can answer it — there is no ARM Linux runner here and
 * no Windows one at all. What can be settled here is the other half: whether a
 * run that *did* bundle on ARM would be recognised as having done so, and
 * whether the three ways it can go wrong quietly are caught.
 *
 * Each case below is a real outcome of the release matrix, not a shape invented
 * to have something to assert.
 */

/** What `ubuntu-24.04-arm` leaves behind when everything worked. */
function linuxArm() {
  return [
    'deb/StackVo_0.1.0_arm64/data/usr/bin/stackvo-desktop',
    'deb/StackVo_0.1.0_arm64.deb',
    'deb/StackVo_0.1.0_arm64.deb.sig',
    'rpm/StackVo-0.1.0-1.aarch64.rpm',
    'rpm/StackVo-0.1.0-1.aarch64.rpm.sig',
    'appimage/StackVo_0.1.0_aarch64.AppImage',
    'appimage/StackVo_0.1.0_aarch64.AppImage.sig',
  ];
}

/** What `windows-11-arm` leaves behind when everything worked. */
function windowsArm() {
  return [
    'msi/StackVo_0.1.0_arm64_en-US.msi',
    'msi/StackVo_0.1.0_arm64_en-US.msi.sig',
    'nsis/StackVo_0.1.0_arm64-setup.exe',
    'nsis/StackVo_0.1.0_arm64-setup.exe.sig',
  ];
}

describe('what an ARM row owes', () => {
  it('passes a Linux aarch64 run that produced all three formats', () => {
    expect(inspect(linuxArm(), { triple: 'aarch64-unknown-linux-gnu' })).toEqual([]);
  });

  it('passes a Windows aarch64 run that produced both formats', () => {
    expect(inspect(windowsArm(), { triple: 'aarch64-pc-windows-msvc' })).toEqual([]);
  });

  /**
   * The one the run page cannot show you.
   *
   * `bundle.targets` is `"all"`, and each format is a different bundler with
   * different native tools — the AppImage one downloads
   * `linuxdeploy-aarch64.AppImage` and executes it, which is precisely the step
   * that has no equivalent on the x86 rows. A `bundle/` directory that exists
   * and holds a `.deb` looks, from an artifact listing, exactly like a
   * directory that holds all three.
   */
  it('refuses a Linux run that produced no AppImage', () => {
    const files = linuxArm().filter((file) => !file.includes('appimage/'));
    const problems = inspect(files, { triple: 'aarch64-unknown-linux-gnu' });

    expect(problems.join(' ')).toContain('no appimage package');
    expect(problems.join(' ')).toContain('separate bundler');
  });

  it('refuses a Windows run that produced no MSI', () => {
    const files = windowsArm().filter((file) => !file.includes('msi/'));
    expect(inspect(files, { triple: 'aarch64-pc-windows-msvc' }).join(' ')).toContain(
      'no msi package'
    );
  });

  /**
   * The failure this whole file is for.
   *
   * A green ARM job that produced an x86 installer answers #22 with a yes and
   * means no. It is not hypothetical: the matrix passes `--target` to three
   * separate commands, and a bundler that never received it builds for the
   * host and names the file accordingly.
   */
  it('refuses an ARM run whose packages are named for x86', () => {
    const files = [
      'deb/StackVo_0.1.0_amd64.deb',
      'rpm/StackVo-0.1.0-1.x86_64.rpm',
      'appimage/StackVo_0.1.0_amd64.AppImage',
      'appimage/StackVo_0.1.0_amd64.AppImage.sig',
    ];
    const problems = inspect(files, { triple: 'aarch64-unknown-linux-gnu' });

    // All three, not the first: a run that reports one wrong package and stops
    // is a run somebody spends twice.
    expect(problems).toHaveLength(3);
    expect(problems.join(' ')).toContain('arm64');
    expect(problems.join(' ')).toContain('aarch64');
    expect(problems.join(' ')).toContain('built for the host instead of the target');
  });

  /**
   * Each bundler spells the same architecture its own way, and the table is
   * read out of `tauri-bundler` rather than chosen here. Getting this wrong in
   * the tidy direction — one word for all of them — turns the check above into
   * one that fails every real ARM release.
   */
  it('knows that dpkg says arm64 where rpm says aarch64', () => {
    expect(FORMATS.deb.arch.aarch64).toBe('arm64');
    expect(FORMATS.rpm.arch.aarch64).toBe('aarch64');
    expect(FORMATS.appimage.arch.aarch64).toBe('aarch64');
    expect(FORMATS.msi.arch.aarch64).toBe('arm64');
    expect(FORMATS.nsis.arch.aarch64).toBe('arm64');
    expect(FORMATS.dmg.arch.aarch64).toBe('aarch64');
  });
});

describe('a run that asked for less', () => {
  /**
   * `--windows-bundle` asks tauri for `--bundles nsis`, because the MSI bundler
   * is `#[cfg(target_os = "windows")]` and cannot run off Windows at all.
   * Without `--only`, this file failed that run for not producing something
   * nobody had asked it to — which is a checker reporting its own wiring as a
   * defect in the build.
   */
  it('owes only what the run asked for', () => {
    const files = ['nsis/StackVo_0.1.0_x64-setup.exe'];
    expect(
      inspect(files, { triple: 'x86_64-pc-windows-msvc', signed: false, only: ['nsis'] })
    ).toEqual([]);
  });

  it('still refuses a narrowed run that did not produce even that', () => {
    expect(
      inspect([], { triple: 'x86_64-pc-windows-msvc', signed: false, only: ['nsis'] })
    ).toHaveLength(1);
  });

  /**
   * A restriction that matches nothing would pass everything, which is the one
   * way this flag could make the check weaker than no flag at all.
   */
  it('refuses a format this platform does not have', () => {
    expect(() =>
      inspect([], { triple: 'x86_64-pc-windows-msvc', only: ['appimage'] })
    ).toThrow(/has no such format/);
  });

  /**
   * The updater's artifact on Windows is the NSIS installer, and on macOS it is
   * a tarball the `--only dmg` caller never asked for. Demanding it in a
   * narrowed run says something false about the release rather than about the
   * run.
   */
  it('does not ask for the updater artifact in a narrowed run', () => {
    expect(
      inspect(['dmg/StackVo_0.1.0_aarch64.dmg'], {
        triple: 'aarch64-apple-darwin',
        only: ['dmg'],
      })
    ).toEqual([]);
  });
});

describe('the signature the updater needs', () => {
  it('refuses an AppImage with no signature beside it', () => {
    const files = linuxArm().filter((file) => !file.endsWith('.AppImage.sig'));
    expect(inspect(files, { triple: 'aarch64-unknown-linux-gnu' }).join(' ')).toContain(
      "refuses it on the user's machine"
    );
  });

  /**
   * A rehearsal builds every target and publishes nothing, so it is run with
   * `--no-sign` — there is no private key on a repository that has not decided
   * where one lives, and an empty `TAURI_SIGNING_PRIVATE_KEY` is worse than an
   * absent one: it gets past tauri's "no private key" guard and fails while
   * decoding, *after* the bundles are already on disk.
   */
  it('does not ask for signatures when the run was told it is unsigned', () => {
    const files = linuxArm().filter((file) => !file.endsWith('.sig'));
    expect(inspect(files, { triple: 'aarch64-unknown-linux-gnu', signed: false })).toEqual([]);
  });

  it('still asks for the packages themselves in an unsigned run', () => {
    expect(inspect([], { triple: 'aarch64-pc-windows-msvc', signed: false })).toHaveLength(2);
  });
});

describe('the targets it knows', () => {
  it('reads the platform and the architecture out of the triple', () => {
    expect(osOf('aarch64-unknown-linux-gnu')).toBe('linux');
    expect(osOf('aarch64-pc-windows-msvc')).toBe('windows');
    expect(osOf('x86_64-apple-darwin')).toBe('macos');
    expect(archOf('aarch64-apple-darwin')).toBe('aarch64');
    expect(archOf('x86_64-unknown-linux-gnu')).toBe('x86_64');
  });

  /**
   * A seventh target added to the matrix has to widen this file rather than
   * pass through it. The way a hand-written table goes stale is by saying
   * nothing.
   */
  it('refuses to guess at a target it has no name for', () => {
    expect(() => osOf('x86_64-unknown-freebsd')).toThrow(/no idea what platform/);
    expect(() => archOf('armv7-unknown-linux-gnueabihf')).toThrow(/only knows what aarch64/);
  });
});

describe('every row of the release matrix', () => {
  const workflow = readFileSync(resolve(import.meta.dirname, '..', '.github/workflows/release.yml'), 'utf8');
  const targets = [...workflow.matchAll(/^\s*-?\s*target:\s*(\S+)\s*$/gm)].map((m) => m[1]);

  it('is six, and this file knows every one of them', () => {
    expect(targets).toHaveLength(6);
    for (const triple of targets) {
      expect(() => inspect([], { triple })).not.toThrow();
    }
  });

  /**
   * The two rows §3 #22 is about. Named rather than merely counted, because
   * "six targets" stays true when one ARM row is swapped for a second x86 one,
   * and that swap is the quiet way this item gets closed without being done.
   */
  it('includes both ARM targets', () => {
    expect(targets).toContain('aarch64-unknown-linux-gnu');
    expect(targets).toContain('aarch64-pc-windows-msvc');
  });
});

describe('reading a bundle directory', () => {
  it('is empty for a directory that is not there, rather than throwing', () => {
    expect(collect(join(tmpdir(), 'stackvo-no-such-bundle-dir'))).toEqual([]);
  });

  /**
   * `.app` is a directory that the bundler treats as one file, and descending
   * into it would list a few thousand resources to say one thing.
   */
  it('stops at a .app instead of walking into it', () => {
    const root = mkdtempSync(join(tmpdir(), 'stackvo-bundle-'));
    mkdirSync(join(root, 'macos', 'StackVo.app', 'Contents'), { recursive: true });
    writeFileSync(join(root, 'macos', 'StackVo.app', 'Contents', 'Info.plist'), '');
    mkdirSync(join(root, 'dmg'), { recursive: true });
    writeFileSync(join(root, 'dmg', 'StackVo_0.1.0_aarch64.dmg'), '');

    expect(collect(root).sort()).toEqual([
      'dmg/StackVo_0.1.0_aarch64.dmg',
      'macos/StackVo.app',
    ]);
  });
});
