import { createLocalStore, isRecord, useLocalStore } from './localStore';

/**
 * Tiến độ đọc và bài đã lưu — chỉ ở máy người đọc (localStorage), không gửi lên máy chủ.
 * Một bài được coi là "đã đọc" khi đạt cùng ngưỡng với lượt xem (hiển thị 5 giây hoặc cuộn qua 30% bài).
 */

export interface ReadEntry {
  slug: string;
  /** Lộ trình chứa bài lúc đọc — để tính tiến độ trên thẻ lộ trình (danh sách lộ trình không kèm id bài). */
  pathSlug?: string;
  at: string;
}
export type ReadMap = Record<string, ReadEntry>;

const isReadMap = (v: unknown): v is ReadMap =>
  isRecord(v) && Object.values(v).every((e) => isRecord(e) && typeof e.slug === 'string' && typeof e.at === 'string');

const readStore = createLocalStore<ReadMap>('tkh-read-articles', isReadMap, {});

export function markArticleRead(articleId: string, slug: string, pathSlug?: string): void {
  readStore.set((prev) => {
    const existing = prev[articleId];
    if (existing && existing.pathSlug === pathSlug && existing.slug === slug) return prev;
    const entry: ReadEntry = { slug, at: existing?.at ?? new Date().toISOString() };
    if (pathSlug) entry.pathSlug = pathSlug;
    return { ...prev, [articleId]: entry };
  });
}

export function useReadArticles(): ReadMap {
  return useLocalStore(readStore);
}

/** Số bài đã đọc của một lộ trình (theo dấu lộ trình ghi lúc đọc), không vượt quá tổng số bài. */
export function countReadInPath(read: ReadMap, pathSlug: string, total: number): number {
  let n = 0;
  for (const e of Object.values(read)) if (e.pathSlug === pathSlug) n++;
  return Math.min(n, total);
}

export interface Bookmark {
  slug: string;
  title: string;
  readingMinutes: number;
  savedAt: string;
}

const isBookmarks = (v: unknown): v is Bookmark[] =>
  Array.isArray(v) &&
  v.every((b) => isRecord(b) && typeof b.slug === 'string' && typeof b.title === 'string' && typeof b.readingMinutes === 'number');

const bookmarkStore = createLocalStore<Bookmark[]>('tkh-bookmarks', isBookmarks, []);

export function useBookmarks(): Bookmark[] {
  return useLocalStore(bookmarkStore);
}

export function isBookmarked(list: Bookmark[], slug: string): boolean {
  return list.some((b) => b.slug === slug);
}

export function toggleBookmark(article: { slug: string; title: string; readingMinutes: number }): void {
  bookmarkStore.set((prev) =>
    isBookmarked(prev, article.slug)
      ? prev.filter((b) => b.slug !== article.slug)
      : [
          { slug: article.slug, title: article.title, readingMinutes: article.readingMinutes, savedAt: new Date().toISOString() },
          ...prev,
        ],
  );
}
