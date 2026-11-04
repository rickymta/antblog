import { createInstance, type i18n as I18n } from 'i18next';
import type { Locale } from '@tkh/shared';
import { vi, type Messages } from './vi';
import { en } from './en';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation';
    resources: { translation: Messages };
  }
}

/** Cookie lưu ngôn ngữ giao diện (máy chủ đọc để render đúng ngôn ngữ ngay từ HTML đầu tiên). */
export const LOCALE_COOKIE = 'tkh-locale';
export const LOCALES: readonly Locale[] = ['vi', 'en'];
export const DEFAULT_LOCALE: Locale = 'vi';

export function normalizeLocale(value: string | null | undefined): Locale {
  return value === 'en' ? 'en' : 'vi';
}

/**
 * Tạo một instance i18next riêng (không dùng instance toàn cục): máy chủ tạo một bản cho mỗi request,
 * trình duyệt tạo một bản cho cả phiên — tránh request này đổi ngôn ngữ của request khác.
 */
export function createI18n(locale: Locale): I18n {
  const instance = createInstance();
  void instance.init({
    resources: { vi: { translation: vi }, en: { translation: en } },
    lng: locale,
    fallbackLng: DEFAULT_LOCALE,
    supportedLngs: ['vi', 'en'],
    initAsync: false,
    // React đã tự thoát ký tự HTML; để i18next thoát nữa thì tên "A & B" sẽ hiện thành "A &amp; B".
    interpolation: { escapeValue: false },
    returnNull: false,
  });
  return instance;
}
