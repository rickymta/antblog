'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { BadgeCheck, ChevronRight, Clock, Eye, House } from 'lucide-react';
import { Avatar } from '@tkh/ui';
import { formatCompact, pickLocalized } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { formatDay } from '../../lib/dates';
import { scrollToId } from '../../lib/dom';
import { BookmarkButton } from '../common/BookmarkButton';
import { CategoryChip, LevelTag } from '../common/Badges';
import { useArticle } from './ArticleContext';
import { REACTIONS } from './reactionMeta';
import { ShareButton } from './ShareButton';

export function Breadcrumb() {
  const { t } = useTranslation();
  const locale = useLocale();
  const article = useArticle();
  const sep = <ChevronRight aria-hidden="true" className="h-3.5 w-3.5 shrink-0 text-muted" />;
  const link = 'shrink-0 text-muted hover:text-ink hover:underline';
  return (
    <nav aria-label={t('article.breadcrumb')}>
      <ol className="flex min-w-0 items-center gap-1.5 text-sm">
        <li className="flex items-center gap-1.5">
          <Link href="/" className={`${link} inline-flex`} aria-label={t('nav.home')}>
            <House aria-hidden="true" className="h-4 w-4" />
          </Link>
          {sep}
        </li>
        <li className="flex items-center gap-1.5">
          <Link href="/articles" className={link}>
            {t('nav.articles')}
          </Link>
          {sep}
        </li>
        <li className="flex shrink-0 items-center gap-1.5">
          <Link href={`/topics/${encodeURIComponent(article.category.slug)}`} className={`${link} block max-w-[7rem] truncate sm:max-w-[12rem]`}>
            {pickLocalized(article.category.name, locale)}
          </Link>
          {sep}
        </li>
        <li className="min-w-0 flex-1 truncate text-ink" aria-current="page">
          {article.title}
        </li>
      </ol>
    </nav>
  );
}

/** Thanh reaction nhỏ dưới tiêu đề (A2): chỉ hiển thị số — bấm để cuộn xuống 4 thẻ chọn cảm xúc. */
function ReactionSummary() {
  const { t } = useTranslation();
  const locale = useLocale();
  const article = useArticle();
  return (
    <a
      href="#reactions"
      onClick={(e) => {
        e.preventDefault();
        scrollToId('reactions');
      }}
      className="inline-flex flex-wrap items-center gap-x-3 gap-y-1 rounded-ui px-2 py-1.5 text-sm text-ink hover:bg-surface-2"
    >
      {/* Tên truy cập dựng từ nội dung (chứa đúng các con số nhìn thấy) + chữ chỉ dành cho trình đọc màn hình. */}
      <span className="sr-only">{t('article.reactionsSummary')}:</span>
      {REACTIONS.map((r) => (
        <span key={r.type} className={`inline-flex items-center gap-1 ${article.myReactions.includes(r.type) ? 'font-semibold text-primary-ink' : ''}`}>
          <span aria-hidden="true">{r.emoji}</span>
          <span className="sr-only">{r.name}</span>
          <span>{formatCompact(article.stats.reactions[r.type], locale)}</span>
        </span>
      ))}
    </a>
  );
}

/** Đầu bài (giữa): breadcrumb, nhãn, tiêu đề rất lớn, tác giả · phút đọc · ngày, thanh reaction + Lưu + Chia sẻ. */
export function ArticleHeader() {
  const { t } = useTranslation();
  const locale = useLocale();
  const article = useArticle();
  const verified = article.author.role === 'admin' || article.author.role === 'author';
  return (
    <header className="space-y-5">
      <Breadcrumb />
      <div className="flex flex-wrap items-center gap-2">
        <CategoryChip category={article.category} />
        <LevelTag level={article.level} />
      </div>
      <h1 lang={article.language} className="text-3xl leading-tight font-extrabold tracking-tight text-balance text-ink sm:text-4xl xl:text-[2.75rem]">
        {article.title}
      </h1>
      {article.summary && (
        <p lang={article.language} className="text-lg text-muted">
          {article.summary}
        </p>
      )}
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-muted">
        <span className="inline-flex items-center gap-2 text-ink">
          <Avatar name={article.author.displayName} src={article.author.avatarUrl} size={32} />
          <span className="font-semibold">{article.author.displayName}</span>
          {verified && <BadgeCheck aria-label={t('common.verified')} role="img" className="h-4 w-4 text-primary" />}
        </span>
        <span className="inline-flex items-center gap-1">
          <Clock aria-hidden="true" className="h-4 w-4" />
          {t('common.minRead', { count: article.readingMinutes })}
        </span>
        <span className="inline-flex items-center gap-1">
          <Eye aria-hidden="true" className="h-4 w-4" />
          {t('common.views', { n: formatCompact(article.stats.views, locale) })}
        </span>
        <time dateTime={article.publishedAt}>{formatDay(article.publishedAt, locale)}</time>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-3 border-y border-line py-2">
        <ReactionSummary />
        <div className="flex items-center gap-2">
          <BookmarkButton article={article} withLabel />
          <ShareButton />
        </div>
      </div>
    </header>
  );
}
