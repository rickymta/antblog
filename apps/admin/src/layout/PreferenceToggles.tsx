import { Moon, Sun } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useTheme } from '@tkh/ui';
import { setLocale, useLocale } from '../i18n';
import { IconButton } from '../components/ui';

/** Nút VI/EN + sáng/tối — dùng ở thanh đầu và màn đăng nhập. */
export function PreferenceToggles() {
  const { t } = useTranslation();
  const locale = useLocale();
  const { theme, toggle } = useTheme();
  const next = locale === 'vi' ? 'en' : 'vi';
  return (
    <div className="flex items-center gap-1">
      <button
        type="button"
        onClick={() => setLocale(next)}
        aria-label={next === 'en' ? t('topbar.switchToEn') : t('topbar.switchToVi')}
        title={next === 'en' ? t('topbar.switchToEn') : t('topbar.switchToVi')}
        className="inline-flex h-9 items-center rounded-ui border border-line px-2.5 text-xs font-semibold text-ink hover:bg-surface-2"
      >
        <span className={locale === 'vi' ? 'text-primary-ink' : 'text-muted'}>VI</span>
        <span className="mx-1 text-muted" aria-hidden>
          /
        </span>
        <span className={locale === 'en' ? 'text-primary-ink' : 'text-muted'}>EN</span>
      </button>
      <IconButton
        icon={theme === 'dark' ? Sun : Moon}
        label={theme === 'dark' ? t('topbar.themeToLight') : t('topbar.themeToDark')}
        onClick={toggle}
      />
    </div>
  );
}
