'use client';

import { useRef, type ReactNode } from 'react';
import { useArticleView } from '../../hooks/useArticleView';
import { useArticle } from './ArticleContext';
import { CodeCopyOverlay } from './CodeCopyOverlay';

/**
 * Khung thân bài phía trình duyệt: đếm lượt xem (5 giây hiển thị hoặc cuộn qua 30%) và lớp nút "Sao chép" cho khối mã.
 * Nội dung Markdown (`children`) render ở MÁY CHỦ — trình duyệt không phải tải bộ Markdown + highlight.js.
 */
export function ArticleBody({ children }: { children: ReactNode }) {
  const article = useArticle();
  const bodyRef = useRef<HTMLDivElement>(null);
  useArticleView(article, bodyRef);
  return (
    <div ref={bodyRef} lang={article.language} className="tkh-article-body relative mt-8">
      {children}
      <CodeCopyOverlay containerRef={bodyRef} contentKey={article.id + article.updatedAt} />
    </div>
  );
}
