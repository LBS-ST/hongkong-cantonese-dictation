import { OFFLINE_ONLY } from './offlineLock';
import type { CredentialsStatus } from './types';

export const PROVIDER_SETUP_PROMPT_DEFERRED_KEY = 'ol.providerSetupPromptDeferredThisSession';

export function areProvidersConfigured(
  credentials: CredentialsStatus,
  offline: boolean = OFFLINE_ONLY,
): boolean {
  // M1 offline Cantonese: only the local SenseVoice model is needed (no LLM, no cloud keys).
  // The backend reports asrConfigured once the model is downloaded.
  if (offline) return credentials.asrConfigured === true;
  // Multimodal (Omni) mode: only requires the multimodal model to be configured; the legacy
  // ASR/LLM pair takes no part in this mode.
  if (credentials.pipelineMode === 'multimodal') {
    return credentials.omniConfigured === true;
  }
  const asrConfigured = credentials.asrConfigured ?? credentials.volcengineConfigured;
  const llmConfigured = credentials.llmConfigured ?? credentials.arkConfigured;
  return asrConfigured && llmConfigured;
}

export function shouldShowProviderSetupPrompt(
  credentials: CredentialsStatus,
  promptDeferredValue: string | null,
  offline: boolean = OFFLINE_ONLY,
): boolean {
  return !areProvidersConfigured(credentials, offline) && promptDeferredValue !== '1';
}
