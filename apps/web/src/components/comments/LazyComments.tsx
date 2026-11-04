'use client';

import { useEffect, useRef, useState } from 'react';
import dynamic from 'next/dynamic';
import { useTranslation } from 'react-i18next';
import { Skeleton } from '@tkh/ui';

/** Khung chờ cùng chiều cao tương đối với khối bình luận — vừa là HTML máy chủ, vừa là lúc đang tải mã. */
function CommentsPlaceholder() {
  const { t } = useTranslation();
  return (
    <section id="comments" aria-labelledby="comments-title" className="scroll-mt-24">
      <h2 id="comments-title" className="text-xl font-bold text-ink">
        {t('comments.title', { n: '…' })}
      </h2>
      <Skeleton className="mt-5 h-24 w-full rounded-card" />
      <div className="mt-4 space-y-3" aria-hidden="true">
        <Skeleton className="h-16 w-full rounded-card" />
        <Skeleton className="h-16 w-full rounded-card" />
      </div>
    </section>
  );
}

// Mã khối bình luận (danh sách, ô soạn, trả lời…) tách thành chunk riêng — không nằm trong JS tải lúc mở bài.
const CommentsSection = dynamic(() => import('./CommentsSection').then((m) => m.CommentsSection), {
  ssr: false,
  loading: CommentsPlaceholder,
});

/**
 * Bình luận chỉ nạp (mã + gọi API) khi người đọc cuộn tới gần (cách ~1 màn hình), hoặc ngay khi URL trỏ tới bình luận
 * (`#comments`, `#comment-…`). Bình luận vốn tải ở trình duyệt (không có trong HTML máy chủ) ⇒ không ảnh hưởng SEO;
 * HTML máy chủ vẫn có tiêu đề + khung chờ và mốc `#comments`.
 */
export function LazyComments({ slug }: { slug: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const [show, setShow] = useState(false);

  useEffect(() => {
    if (show) return;
    if (window.location.hash.startsWith('#comment') || typeof IntersectionObserver === 'undefined') {
      setShow(true);
      return;
    }
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => {
        if (entries.some((e) => e.isIntersecting)) setShow(true);
      },
      { rootMargin: '100% 0px' },
    );
    io.observe(el);
    const onHash = () => {
      if (window.location.hash.startsWith('#comment')) setShow(true);
    };
    window.addEventListener('hashchange', onHash);
    return () => {
      io.disconnect();
      window.removeEventListener('hashchange', onHash);
    };
  }, [show]);

  return <div ref={ref}>{show ? <CommentsSection slug={slug} /> : <CommentsPlaceholder />}</div>;
}
