// About → app name + tagline + version.
// "Personalization" was once its own tab but was left with only font size, so it merged into "About".
// 唔打啦 ships without an updater, so there is no "check updates" button.

import { useTranslation } from 'react-i18next';
import { APP_VERSION_LABEL } from '../../lib/appVersion';
import { Card } from '../_atoms';
import { useConservativeLayout, useLayoutStack } from '../../lib/useMobileLayout';

export function AboutSection() {
  const { t } = useTranslation();
  const baseLayoutStack = useLayoutStack();
  const conservativeLayout = useConservativeLayout();
  const compactLayout = baseLayoutStack || conservativeLayout;
  const appIconSize = compactLayout ? 36 : 56;
  return (
    <>
      {/* ─── Version info ─────────────────────── */}
      <Card>
        <div
          className="ol-inline-composite"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: compactLayout ? 4 : 14,
            minWidth: 0,
            flexWrap: 'nowrap',
          }}
        >
          <img
            src="AppIcon.png"
            alt=""
            style={{
              width: appIconSize,
              height: appIconSize,
              flexShrink: 0,
              borderRadius: 13,
              boxShadow: '0 4px 10px rgba(0,0,0,.10), 0 0 0 0.5px rgba(0,0,0,.06)',
            }}
          />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div
              style={{
                fontSize: 17,
                fontWeight: 600,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              唔打啦
            </div>
            <div
              style={{
                fontSize: 12,
                color: 'var(--ol-ink-3)',
                marginTop: 2,
                overflow: 'hidden',
                textOverflow: 'ellipsis',
                whiteSpace: 'nowrap',
              }}
            >
              {t('modal.about.tagline')} · {APP_VERSION_LABEL}
            </div>
          </div>
        </div>
      </Card>

    </>
  );
}
