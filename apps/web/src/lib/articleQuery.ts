import { LEVELS, type ArticleSort, type Level } from '@tkh/shared';

// Đọc bộ lọc danh sách bài/lộ trình từ query string — dùng chung ở máy chủ (trang) và trình duyệt (thanh lọc).

export const ARTICLES_PAGE_SIZE = 24;
export const ARTICLE_SORTS: readonly ArticleSort[] = ['latest', 'path', 'popular', 'trending'];
/** The blog opens with recent posts; ordered series remain available through sort=path. */
export const DEFAULT_ARTICLE_SORT: ArticleSort = 'latest';

export type SearchParamsRecord = Record<string, string | string[] | undefined>;

export function firstParam(v: string | string[] | undefined): string | undefined {
  return Array.isArray(v) ? v[0] : v;
}

export interface ParsedArticleQuery {
  category?: string;
  level?: Level;
  tag?: string;
  q?: string;
  sort: ArticleSort;
  page: number;
}

export function parseLevel(value: string | undefined): Level | undefined {
  return LEVELS.includes(value as Level) ? (value as Level) : undefined;
}

export function parseArticleQuery(sp: SearchParamsRecord, fixedCategory?: string): ParsedArticleQuery {
  const sort = firstParam(sp.sort);
  const page = Number.parseInt(firstParam(sp.page) ?? '1', 10);
  const out: ParsedArticleQuery = {
    sort: ARTICLE_SORTS.includes(sort as ArticleSort) ? (sort as ArticleSort) : DEFAULT_ARTICLE_SORT,
    page: Number.isFinite(page) && page > 0 ? Math.min(page, 10_000) : 1,
  };
  const category = fixedCategory ?? (firstParam(sp.category)?.trim() || undefined);
  if (category) out.category = category;
  const level = parseLevel(firstParam(sp.level));
  if (level) out.level = level;
  const tag = firstParam(sp.tag)?.trim();
  if (tag) out.tag = tag;
  const q = firstParam(sp.q)?.trim();
  if (q) out.q = q.slice(0, 200);
  return out;
}

/** Tham số URL của bộ lọc (không có `page`, không có chủ đề cố định) — để dựng liên kết phân trang/xoá lọc. */
export function articleFilterParams(query: ParsedArticleQuery, fixedCategory?: string): Record<string, string> {
  const p: Record<string, string> = {};
  if (!fixedCategory && query.category) p.category = query.category;
  if (query.level) p.level = query.level;
  if (query.tag) p.tag = query.tag;
  if (query.q) p.q = query.q;
  if (query.sort !== DEFAULT_ARTICLE_SORT) p.sort = query.sort;
  return p;
}

export function hasArticleFilters(query: ParsedArticleQuery, fixedCategory?: string): boolean {
  return Object.keys(articleFilterParams(query, fixedCategory)).length > 0;
}

/**
 * Canonical của trang danh sách: bỏ mọi tham số lọc/sắp xếp/tìm kiếm (các tổ hợp đó trùng nội dung trang gốc),
 * CHỈ giữ `?page=N` (N > 1) khi trang không có bộ lọc nào — mỗi trang phân trang là nội dung riêng.
 */
export function listCanonical(basePath: string, page: number, filtered: boolean): string {
  return !filtered && page > 1 ? `${basePath}?page=${page}` : basePath;
}
