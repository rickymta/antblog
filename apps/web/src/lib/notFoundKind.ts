/**
 * Proxy rewrite slug không tồn tại về `${NOT_FOUND_REWRITE_PATH}/<mục>` — đường dẫn không khớp route nào ⇒ Next render
 * `app/not-found.tsx` gốc với mã 404 ngay trên máy chủ. `<mục>` (articles | topics | learning-paths) giữ lại để trang 404
 * hiện câu báo riêng: lúc render máy chủ usePathname() trả đường dẫn đã rewrite, ở trình duyệt trả URL thật — cả hai cùng
 * suy ra một câu (không lệch hydrate).
 */
export const NOT_FOUND_REWRITE_PATH = '/__tkh-not-found';

export type NotFoundKey = 'article.notFound' | 'topic.notFound' | 'paths.notFound';

const KEY_BY_SECTION: Record<string, NotFoundKey> = {
  articles: 'article.notFound',
  topics: 'topic.notFound',
  'learning-paths': 'paths.notFound',
};

/** Câu báo 404 riêng theo URL: `/articles/x` hoặc `/__tkh-not-found/articles` ⇒ bài; tương tự chủ đề, lộ trình. */
export function notFoundKeyForPath(pathname: string | null | undefined): NotFoundKey | undefined {
  const parts = (pathname ?? '').split('/');
  const section = `/${parts[1]}` === NOT_FOUND_REWRITE_PATH ? parts[2] : parts[1];
  return section ? KEY_BY_SECTION[section] : undefined;
}
