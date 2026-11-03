import { useEffect, useRef, type RefObject } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { api, type ArticleDetail } from '@tkh/shared';
import { markArticleRead } from '../lib/reading';

const VISIBLE_MS = 5000;
const SCROLL_FRACTION = 0.3;

/**
 * Đếm lượt xem MỘT lần mỗi lần mở trang bài: sau khi bài hiển thị 5 giây (tab đang mở) HOẶC người đọc cuộn qua 30% bài.
 * Cùng lúc đánh dấu bài "đã đọc" cho tiến độ lộ trình (lưu ở máy).
 */
export function useArticleView(article: ArticleDetail | undefined, bodyRef: RefObject<HTMLElement | null>): void {
  const qc = useQueryClient();
  const firedFor = useRef<string | null>(null);
  const slug = article?.slug;
  const id = article?.id;
  const pathSlug = article?.learningPath?.slug;

  useEffect(() => {
    if (!slug || !id || firedFor.current === slug) return;

    let visibleMs = 0;
    let last = performance.now();
    let done = false;

    const fire = () => {
      if (done) return;
      done = true;
      firedFor.current = slug;
      cleanup();
      markArticleRead(id, slug, pathSlug);
      api.articles
        .view(slug)
        .then((res) => {
          qc.setQueryData<ArticleDetail>(['article', slug], (prev) =>
            prev ? { ...prev, stats: { ...prev.stats, views: res.views } } : prev,
          );
        })
        .catch(() => {
          /* thống kê không được làm hỏng trang */
        });
    };

    const tick = () => {
      const now = performance.now();
      if (document.visibilityState === 'visible') visibleMs += now - last;
      last = now;
      if (visibleMs >= VISIBLE_MS) fire();
    };

    const onScroll = () => {
      const el = bodyRef.current;
      if (!el) return;
      const rect = el.getBoundingClientRect();
      if (rect.height <= 0) return;
      const seen = (window.innerHeight - rect.top) / rect.height;
      if (seen >= SCROLL_FRACTION) fire();
    };

    const timer = window.setInterval(tick, 500);
    window.addEventListener('scroll', onScroll, { passive: true });
    function cleanup() {
      window.clearInterval(timer);
      window.removeEventListener('scroll', onScroll);
    }
    return cleanup;
  }, [slug, id, pathSlug, bodyRef, qc]);
}
