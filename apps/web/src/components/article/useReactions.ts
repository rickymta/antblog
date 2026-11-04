'use client';

import { useCallback, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { api, type ArticleDetail, type ReactionType } from '@tkh/shared';
import { useAuth } from '../../lib/auth';
import { isUnauthenticated } from '../../lib/errors';

type Outcome = 'ok' | 'login' | 'error' | 'busy';

function applyToggle(a: ArticleDetail, type: ReactionType, on: boolean): ArticleDetail {
  const has = a.myReactions.includes(type);
  if (has === on) return a;
  return {
    ...a,
    myReactions: on ? [...a.myReactions, type] : a.myReactions.filter((r) => r !== type),
    stats: { ...a.stats, reactions: { ...a.stats.reactions, [type]: Math.max(0, a.stats.reactions[type] + (on ? 1 : -1)) } },
  };
}

/**
 * Bật/tắt reaction với cập nhật lạc quan trên cache `['article', slug]` — thanh tóm tắt đầu bài và 4 thẻ cuối bài
 * cùng đọc cache này nên luôn khớp nhau. Lỗi ⇒ hoàn tác đúng loại vừa bấm; 401 ⇒ phiên đã hết, báo cần đăng nhập.
 */
export function useReactions(slug: string) {
  const qc = useQueryClient();
  const { user, setUser } = useAuth();
  const [pending, setPending] = useState<ReadonlySet<ReactionType>>(new Set());
  const key = ['article', slug];

  const toggle = useCallback(
    async (type: ReactionType): Promise<Outcome> => {
      if (!user) return 'login';
      const current = qc.getQueryData<ArticleDetail>(key);
      if (!current) return 'error';
      if (pending.has(type)) return 'busy';
      const turnOn = !current.myReactions.includes(type);

      qc.setQueryData<ArticleDetail>(key, (prev) => (prev ? applyToggle(prev, type, turnOn) : prev));
      setPending((s) => new Set(s).add(type));
      try {
        const res = turnOn ? await api.articles.react(slug, type) : await api.articles.unreact(slug, type);
        qc.setQueryData<ArticleDetail>(key, (prev) =>
          prev ? { ...prev, myReactions: res.myReactions, stats: { ...prev.stats, reactions: res.reactions } } : prev,
        );
        return 'ok';
      } catch (e) {
        qc.setQueryData<ArticleDetail>(key, (prev) => (prev ? applyToggle(prev, type, !turnOn) : prev));
        if (isUnauthenticated(e)) {
          setUser(null);
          return 'login';
        }
        return 'error';
      } finally {
        setPending((s) => {
          const next = new Set(s);
          next.delete(type);
          return next;
        });
      }
    },
    // key là mảng mới mỗi render nhưng chỉ phụ thuộc slug.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [qc, slug, user, pending, setUser],
  );

  return { toggle, pending, loggedIn: Boolean(user) };
}
