import { useParams } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { CategoryGlyph, Skeleton } from '@tkh/ui';
import { api, formatNumber, pickLocalized } from '@tkh/shared';
import { useLocale } from '../i18n';
import { ArticleBrowser } from '../components/common/ArticleBrowser';
import { ErrorState, LoadingRegion } from '../components/common/States';
import { useDocumentTitle } from '../hooks/useDocumentTitle';
import { isNotFound } from '../lib/errors';
import { safeColor, withAlpha } from '../lib/format';
import { container } from '../lib/ui';
import { NotFoundPage } from './NotFoundPage';

export function TopicPage() {
  const { slug = '' } = useParams();
  const { t } = useTranslation();
  const locale = useLocale();
  const category = useQuery({ queryKey: ['category', slug], queryFn: () => api.categories.get(slug) });
  const name = category.data ? pickLocalized(category.data.name, locale) : undefined;
  useDocumentTitle(name);

  if (category.isError && isNotFound(category.error)) return <NotFoundPage message={t('topic.notFound')} />;

  return (
    <div className={container}>
      {category.isPending ? (
        <LoadingRegion className="flex items-center gap-5 py-10">
          <Skeleton className="h-16 w-16 rounded-card" />
          <div className="flex-1 space-y-3">
            <Skeleton className="h-8 w-1/3" />
            <Skeleton className="h-4 w-2/3" />
          </div>
        </LoadingRegion>
      ) : category.isError ? (
        <div className="py-10">
          <ErrorState onRetry={() => void category.refetch()} />
        </div>
      ) : (
        <header className="flex flex-col gap-5 py-10 sm:flex-row sm:items-center">
          <span
            className="inline-flex h-16 w-16 shrink-0 items-center justify-center rounded-card"
            style={{ backgroundColor: withAlpha(safeColor(category.data.color), 0.15) }}
          >
            <CategoryGlyph icon={category.data.icon} className="h-9 w-9" style={{ color: safeColor(category.data.color) }} />
          </span>
          <div className="min-w-0 space-y-2">
            <p className="text-xs font-bold tracking-wide text-muted uppercase">{t('topic.eyebrow')}</p>
            <h1 className="text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{name}</h1>
            <p className="max-w-3xl text-muted">{pickLocalized(category.data.description, locale)}</p>
            <p className="text-sm font-semibold text-ink">
              {t('common.articleCount', { count: category.data.articleCount, n: formatNumber(category.data.articleCount, locale) })}
            </p>
          </div>
        </header>
      )}
      {/* Danh sách bài tải song song với thông tin chủ đề (lọc theo slug trên URL). */}
      {!category.isError && <ArticleBrowser fixedCategory={slug} />}
    </div>
  );
}
