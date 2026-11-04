'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowLeft, ArrowRight } from 'lucide-react';
import type { ArticleCard, ArticleDetail } from '@tkh/shared';
import { articleHref, ArticleRow } from '../common/ArticleCard';
import { TagChip } from '../common/Badges';
import { ShareButton } from './ShareButton';

const asideTitle = 'mb-3 text-xs font-bold tracking-wide text-ink uppercase';

/** Bài liên quan — danh sách do máy chủ tải sẵn (liên kết nội bộ có trong HTML). `headingId` khác nhau khi khối xuất hiện hai lần. */
export function RelatedArticles({ articles, headingId = 'related-title' }: { articles: ArticleCard[]; headingId?: string }) {
  const { t } = useTranslation();
  if (articles.length === 0) return null;
  return (
    <section aria-labelledby={headingId}>
      <h2 id={headingId} className={asideTitle}>
        {t('article.related')}
      </h2>
      <ul className="space-y-4">
        {articles.map((a) => (
          <li key={a.id}>
            <ArticleRow article={a} compact />
          </li>
        ))}
      </ul>
    </section>
  );
}

export function TagsBlock({ tags, headingId = 'tags-title' }: { tags: string[]; headingId?: string }) {
  const { t } = useTranslation();
  if (tags.length === 0) return null;
  return (
    <section aria-labelledby={headingId}>
      <h2 id={headingId} className={asideTitle}>
        {t('article.tags')}
      </h2>
      <ul className="flex flex-wrap gap-2">
        {tags.map((tag) => (
          <li key={tag}>
            <TagChip tag={tag} />
          </li>
        ))}
      </ul>
    </section>
  );
}

export function ShareBlock() {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="share-title">
      <h2 id="share-title" className={asideTitle}>
        {t('article.share')}
      </h2>
      <p className="mb-3 text-sm text-muted">{t('article.shareHint')}</p>
      <ShareButton variant="block" label={t('common.copyLink')} />
    </section>
  );
}

/** Bài trước / bài sau trong lộ trình. */
export function PrevNext({ article }: { article: Pick<ArticleDetail, 'slug' | 'learningPath'> }) {
  const { t } = useTranslation();
  const list = article.learningPath?.articles ?? [];
  const i = list.findIndex((a) => a.slug === article.slug);
  if (i < 0) return null;
  const prev = list[i - 1];
  const next = list[i + 1];
  if (!prev && !next) return null;
  const box = 'group flex min-w-0 flex-col gap-1 rounded-card border border-line bg-surface p-4 transition-colors hover:border-primary';
  return (
    <nav aria-label={t('article.pathNav')} className="grid gap-3 sm:grid-cols-2">
      {prev ? (
        <Link href={articleHref(prev.slug)} rel="prev" className={box}>
          <span className="inline-flex items-center gap-1 text-xs font-semibold text-muted">
            <ArrowLeft aria-hidden="true" className="h-3.5 w-3.5" />
            {t('article.prev')}
          </span>
          <span className="line-clamp-2 font-semibold text-ink group-hover:text-primary-ink">{prev.title}</span>
        </Link>
      ) : (
        <span className="hidden sm:block" />
      )}
      {next && (
        <Link href={articleHref(next.slug)} rel="next" className={`${box} sm:items-end sm:text-right`}>
          <span className="inline-flex items-center gap-1 text-xs font-semibold text-muted">
            {t('article.next')}
            <ArrowRight aria-hidden="true" className="h-3.5 w-3.5" />
          </span>
          <span className="line-clamp-2 font-semibold text-ink group-hover:text-primary-ink">{next.title}</span>
        </Link>
      )}
    </nav>
  );
}
