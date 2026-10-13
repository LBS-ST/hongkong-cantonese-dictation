// offlineLock.ts — M1 offline Cantonese build (see repo-root SPEC.md §5).
//
// Mirrors openless-core `offline_lock`: the only speech engine is the local sherpa-onnx
// SenseVoice model, the language defaults to Cantonese (yue) with a manual zh / en switch,
// and LLM polish is off. Cloud ASR / LLM / multimodal providers are hidden from the UI
// (their code is kept; M1 = disable + hide).

export const OFFLINE_ONLY = true;

export const OFFLINE_ASR_PROVIDER_ID = 'sherpa-onnx-local';
export const OFFLINE_ASR_MODEL = 'sense-voice-small-zh';
export const OFFLINE_LANGUAGES = ['yue', 'zh', 'en'] as const;
export const OFFLINE_DEFAULT_LANGUAGE = 'yue';
