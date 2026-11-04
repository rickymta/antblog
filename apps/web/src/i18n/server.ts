import { cache } from 'react';
import { cookies } from 'next/headers';
import type { TFunction } from 'i18next';
import type { Locale } from '@tkh/shared';
import { I18N_ENABLED } from './config';
import { createI18n, DEFAULT_LOCALE, LOCALE_COOKIE, normalizeLocale } from './resources';

/**
 * Ngôn ngữ giao diện của request hiện tại. Cờ song ngữ tắt ⇒ luôn `vi` và KHÔNG gọi `cookies()` (gọi là biến mọi
 * trang thành trang động `no-store`). Cờ bật ⇒ cookie `tkh-locale`, mặc định `vi`.
 */
export const getLocale = cache(async (): Promise<Locale> => {
  if (!I18N_ENABLED) return DEFAULT_LOCALE;
  return normalizeLocale((await cookies()).get(LOCALE_COOKIE)?.value);
});

/** Hàm dịch phía máy chủ cho request hiện tại (một instance i18next mỗi request). */
export const getServerI18n = cache(async (): Promise<{ locale: Locale; t: TFunction }> => {
  const locale = await getLocale();
  return { locale, t: createI18n(locale).getFixedT(locale) };
});
