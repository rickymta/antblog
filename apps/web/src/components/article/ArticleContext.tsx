'use client';

import { createContext, useContext, useMemo, type ReactNode } from 'react';
import { useQuery } from '@tanstack/react-query';
import { api, type ArticleDetail } from '@tkh/shared';
import { useAuth } from '../../lib/auth';

const ArticleContext = createContext<ArticleDetail | null>(null);

/**
 * Bài đang đọc phía trình duyệt. Máy chủ render bài kèm số đếm công khai (`myReactions` rỗng — trang được cache chung);
 * sau khi hydrate, nếu đã đăng nhập thì tải lại bài để biết reaction CỦA NGƯỜI XEM. Cache `['article', slug]`
 * là nơi thanh reaction đầu bài, 4 thẻ cuối bài, số lượt xem và số bình luận cùng cập nhật.
 */
export function ArticleProvider({ initial, children }: { initial: ArticleDetail; children: ReactNode }) {
  const { user } = useAuth();
  const query = useQuery({
    queryKey: ['article', initial.slug],
    queryFn: () => api.articles.get(initial.slug),
    initialData: initial,
    // Bản của máy chủ không có reaction của người xem ⇒ coi như cũ, tải lại ngay khi biết đã đăng nhập.
    initialDataUpdatedAt: 0,
    enabled: Boolean(user),
  });
  const data = query.data;
  const value = useMemo<ArticleDetail>(() => (user ? data : { ...data, myReactions: [] }), [data, user]);
  return <ArticleContext.Provider value={value}>{children}</ArticleContext.Provider>;
}

export function useArticle(): ArticleDetail {
  const ctx = useContext(ArticleContext);
  if (!ctx) throw new Error('useArticle phải nằm trong <ArticleProvider>');
  return ctx;
}
