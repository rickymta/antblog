/** Tạo slug `a-z0-9-` từ tiêu đề tiếng Việt: bỏ dấu, đ → d. */
export function slugify(input: string, maxLength = 100): string {
  const s = input
    .normalize('NFD')
    .replace(/[̀-ͯ]/g, '')
    .replace(/đ/g, 'd')
    .replace(/Đ/g, 'D')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  return s.slice(0, maxLength).replace(/-+$/g, '');
}

export const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

function countWords(text: string): number {
  return text.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w)).length;
}

/**
 * Thời gian đọc (phút) theo hợp đồng: số từ của nội dung (bỏ thẻ HTML/SVG, khối mã tính 1/2) ÷ 200,
 * làm tròn lên, tối thiểu 1.
 */
export function computeReadingMinutes(markdown: string): number {
  let codeWords = 0;
  let text = markdown.replace(/^(```|~~~)[^\n]*\n[\s\S]*?^\1[ \t]*$/gm, (block) => {
    const inner = block.split('\n').slice(1, -1).join('\n');
    codeWords += countWords(inner);
    return ' ';
  });
  text = text
    .replace(/<svg[\s\S]*?<\/svg>/gi, ' ')
    .replace(/<(script|style)[\s\S]*?<\/\1>/gi, ' ')
    .replace(/<!--[\s\S]*?-->/g, ' ')
    .replace(/<[^>]+>/g, ' ');
  const words = countWords(text) + codeWords / 2;
  return Math.max(1, Math.ceil(words / 200));
}

export function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

const BOT_PATTERN = /bot|crawler|spider|preview/i;
export function isBot(userAgent: string | undefined): boolean {
  return !!userAgent && BOT_PATTERN.test(userAgent);
}
