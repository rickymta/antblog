import type { Locale, Localized } from './types';

const LOCALE_TAG: Record<Locale, string> = { vi: 'vi-VN', en: 'en-US' };

export function pickLocalized(value: Localized | undefined, locale: Locale): string {
  if (!value) return '';
  return value[locale] || value.vi || value.en;
}

/** 24500 → "24,5 N" (vi) / "24.5K" (en). */
export function formatCompact(n: number, locale: Locale): string {
  return new Intl.NumberFormat(LOCALE_TAG[locale], { notation: 'compact', maximumFractionDigits: 1 }).format(n);
}

export function formatNumber(n: number, locale: Locale): string {
  return new Intl.NumberFormat(LOCALE_TAG[locale]).format(n);
}

export function formatDate(iso: string, locale: Locale): string {
  return new Date(iso).toLocaleDateString(LOCALE_TAG[locale], { day: '2-digit', month: '2-digit', year: 'numeric' });
}

/** "2 giờ trước" / "2 hours ago". */
export function formatRelative(iso: string, locale: Locale, now: Date = new Date()): string {
  const diffSec = Math.round((new Date(iso).getTime() - now.getTime()) / 1000);
  const rtf = new Intl.RelativeTimeFormat(LOCALE_TAG[locale], { numeric: 'auto' });
  const abs = Math.abs(diffSec);
  if (abs < 60) return rtf.format(diffSec, 'second');
  if (abs < 3600) return rtf.format(Math.round(diffSec / 60), 'minute');
  if (abs < 86400) return rtf.format(Math.round(diffSec / 3600), 'hour');
  if (abs < 86400 * 30) return rtf.format(Math.round(diffSec / 86400), 'day');
  if (abs < 86400 * 365) return rtf.format(Math.round(diffSec / (86400 * 30)), 'month');
  return rtf.format(Math.round(diffSec / (86400 * 365)), 'year');
}

/** Bỏ dấu tiếng Việt và sinh slug a-z0-9- (khớp luật slug của API). */
export function slugify(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 120);
}
