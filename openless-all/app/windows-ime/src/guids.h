#pragma once

#include <msctf.h>
#include <windows.h>

// 唔打啦 uses its own CLSID / profile GUID (not upstream OpenLess's), so both IMEs can be
// installed side by side without overwriting each other's TSF registration.
// {3515ED82-D97F-4298-8C00-2AF7B2137F7A}
inline constexpr CLSID CLSID_MudalaTextService = {
    0x3515ed82,
    0xd97f,
    0x4298,
    {0x8c, 0x00, 0x2a, 0xf7, 0xb2, 0x13, 0x7f, 0x7a},
};

// {D59C795B-7F32-45B6-A321-DAD27CC5FFC3}
inline constexpr GUID GUID_MudalaProfile = {
    0xd59c795b,
    0x7f32,
    0x45b6,
    {0xa3, 0x21, 0xda, 0xd2, 0x7c, 0xc5, 0xff, 0xc3},
};

inline constexpr wchar_t kMudalaImeName[] = L"唔打啦";
// Registered for every language (like the system speech text service) under the
// speech category, so TSF keeps it active next to the user's keyboard IME and
// dictation never has to switch the keyboard IME away and back.
inline constexpr LANGID kOpenLessLangId = 0xFFFF;
// Releases up to 2.0.0-Beta.4 registered a zh-CN keyboard profile instead.
inline constexpr LANGID kOpenLessLegacyKeyboardLangId = 0x0804;
