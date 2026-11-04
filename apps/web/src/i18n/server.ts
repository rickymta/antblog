import { cache } from 'react';
import { cookies } from 'next/headers';
import type { TFunction } from 'i18next';
import type { Locale } from '@tkh/shared';
import { createI18n, LOCALE_COOKIE, normalizeLocale } from './resources';

/** Ngôn ngữ giao diện của request hiện tại (cookie `tkh-locale`, mặc định `vi`). */
export const getLocale = cache(async (): Promise<Locale> => normalizeLocale((await cookies()).get(LOCALE_COOKIE)?.value));

/** Hàm dịch phía máy chủ cho request hiện tại (một instance i18next mỗi request). */
export const getServerI18n = cache(async (): Promise<{ locale: Locale; t: TFunction }> => {
  const locale = await getLocale();
  return { locale, t: createI18n(locale).getFixedT(locale) };
});
