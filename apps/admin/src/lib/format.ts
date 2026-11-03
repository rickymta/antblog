import type { TFunction } from 'i18next';

/** 95 ⇒ "1 giờ 35 phút" / "1 h 35 min"; dưới một giờ ⇒ "35 phút". */
export function formatMinutes(total: number, t: TFunction): string {
  const m = Math.max(0, Math.round(total));
  if (m < 60) return t('common.minutes', { count: m });
  const h = Math.floor(m / 60);
  const rest = m % 60;
  return rest ? t('common.hoursMinutes', { h, m: rest }) : t('common.hours', { h });
}
