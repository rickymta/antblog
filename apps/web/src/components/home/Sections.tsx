'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { Flame, Newspaper, Route } from 'lucide-react';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, LEVELS, pickLocalized, type ArticleCard as ArticleCardData, type Category, type LearningPathCard } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { safeColor, withAlpha } from '../../lib/format';
import { card, container } from '../../lib/ui';
import { ArticleCard, ArticleRow } from '../common/ArticleCard';
import { SectionHeader } from '../common/PageHeader';
import { PathCard } from '../common/PathCard';
import { EmptyState, RefreshErrorState } from '../common/States';

// Các khối trang chủ nhận dữ liệu do máy chủ tải sẵn; `null` = khối đó tải lỗi (các khối khác vẫn hiển thị).

/** Ô chủ đề: lưới trên màn rộng, hàng cuộn ngang trên điện thoại. */
export function TopicTiles({ categories }: { categories: Category[] | null }) {
  const { t } = useTranslation();
  const locale = useLocale();

  return (
    <section aria-labelledby="home-topics" className={`${container} pt-12`}>
      <SectionHeader id="home-topics" title={t('home.topicsTitle')} subtitle={t('home.topicsSubtitle')} />
      {!categories ? (
        <RefreshErrorState compact />
      ) : categories.length === 0 ? (
        <EmptyState title={t('home.topicsEmpty')} />
      ) : (
        <ul className="-mx-4 flex snap-x gap-4 overflow-x-auto px-4 pb-2 sm:mx-0 sm:grid sm:grid-cols-3 sm:overflow-visible sm:px-0 lg:grid-cols-4 xl:grid-cols-7">
          {categories.map((c) => {
            const color = safeColor(c.color);
            return (
              <li key={c.id} className="w-44 shrink-0 snap-start sm:w-auto">
                <Link
                  href={`/topics/${encodeURIComponent(c.slug)}`}
                  className={`flex h-full flex-col gap-3 p-4 transition-all hover:-translate-y-0.5 hover:border-primary hover:shadow-md ${card}`}
                >
                  <span className="inline-flex h-11 w-11 items-center justify-center rounded-card" style={{ backgroundColor: withAlpha(color, 0.14) }}>
                    <CategoryGlyph icon={c.icon} className="h-6 w-6" style={{ color }} />
                  </span>
                  <span>
                    <span className="block font-semibold text-ink">{pickLocalized(c.name, locale)}</span>
                    <span className="mt-0.5 block text-xs text-muted">
                      {t('common.articleCount', { count: c.articleCount, n: formatNumber(c.articleCount, locale) })}
                    </span>
                  </span>
                </Link>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}

export function TrendingSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();

  return (
    <section aria-labelledby="home-trending" className={`${container} pt-14`}>
      <SectionHeader
        id="home-trending"
        title={t('home.trendingTitle')}
        subtitle={t('home.trendingSubtitle')}
        icon={<Flame aria-hidden="true" className="h-6 w-6 text-orange-500" />}
        link={{ to: '/articles?sort=trending', label: t('common.seeAll') }}
      />
      {!articles ? (
        <RefreshErrorState />
      ) : articles.length === 0 ? (
        <EmptyState title={t('home.trendingEmpty')} />
      ) : (
        <ul className="grid gap-5 sm:grid-cols-2 lg:grid-cols-4">
          {articles.map((a) => (
            <li key={a.id}>
              <ArticleCard article={a} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/** Mỗi cấp độ một lộ trình tiêu biểu (lộ trình đầu tiên API trả về cho cấp đó). */
function onePerLevel(paths: LearningPathCard[]): LearningPathCard[] {
  return LEVELS.map((level) => paths.find((p) => p.level === level)).filter((p): p is LearningPathCard => Boolean(p));
}

export function PathsSection({ paths }: { paths: LearningPathCard[] | null }) {
  const { t } = useTranslation();
  const featured = paths ? onePerLevel(paths) : [];

  return (
    <section aria-labelledby="home-paths" className={`${container} pt-14`}>
      <SectionHeader
        id="home-paths"
        title={t('home.pathsTitle')}
        subtitle={t('home.pathsSubtitle')}
        icon={<Route aria-hidden="true" className="h-6 w-6 text-secondary" />}
        link={{ to: '/learning-paths', label: t('home.pathsAll') }}
      />
      {!paths ? (
        <RefreshErrorState />
      ) : featured.length === 0 ? (
        <EmptyState title={t('home.pathsEmpty')} />
      ) : (
        <ul className="grid gap-5 md:grid-cols-3">
          {featured.map((p) => (
            <li key={p.id}>
              <PathCard path={p} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

export function LatestSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();

  return (
    <section aria-labelledby="home-latest" className={`${container} pt-14`}>
      <SectionHeader
        id="home-latest"
        title={t('home.latestTitle')}
        icon={<Newspaper aria-hidden="true" className="h-6 w-6 text-primary" />}
        link={{ to: '/articles', label: t('common.seeAll') }}
      />
      {!articles ? (
        <RefreshErrorState />
      ) : articles.length === 0 ? (
        <EmptyState title={t('home.latestEmpty')} />
      ) : (
        <ul className={`grid gap-x-8 gap-y-6 p-5 md:grid-cols-2 sm:p-6 ${card}`}>
          {articles.map((a) => (
            <li key={a.id}>
              <ArticleRow article={a} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
