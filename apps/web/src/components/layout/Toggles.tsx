import { useTranslation } from 'react-i18next';
import { Moon, Sun } from 'lucide-react';
import { useTheme } from '@tkh/ui';
import { LOCALES, setLocale, useLocale } from '../../i18n';
import { iconBtn } from '../../lib/ui';

export function ThemeToggle() {
  const { t } = useTranslation();
  const { theme, toggle } = useTheme();
  const toDark = theme !== 'dark';
  return (
    <button type="button" onClick={toggle} className={iconBtn} aria-label={toDark ? t('header.themeToDark') : t('header.themeToLight')} title={toDark ? t('header.themeToDark') : t('header.themeToLight')}>
      {toDark ? <Moon aria-hidden="true" className="h-[18px] w-[18px]" /> : <Sun aria-hidden="true" className="h-[18px] w-[18px]" />}
    </button>
  );
}

/** Nút VI/EN dạng hai nửa; nửa đang chọn nền primary. */
export function LanguageToggle({ className = '' }: { className?: string }) {
  const { t } = useTranslation();
  const locale = useLocale();
  return (
    <div role="group" aria-label={t('header.language')} className={`inline-flex rounded-ui border border-line bg-surface p-0.5 ${className}`}>
      {LOCALES.map((l) => (
        <button
          key={l}
          type="button"
          lang={l}
          aria-pressed={locale === l}
          aria-label={l === 'vi' ? 'Tiếng Việt' : 'English'}
          onClick={() => setLocale(l)}
          className={`rounded-[6px] px-2 py-1 text-xs font-bold transition-colors ${
            locale === l ? 'bg-primary text-white' : 'text-muted hover:text-ink'
          }`}
        >
          {l.toUpperCase()}
        </button>
      ))}
    </div>
  );
}
