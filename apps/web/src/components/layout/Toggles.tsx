'use client';

import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Moon, Sun } from 'lucide-react';
import { useTheme } from '@tkh/ui';
import { I18N_ENABLED, LOCALES, useLocale, useSetLocale } from '../../i18n';
import { iconBtn } from '../../lib/ui';

/**
 * Nút sáng/tối. Máy chủ không biết theme (localStorage) ⇒ biểu tượng chọn bằng CSS theo class `.dark` có sẵn trên <html>
 * (đúng ngay từ HTML đầu tiên), nhãn cụ thể chỉ gắn sau khi hydrate — không lệch hydrate.
 */
export function ThemeToggle() {
  const { t } = useTranslation();
  const { theme, toggle } = useTheme();
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  const label = !mounted ? t('header.themeToDark') : theme === 'dark' ? t('header.themeToLight') : t('header.themeToDark');
  return (
    <button type="button" onClick={toggle} className={iconBtn} aria-label={label} title={label}>
      <Moon aria-hidden="true" className="h-[18px] w-[18px] dark:hidden" />
      <Sun aria-hidden="true" className="hidden h-[18px] w-[18px] dark:block" />
    </button>
  );
}

/**
 * Nút VI/EN dạng hai nửa; nửa đang chọn nền primary. Đổi ⇒ ghi cookie `tkh-locale` và render lại phía máy chủ.
 * Chỉ hiện khi bật cờ `NEXT_PUBLIC_ENABLE_I18N` (nơi đặt nút kiểm `I18N_ENABLED` để ẩn cả khung bao quanh).
 */
export function LanguageToggle({ className = '' }: { className?: string }) {
  if (!I18N_ENABLED) return null;
  return <LanguageToggleButtons className={className} />;
}

function LanguageToggleButtons({ className }: { className: string }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const { setLocale, pending } = useSetLocale();
  return (
    <div role="group" aria-label={t('header.language')} aria-busy={pending} className={`inline-flex rounded-ui border border-line bg-surface p-0.5 ${className}`}>
      {LOCALES.map((l) => (
        <button
          key={l}
          type="button"
          lang={l}
          aria-pressed={locale === l}
          onClick={() => {
            if (l !== locale) setLocale(l);
          }}
          className={`rounded-[6px] px-2 py-1 text-xs font-bold transition-colors ${
            locale === l ? 'bg-primary text-white' : 'text-muted hover:text-ink'
          }`}
        >
          {/* Tên truy cập chứa đúng chữ nhìn thấy ("VI"), phần giải nghĩa chỉ cho trình đọc màn hình. */}
          {l.toUpperCase()}
          <span className="sr-only"> — {l === 'vi' ? 'Tiếng Việt' : 'English'}</span>
        </button>
      ))}
    </div>
  );
}
