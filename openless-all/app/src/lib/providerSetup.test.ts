import { areProvidersConfigured, shouldShowProviderSetupPrompt } from './providerSetup';

function assertEqual(actual: boolean, expected: boolean, name: string) {
  if (actual !== expected) {
    throw new Error(`${name}: expected ${expected}, got ${actual}`);
  }
}

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'volcengine',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: true,
      llmConfigured: true,
      omniConfigured: false,
      volcengineConfigured: true,
      arkConfigured: true,
    },
    false,
  ),
  true,
  'configured when ASR and LLM are both ready',
);

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'volcengine',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: false,
      llmConfigured: true,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: true,
    },
    false,
  ),
  false,
  'not configured when ASR provider is missing',
);

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'volcengine',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: true,
      llmConfigured: false,
      omniConfigured: false,
      volcengineConfigured: true,
      arkConfigured: false,
    },
    false,
  ),
  false,
  'not configured when LLM provider is missing',
);

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'whisper',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: true,
      llmConfigured: true,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: true,
    },
    false,
  ),
  true,
  'configured when active ASR is non-volcengine but already ready',
);

assertEqual(
  shouldShowProviderSetupPrompt(
    {
      activeAsrProvider: 'whisper',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: false,
      llmConfigured: false,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: false,
    },
    null,
    false,
  ),
  true,
  'show first-run prompt when providers are missing and no prompt was seen',
);

assertEqual(
  shouldShowProviderSetupPrompt(
    {
      activeAsrProvider: 'whisper',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: false,
      llmConfigured: false,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: false,
    },
    '1',
    false,
  ),
  false,
  'do not repeat first-run prompt after the user has deferred it in this session',
);

assertEqual(
  shouldShowProviderSetupPrompt(
    {
      activeAsrProvider: 'whisper',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: true,
      llmConfigured: true,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: true,
    },
    null,
    false,
  ),
  false,
  'do not show prompt when providers are already configured',
);

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'volcengine',
      activeLlmProvider: 'ark',
      pipelineMode: 'multimodal',
      asrConfigured: false,
      llmConfigured: false,
      omniConfigured: true,
      volcengineConfigured: false,
      arkConfigured: false,
    },
    false,
  ),
  true,
  'multimodal mode only requires the omni model',
);

assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'volcengine',
      activeLlmProvider: 'ark',
      pipelineMode: 'multimodal',
      asrConfigured: true,
      llmConfigured: true,
      omniConfigured: false,
      volcengineConfigured: true,
      arkConfigured: true,
    },
    false,
  ),
  false,
  'multimodal mode ignores traditional ASR/LLM readiness',
);

// M1 offline Cantonese: only the downloaded local model is required; no LLM.
assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'sherpa-onnx-local',
      activeLlmProvider: 'ark',
      pipelineMode: 'traditional',
      asrConfigured: true,
      llmConfigured: false,
      omniConfigured: false,
      volcengineConfigured: false,
      arkConfigured: false,
    },
    true,
  ),
  true,
  'offline build needs only the local SenseVoice model',
);
assertEqual(
  areProvidersConfigured(
    {
      activeAsrProvider: 'sherpa-onnx-local',
      activeLlmProvider: 'ark',
      pipelineMode: 'multimodal',
      asrConfigured: false,
      llmConfigured: true,
      omniConfigured: true,
      volcengineConfigured: true,
      arkConfigured: true,
    },
    true,
  ),
  false,
  'offline build is not ready until the SenseVoice model is downloaded',
);
