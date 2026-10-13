import { isTabVisibleOffline } from './offlineLock';

function assertEqual(actual: boolean, expected: boolean, name: string) {
  if (actual !== expected) throw new Error(`${name}: expected ${expected}, got ${actual}`);
}

for (const tab of ['style', 'marketplace', 'translation', 'selectionAsk']) {
  assertEqual(isTabVisibleOffline(tab, true), false, `${tab} hidden offline`);
  assertEqual(isTabVisibleOffline(tab, false), true, `${tab} visible upstream`);
}
for (const tab of ['overview', 'history', 'vocab', 'quickNote', 'corrections']) {
  assertEqual(isTabVisibleOffline(tab, true), true, `${tab} stays offline`);
}

console.log('offline lock navigation tests passed');
