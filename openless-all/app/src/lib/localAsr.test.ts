import { LOCAL_ASR_KEEP_LOADED_OPTIONS, isLocalAsrModelSupportedOnOs } from './localAsr';

function assertEqual(actual: boolean, expected: boolean, name: string) {
  if (actual !== expected) {
    throw new Error(`${name}: expected ${expected}, got ${actual}`);
  }
}

const qwen = { runtime: 'generic', family: 'qwen3' } as const;
const whisper = { runtime: 'generic', family: 'whisper' } as const;
const foundry = { runtime: 'foundry', family: 'whisper' } as const;
const sherpa = { runtime: 'sherpa_onnx', family: 'qwen3_asr' } as const;
for (const os of ['mac', 'win', 'android'] as const) {
  assertEqual(isLocalAsrModelSupportedOnOs(qwen, os), os === 'mac', `generic Qwen on ${os}`);
  assertEqual(isLocalAsrModelSupportedOnOs(whisper, os), os === 'mac', `whisper.cpp on ${os}`);
  assertEqual(isLocalAsrModelSupportedOnOs(foundry, os), os === 'win', `Foundry Whisper on ${os}`);
  // M1 offline Cantonese: the sherpa-onnx runtime is built for macOS as well as Windows.
  assertEqual(
    isLocalAsrModelSupportedOnOs(sherpa, os),
    os === 'win' || os === 'mac',
    `Sherpa Qwen on ${os}`,
  );
}
assertEqual(
  isLocalAsrModelSupportedOnOs({ runtime: 'generic', family: 'unknown' }, 'mac'),
  false,
  'unknown models do not default to available',
);

const keepLoadedSeconds = LOCAL_ASR_KEEP_LOADED_OPTIONS.map((option) => option.seconds);
if (JSON.stringify(keepLoadedSeconds) !== JSON.stringify([0, 60, 300, 1800, 86400])) {
  throw new Error(`unexpected keep-loaded options: ${keepLoadedSeconds.join(', ')}`);
}
