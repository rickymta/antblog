import i18n from 'i18next';
import { initReactI18next, useTranslation } from 'react-i18next';
import type { Locale } from '@tkh/shared';
import { vi, type Messages } from './vi';
import { en } from './en';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation';
    resources: { translation: Messages };
  }
}

const STORAGE_KEY = 'tkh-admin-locale';

/**
 * Bật giao diện song ngữ (nút VI/EN) — `VITE_ENABLE_I18N=true` lúc build. Mặc định TẮT (tạm ẩn theo yêu cầu):
 * không có nút VI/EN, ngôn ngữ luôn là `vi` (bỏ qua lựa chọn cũ trong localStorage). Tài nguyên tiếng Anh + mã nút
 * vẫn giữ nguyên để bật lại là chạy như cũ.
 */
export const I18N_ENABLED = String(import.meta.env.VITE_ENABLE_I18N ?? '').trim().toLowerCase() === 'true';

function readStoredLocale(): Locale {
  if (!I18N_ENABLED) return 'vi';
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === 'vi' || v === 'en') return v;
  } catch {
    /* storage bị chặn: dùng mặc định */
  }
  return 'vi';
}

const initial = readStoredLocale();
document.documentElement.lang = initial;

void i18n.use(initReactI18next).init({
  resources: { vi: { translation: vi }, en: { translation: en } },
  lng: initial,
  fallbackLng: 'vi',
  interpolation: { escapeValue: false },
  initAsync: false,
});

export function setLocale(locale: Locale): void {
  if (!I18N_ENABLED) return;
  void i18n.changeLanguage(locale);
  document.documentElement.lang = locale;
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    /* bỏ qua */
  }
}

/** Ngôn ngữ giao diện hiện tại, dạng Locale của @tkh/shared (dùng cho format số/ngày). */
export function useLocale(): Locale {
  const { i18n: inst } = useTranslation();
  return inst.language === 'en' ? 'en' : 'vi';
}

export default i18n;
