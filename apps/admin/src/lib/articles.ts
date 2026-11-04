import type { AdminArticle, ArticleStatus } from '@tkh/shared';

/** Trạng thái hiển thị: `published` + `publishedAt` ở tương lai = đã lên lịch (hợp đồng §8.4). */
export type DisplayStatus = ArticleStatus | 'scheduled';

export function isScheduled(a: Pick<AdminArticle, 'status' | 'publishedAt'>, now = Date.now()): boolean {
  return a.status === 'published' && !!a.publishedAt && Date.parse(a.publishedAt) > now;
}

export function displayStatus(a: Pick<AdminArticle, 'status' | 'publishedAt'>, now = Date.now()): DisplayStatus {
  return isScheduled(a, now) ? 'scheduled' : a.status;
}

const pad = (n: number) => String(n).padStart(2, '0');

/** ISO ⇒ giá trị cho `<input type="datetime-local">` theo múi giờ của trình duyệt. */
export function toLocalInput(iso: string | Date): string {
  const d = typeof iso === 'string' ? new Date(iso) : iso;
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** Giá trị `datetime-local` (giờ trình duyệt) ⇒ ISO UTC; rỗng/sai ⇒ undefined. */
export function fromLocalInput(value: string): string | undefined {
  if (!value) return undefined;
  const d = new Date(value);
  return Number.isNaN(d.getTime()) ? undefined : d.toISOString();
}

/** Ngày + giờ theo ngôn ngữ giao diện. */
export function formatDateTime(iso: string, locale: 'vi' | 'en'): string {
  return new Date(iso).toLocaleString(locale === 'vi' ? 'vi-VN' : 'en-US', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function countWords(text: string): number {
  return text.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w)).length;
}

/**
 * Số từ + phút đọc — CÙNG công thức với API (`computeReadingMinutes`): bỏ thẻ HTML/SVG, khối mã tính 1/2, ÷ 200, tối thiểu 1.
 */
export function wordStats(markdown: string): { words: number; minutes: number } {
  let codeWords = 0;
  let text = markdown.replace(/^(```|~~~)[^\n]*\n[\s\S]*?^\1[ \t]*$/gm, (block) => {
    codeWords += countWords(block.split('\n').slice(1, -1).join('\n'));
    return ' ';
  });
  text = text
    .replace(/<svg[\s\S]*?<\/svg>/gi, ' ')
    .replace(/<(script|style)[\s\S]*?<\/\1>/gi, ' ')
    .replace(/<!--[\s\S]*?-->/g, ' ')
    .replace(/<[^>]+>/g, ' ');
  const plain = countWords(text);
  return { words: plain + codeWords, minutes: Math.max(1, Math.ceil((plain + codeWords / 2) / 200)) };
}
