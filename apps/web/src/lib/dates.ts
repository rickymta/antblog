import type { Locale } from '@tkh/shared';

const LOCALE_TAG: Record<Locale, string> = { vi: 'vi-VN', en: 'en-US' };

/**
 * Ngày dạng dd/mm/yyyy theo GIỜ VIỆT NAM cố định — máy chủ (Docker, UTC) và trình duyệt cho cùng một chuỗi,
 * nên không lệch hydrate (bản `formatDate` của @tkh/shared theo múi giờ của máy đang chạy).
 */
export function formatDay(iso: string, locale: Locale): string {
  return new Date(iso).toLocaleDateString(LOCALE_TAG[locale], {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    timeZone: 'Asia/Ho_Chi_Minh',
  });
}
