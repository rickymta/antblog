'use client';

import { useTranslation } from 'react-i18next';
import { Bookmark, BookmarkCheck } from 'lucide-react';
import { isBookmarked, toggleBookmark, useBookmarks } from '../../lib/reading';

/** Lưu bài để đọc sau (localStorage). `withLabel` ⇒ nút có chữ, không thì nút biểu tượng. */
export function BookmarkButton({
  article,
  withLabel = false,
  className = '',
}: {
  article: { slug: string; title: string; readingMinutes: number };
  withLabel?: boolean;
  className?: string;
}) {
  const { t } = useTranslation();
  const saved = isBookmarked(useBookmarks(), article.slug);
  const Icon = saved ? BookmarkCheck : Bookmark;
  const label = saved ? t('common.unsaveArticle') : t('common.saveArticle');

  return (
    <button
      type="button"
      onClick={(e) => {
        // Nút nằm trong thẻ có liên kết: không để bấm lưu kéo theo mở bài.
        e.preventDefault();
        e.stopPropagation();
        toggleBookmark(article);
      }}
      aria-pressed={saved}
      aria-label={withLabel ? undefined : label}
      title={label}
      className={`inline-flex items-center gap-1.5 rounded-ui text-sm font-medium transition-colors ${
        withLabel ? 'border border-line px-3 py-1.5 hover:bg-surface-2' : 'h-8 w-8 justify-center hover:bg-surface-2'
      } ${saved ? 'text-primary-ink' : 'text-muted hover:text-ink'} ${className}`}
    >
      <Icon aria-hidden="true" className="h-4 w-4" fill={saved ? 'currentColor' : 'none'} />
      {withLabel && <span>{saved ? t('common.saved') : t('common.save')}</span>}
    </button>
  );
}
