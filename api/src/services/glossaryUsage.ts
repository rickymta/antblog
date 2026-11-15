import { proseLines, phraseRegex } from '../lib/contentLint.js';
import { Article, type IArticle, type Language } from '../models/index.js';

export interface TermUsageInput {
  language: Language;
  term: string;
  aliases: string[];
  caseSensitive: boolean;
}

export interface TermUsageSample {
  id: string;
  slug: string;
  title: string;
  status: IArticle['status'];
  occurrences: number;
  /** Đoạn văn quanh chỗ khớp đầu tiên. */
  excerpt: string;
}

export interface TermUsage {
  /** Số bài (mọi trạng thái) có ít nhất một chỗ khớp. */
  articles: number;
  occurrences: number;
  samples: TermUsageSample[];
}

const SAMPLE_LIMIT = 8;
const EXCERPT_RADIUS = 60;

/**
 * Đếm số bài chứa keyword (tên chính + alias) — để biên tập viên ước lượng nguy cơ bắt nhầm TRƯỚC khi lưu thuật ngữ.
 * Chỉ quét văn xuôi (che code, inline code, thẻ HTML, đích liên kết — cùng quy tắc với `content:lint`), nên con số là
 * ước lượng trên: trang đọc còn bỏ qua chữ trong heading và liên kết, và chỉ đánh dấu lần xuất hiện đầu tiên mỗi bài.
 */
export async function termUsage(input: TermUsageInput): Promise<TermUsage> {
  const phrases = [...new Set([input.term, ...input.aliases].map((p) => p.trim()).filter(Boolean))];
  const regexes = phrases.map((p) => phraseRegex(p, input.caseSensitive));
  const articles = await Article.find({ language: input.language })
    .select({ slug: 1, title: 1, status: 1, content: 1 })
    .lean<Pick<IArticle, '_id' | 'slug' | 'title' | 'status' | 'content'>[]>();

  const hits: TermUsageSample[] = [];
  let occurrences = 0;
  for (const a of articles) {
    let count = 0;
    let excerpt = '';
    for (const line of proseLines(a.content ?? '')) {
      if (!line.trim()) continue;
      for (const rx of regexes) {
        rx.lastIndex = 0;
        for (const m of line.matchAll(rx)) {
          count++;
          if (!excerpt) {
            const start = Math.max(0, m.index! - EXCERPT_RADIUS);
            const end = Math.min(line.length, m.index! + m[0].length + EXCERPT_RADIUS);
            excerpt = `${start > 0 ? '…' : ''}${line.slice(start, end).replace(/\s+/g, ' ').trim()}${end < line.length ? '…' : ''}`;
          }
        }
      }
    }
    if (!count) continue;
    occurrences += count;
    hits.push({ id: a._id.toString(), slug: a.slug, title: a.title, status: a.status, occurrences: count, excerpt });
  }
  hits.sort((x, y) => y.occurrences - x.occurrences || x.slug.localeCompare(y.slug));
  return { articles: hits.length, occurrences, samples: hits.slice(0, SAMPLE_LIMIT) };
}
