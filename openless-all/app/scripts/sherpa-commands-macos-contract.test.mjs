// M1 offline Cantonese: sherpa-onnx is the macOS engine, so every sherpa Tauri command the
// frontend invokes must be registered for macOS — not gated to Windows only.
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const appRoot = fileURLToPath(new URL('..', import.meta.url));

function sourceFiles(directory, files = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) sourceFiles(path, files);
    else if (/\.(ts|tsx)$/.test(entry.name) && !/\.test\.tsx?$/.test(entry.name)) files.push(path);
  }
  return files;
}

const invoked = new Set();
for (const file of sourceFiles(join(appRoot, 'src'))) {
  for (const match of readFileSync(file, 'utf8').matchAll(/'(sherpa_onnx_[a-z_]+)'/g)) {
    invoked.add(match[1]);
  }
}
if (invoked.size === 0) throw new Error('expected the frontend to invoke sherpa_onnx_* commands');

const lib = readFileSync(join(appRoot, 'src-tauri/src/lib.rs'), 'utf8');
for (const command of invoked) {
  const registration = new RegExp(`(#\\[cfg\\(([^\\]]*)\\)\\]\\s*)?commands::${command}\\s*,`).exec(
    lib,
  );
  if (!registration) throw new Error(`${command} is invoked by the frontend but never registered`);
  const gate = registration[2];
  if (gate && gate !== 'sherpa_native') {
    throw new Error(`${command} must be registered on macOS (gated by cfg(${gate}))`);
  }
}

console.log(`sherpa commands registered for macOS: ${invoked.size}`);
