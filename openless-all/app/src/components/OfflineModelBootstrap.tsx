// OfflineModelBootstrap.tsx — M2 offline build: start the SenseVoice download on launch.
//
// Mounted at the main-window root (next to GlobalDownloadProgress), so it also runs while
// the first-launch permissions onboarding is showing. The download is single-flight and
// resumable; GlobalDownloadProgress renders its progress.

import { useEffect } from 'react';
import { getCredentials, isTauri } from '../lib/ipc';
import { OFFLINE_ONLY } from '../lib/offlineLock';
import { ensureOfflineModel } from '../lib/offlineModelBootstrap';

export function OfflineModelBootstrap() {
  useEffect(() => {
    if (!OFFLINE_ONLY || !isTauri) return;
    void (async () => {
      try {
        // asrConfigured is true once the SenseVoice model is installed.
        if ((await getCredentials()).asrConfigured === true) return;
        await ensureOfflineModel();
      } catch (error) {
        // The main window offers Settings → Services (manual 下載 + error) on failure.
        console.warn('[offline-model] background download failed', error);
      }
    })();
  }, []);
  return null;
}
