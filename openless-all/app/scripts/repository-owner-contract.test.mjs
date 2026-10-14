import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = fileURLToPath(new URL('../../..', import.meta.url));
const currentRepo = 'Open-Less/openless';
const legacyRepo = 'appergb/openless';

const repositoryReferences = [
  '.github/ISSUE_TEMPLATE/config.yml',
  '.github/workflows/android-apk.yml',
  '.github/workflows/release-tauri.yml',
  'Casks/openless.rb',
  'README.md',
  'README.zh.md',
  'USAGE.md',
  'openless-all/app/src-tauri/src/android/updater_logic.rs',
  'openless-all/app/src-tauri/src/commands/mod.rs',
  'openless-all/app/src-tauri/src/commands/settings.rs',
  'openless-all/app/src-tauri/tauri.conf.json',
  'openless-all/app/src/components/SettingsModal.tsx',
  'openless-all/app/src/pages/settings/AboutSection.tsx',
  'openless-all/app/scripts/write-updater-manifest.mjs',
];

for (const relativePath of repositoryReferences) {
  const content = await readFile(join(repoRoot, relativePath), 'utf8');
  assert(
    !content.includes(legacyRepo),
    `${relativePath} still references the pre-transfer repository`,
  );
}

const tauriConfig = JSON.parse(
  await readFile(join(repoRoot, 'openless-all/app/src-tauri/tauri.conf.json'), 'utf8'),
);
// 唔打啦 ships without the desktop updater: it must never fetch upstream OpenLess releases.
assert.equal(tauriConfig?.plugins?.updater, undefined, 'desktop updater must stay removed');

console.log('repository-owner-contract.test.mjs passed');
