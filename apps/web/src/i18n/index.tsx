'use client';

import { useCallback, useEffect, useState, useTransition, type ReactNode } from 'react';
import { useRouter } from 'next/navigation';
import { I18nextProvider, useTranslation } from 'react-i18next';
import type { Locale } from '@tkh/shared';
import { readStorage, writeStorage } from '../lib/storage';
import { I18N_ENABLED } from './config';
import { createI18n, LOCALE_COOKIE, LOCALES, normalizeLocale } from './resources';

export { I18N_ENABLED, LOCALES };

function writeLocaleCookie(locale: Locale) {
  document.cookie = `${LOCALE_COOKIE}=${locale}; Path=/; Max-Age=31536000; SameSite=Lax`;
}

/**
 * Cấp i18next cho Client Components. `locale` do máy chủ đọc từ cookie ⇒ HTML máy chủ và lần hydrate đầu khớp nhau.
 * Bản trước lưu ngôn ngữ ở localStorage: người đã chọn EN mà chưa có cookie ⇒ chuyển một lần sang EN.
 */
export function I18nProvider({ locale, children }: { locale: Locale; children: ReactNode }) {
  const [i18n] = useState(() => createI18n(locale));
  const router = useRouter();

  useEffect(() => {
    if (i18n.language !== locale) void i18n.changeLanguage(locale);
  }, [i18n, locale]);

  useEffect(() => {
    // Cờ song ngữ tắt: luôn `vi`, không chuyển theo lựa chọn cũ.
    if (!I18N_ENABLED) return;
    if (document.cookie.split('; ').some((c) => c.startsWith(`${LOCALE_COOKIE}=`))) return;
    const saved = readStorage(LOCALE_COOKIE);
    if (saved && normalizeLocale(saved) !== locale) {
      writeLocaleCookie(normalizeLocale(saved));
      router.refresh();
    }
    // Chỉ kiểm một lần khi mở trang.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>;
}

/** Ngôn ngữ giao diện hiện tại, dạng `Locale` của @tkh/shared (dùng cho pickLocalized / format*). */
export function useLocale(): Locale {
  const { i18n } = useTranslation();
  return normalizeLocale(i18n.resolvedLanguage ?? i18n.language);
}

/** Đổi ngôn ngữ: Client Components đổi ngay, ghi cookie (1 năm) + localStorage, rồi tải lại phần do máy chủ render. */
export function useSetLocale(): { setLocale: (locale: Locale) => void; pending: boolean } {
  const { i18n } = useTranslation();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const setLocale = useCallback(
    (locale: Locale) => {
      writeLocaleCookie(locale);
      writeStorage(LOCALE_COOKIE, locale);
      document.documentElement.lang = locale;
      void i18n.changeLanguage(locale);
      startTransition(() => router.refresh());
    },
    [i18n, router],
  );
  return { setLocale, pending };
}
