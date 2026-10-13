// offlineModelBootstrap.ts — M2 offline build: fetch the SenseVoice model without a button press.
//
// Progress travels on `sherpa-onnx-asr-download-progress`, which GlobalDownloadProgress
// (top-right overlay) and the Settings → Services model card already render. Downloads are
// resumable, so a launch after an interrupted download continues where it stopped.

import { isTauri } from './ipc';
import { activateLocalAsr, downloadSherpaOnnxAsrModel, getLocalAsrSettings } from './localAsr';
import { OFFLINE_ASR_MODEL, OFFLINE_ASR_PROVIDER_ID } from './offlineLock';

let inflight: Promise<void> | null = null;

/** Start (or join) the background SenseVoice download, then load the model so the first
 * dictation is instant. Resolves once the model is ready; rejects if the download fails. */
export function ensureOfflineModel(): Promise<void> {
  inflight ??= (async () => {
    const settings = await getLocalAsrSettings().catch(() => null);
    // Same mirror rule as the model card: ModelScope has no sherpa-onnx models.
    const mirror =
      settings?.mirror && settings.mirror !== 'modelscope' ? settings.mirror : 'huggingface';
    await downloadSherpaOnnxAsrModel(OFFLINE_ASR_MODEL, mirror);
    try {
      await activateLocalAsr('sherpa_onnx', OFFLINE_ASR_MODEL, OFFLINE_ASR_PROVIDER_ID);
    } catch (error) {
      // The model is installed; dictation loads it on demand if this preload failed.
      console.warn('[offline-model] preload after download failed', error);
    }
    if (isTauri) {
      const { emit } = await import('@tauri-apps/api/event');
      await emit('credentials:changed');
    }
  })().finally(() => {
    inflight = null;
  });
  return inflight;
}
