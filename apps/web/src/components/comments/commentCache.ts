import type { InfiniteData, QueryClient } from '@tanstack/react-query';
import type { ArticleDetail, Comment, CommentPage } from '@tkh/shared';

export type CommentPages = InfiniteData<CommentPage, number>;

export const commentsKey = (slug: string) => ['comments', slug] as const;

/** Sửa một bình luận (gốc hoặc phản hồi) trong MỌI cache danh sách bình luận của bài (mọi kiểu sắp xếp). */
export function patchComment(qc: QueryClient, slug: string, id: string, patch: (c: Comment) => Comment): void {
  qc.setQueriesData<CommentPages>({ queryKey: commentsKey(slug) }, (data) =>
    data
      ? {
          ...data,
          pages: data.pages.map((p) => ({
            ...p,
            items: p.items.map((thread) => {
              if (thread.id === id) return { ...thread, ...patch(thread) };
              if (!thread.replies.some((r) => r.id === id)) return thread;
              return { ...thread, replies: thread.replies.map((r) => (r.id === id ? patch(r) : r)) };
            }),
          })),
        }
      : data,
  );
}

/** Thêm phản hồi mới vào cuối luồng của bình luận gốc `reply.parentId` và tăng các bộ đếm. */
export function appendReply(qc: QueryClient, slug: string, reply: Comment): boolean {
  let found = false;
  qc.setQueriesData<CommentPages>({ queryKey: commentsKey(slug) }, (data) =>
    data
      ? {
          ...data,
          pages: data.pages.map((p) => {
            const has = p.items.some((t) => t.id === reply.parentId);
            if (has) found = true;
            return {
              ...p,
              commentCount: p.commentCount + 1,
              items: has
                ? p.items.map((t) =>
                    t.id === reply.parentId && !t.replies.some((r) => r.id === reply.id)
                      ? { ...t, replies: [...t.replies, reply], replyCount: t.replyCount + 1 }
                      : t,
                  )
                : p.items,
            };
          }),
        }
      : data,
  );
  return found;
}

/** Đồng bộ số bình luận hiển thị ở đầu bài (stats.comments). */
export function bumpArticleComments(qc: QueryClient, slug: string, delta: number): void {
  qc.setQueryData<ArticleDetail>(['article', slug], (a) =>
    a ? { ...a, stats: { ...a.stats, comments: Math.max(0, a.stats.comments + delta) } } : a,
  );
}
