import Link from 'next/link';
import type { TFunction } from 'i18next';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, pickLocalized, type Category, type Locale } from '@tkh/shared';
import { card } from '../../lib/ui';
import { bannerBackground } from './bannerBackground';

/**
 * Thẻ chủ đề con trên trang chủ đề cha (Server Component): cả thẻ là một liên kết; banner riêng ở trên (cùng nền và
 * dự phòng gradient như đầu trang chủ đề), rồi tên, mô tả 2 dòng, số bài — hoặc nhãn "Sắp ra mắt" khi chưa có bài.
 */
export function ChildCategoryCard({ category, locale, t }: { category: Category; locale: Locale; t: TFunction }) {
  const name = pickLocalized(category.name, locale);
  const description = pickLocalized(category.description, locale);
  return (
    <Link
      href={`/topics/${encodeURIComponent(category.slug)}`}
      className={`group flex h-full flex-col overflow-hidden ${card} transition-shadow hover:border-primary hover:shadow-lg hover:shadow-slate-900/5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary dark:hover:shadow-black/30`}
    >
      <div className="relative aspect-[16/7] bg-slate-900" style={bannerBackground(category.slug, category.color)}>
        <span className="absolute bottom-3 left-3 inline-flex h-9 w-9 items-center justify-center rounded-ui bg-slate-950/55 ring-1 ring-white/25">
          <CategoryGlyph icon={category.icon} className="h-5 w-5 text-white" />
        </span>
      </div>
      <div className="flex flex-1 flex-col gap-2 p-4">
        <h2 className="text-base leading-snug font-semibold text-ink group-hover:text-primary-ink">{name}</h2>
        {description && <p className="text-sm leading-relaxed text-muted">{description}</p>}
        <p className="mt-auto pt-2 text-sm">
          {category.articleCount > 0 ? (
            <span className="font-semibold text-ink">{t('common.articleCount', { count: category.articleCount, n: formatNumber(category.articleCount, locale) })}</span>
          ) : (
            <span className="inline-flex rounded-full border border-line bg-surface-2 px-2.5 py-0.5 text-xs font-semibold text-muted">{t('topic.comingSoon')}</span>
          )}
        </p>
      </div>
    </Link>
  );
}
