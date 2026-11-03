import type { TFunction } from 'i18next';
import type { ReactionCounts } from '@tkh/shared';

/** 95 phút → "1 giờ 35 phút" / "1h 35m"; dưới 1 giờ → "35 phút". */
export function formatDuration(totalMinutes: number, t: TFunction): string {
  const minutes = Math.max(0, Math.round(totalMinutes));
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (h === 0) return t('common.minutes', { count: m });
  if (m === 0) return t('common.durationH', { h });
  return t('common.durationHm', { h, m });
}

export function totalReactions(r: ReactionCounts): number {
  return r.love + r.helpful + r.insightful + r.inspiring;
}

/** Màu chủ đề từ API (#RRGGBB) cộng độ trong suốt dạng hex 2 ký tự; màu lạ ⇒ màu primary. */
export function withAlpha(color: string, alpha: number): string {
  const base = /^#[0-9a-f]{6}$/i.test(color) ? color : '#2563EB';
  const a = Math.round(Math.min(1, Math.max(0, alpha)) * 255)
    .toString(16)
    .padStart(2, '0');
  return `${base}${a}`;
}

export function safeColor(color: string | undefined): string {
  return color && /^#[0-9a-f]{6}$/i.test(color) ? color : '#2563EB';
}
