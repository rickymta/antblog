import Link from 'next/link';
import type { TFunction } from 'i18next';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, pickLocalized, type Category, type Locale } from '@tkh/shared';
import { card } from '../../lib/ui';

/** Compact topic link with a theme-aware icon, description and article count. */
export function ChildCategoryCard({ category, locale, t }: { category: Category; locale: Locale; t: TFunction }) {
  const name = pickLocalized(category.name, locale);
  const description = pickLocalized(category.description, locale);
  return (
    <Link
      href={`/topics/${encodeURIComponent(category.slug)}`}
      className={`group flex h-full flex-col overflow-hidden ${card} transition-shadow hover:border-primary hover:shadow-lg hover:shadow-slate-900/5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary dark:hover:shadow-black/30`}
    >
      <span className="mx-5 mt-5 inline-flex h-10 w-10 items-center justify-center rounded-ui bg-primary-soft text-primary-ink">
        <CategoryGlyph icon={category.icon} className="h-5 w-5" />
      </span>
      <div className="flex flex-1 flex-col gap-2 p-5">
        <h3 className="text-lg leading-snug font-semibold text-ink group-hover:text-primary-ink">{name}</h3>
        {description && <p className="text-sm leading-relaxed text-muted">{description}</p>}
        <p className="mt-auto pt-2 text-sm">
          {category.articleCount > 0 ? (
            <span className="font-semibold text-ink">{t('common.articleCount', { count: category.articleCount, n: formatNumber(category.articleCount, locale) })}</span>
          ) : (
            <span className="inline-flex rounded-full border border-line bg-surface-2 px-2.5 py-0.5 text-xs font-semibold text-muted">{t('topic.comingSoon')}</span>
          )}
        </p>
        {category.articleCount > 0 && <span className="mt-1 text-sm font-semibold text-primary-ink">{t('topic.viewTopic')} <span aria-hidden="true">→</span></span>}
      </div>
    </Link>
  );
}
