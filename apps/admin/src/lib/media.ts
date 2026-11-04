import type { MediaItem } from '@tkh/shared';

/** Loại ảnh API nhận (hợp đồng §8.2) — không có SVG. */
export const MEDIA_TYPES = ['image/png', 'image/jpeg', 'image/webp', 'image/gif'] as const;
export const MEDIA_ACCEPT = MEDIA_TYPES.join(',');
export const MEDIA_MAX_BYTES = 5 * 1024 * 1024;

/** Kiểm trước khi gửi (API kiểm lại bằng magic bytes). Trả mã lỗi hoặc undefined. */
export function checkImageFile(file: File): 'type' | 'size' | undefined {
  // Không có loại (một số hệ điều hành) ⇒ để API đoán theo magic bytes.
  if (file.type && !(MEDIA_TYPES as readonly string[]).includes(file.type)) return 'type';
  if (file.size > MEDIA_MAX_BYTES) return 'size';
  return undefined;
}

export function formatBytes(n: number, locale: 'vi' | 'en'): string {
  const fmt = (v: number) => new Intl.NumberFormat(locale === 'vi' ? 'vi-VN' : 'en-US', { maximumFractionDigits: 1 }).format(v);
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${fmt(n / 1024)} KB`;
  return `${fmt(n / 1024 / 1024)} MB`;
}

/** Alt mặc định khi chưa đặt: tên tệp bỏ đuôi, gạch nối thành khoảng trắng. */
export function defaultAlt(item: Pick<MediaItem, 'alt' | 'fileName'>): string {
  if (item.alt?.trim()) return item.alt.trim();
  return item.fileName.replace(/\.[a-z0-9]+$/i, '').replace(/[-_]+/g, ' ').trim();
}

/** `![alt](url)` — bỏ ký tự làm vỡ cú pháp Markdown trong alt. */
export function imageMarkdown(url: string, alt: string): string {
  return `![${alt.replace(/[[\]\n\r]/g, ' ').trim()}](${url})`;
}

export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
