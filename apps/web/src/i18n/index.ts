import i18n from 'i18next';
import { initReactI18next, useTranslation } from 'react-i18next';
import type { Locale } from '@tkh/shared';
import { readStorage, writeStorage } from '../lib/storage';
import { vi, type Messages } from './vi';
import { en } from './en';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation';
    resources: { translation: Messages };
  }
}

const STORAGE_KEY = 'tkh-locale';
export const LOCALES: readonly Locale[] = ['vi', 'en'];

function initialLocale(): Locale {
  const saved = readStorage(STORAGE_KEY);
  return saved === 'en' || saved === 'vi' ? saved : 'vi';
}

function applyHtmlLang(lng: string) {
  document.documentElement.lang = lng === 'en' ? 'en' : 'vi';
}

void i18n.use(initReactI18next).init({
  resources: { vi: { translation: vi }, en: { translation: en } },
  lng: initialLocale(),
  fallbackLng: 'vi',
  supportedLngs: ['vi', 'en'],
  initAsync: false,
  // React đã tự thoát ký tự HTML; để i18next thoát nữa thì tên "A & B" sẽ hiện thành "A &amp; B".
  interpolation: { escapeValue: false },
  returnNull: false,
});

applyHtmlLang(i18n.language);
i18n.on('languageChanged', (lng) => {
  applyHtmlLang(lng);
  writeStorage(STORAGE_KEY, lng);
});

export function setLocale(locale: Locale): void {
  void i18n.changeLanguage(locale);
}

/** Ngôn ngữ giao diện hiện tại, dạng `Locale` của @tkh/shared (dùng cho pickLocalized / format*). */
export function useLocale(): Locale {
  const { i18n: instance } = useTranslation();
  return instance.resolvedLanguage === 'en' ? 'en' : 'vi';
}

export default i18n;
