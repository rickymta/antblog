'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowUpRight, ArrowRight } from 'lucide-react';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, pickLocalized, type ArticleCard as ArticleCardData, type Category, type LearningPathCard } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { safeColor, withAlpha } from '../../lib/format';
import { orderLearningPaths } from '../../lib/learning-paths';
import { container } from '../../lib/ui';
import { ArticleCard, ArticleRow } from '../common/ArticleCard';
import { SectionHeader } from '../common/PageHeader';
import { PathCard } from '../common/PathCard';
import { EmptyState, RefreshErrorState } from '../common/States';

export function TopicTiles({ categories }: { categories: Category[] | null }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const available = categories?.filter(c => !c.parentId && c.articleCount > 0).sort((a, b) => a.order - b.order);
  return (
    <section aria-labelledby="home-topics" className={container + ' home-section topic-section'}>
      <SectionHeader id="home-topics" title={t('home.topicsTitle')} subtitle={t('home.topicsSubtitle')} />
      {!categories ? <RefreshErrorState compact /> : !available?.length ? <EmptyState title={t('home.topicsEmpty')} /> : (
        <ul className="topic-grid">{available.map(c => {
          const color = safeColor(c.color);
          return <li key={c.id}><Link href={'/topics/' + encodeURIComponent(c.slug)} className="topic-tile group">
            <span className="topic-icon" style={{ backgroundColor: withAlpha(color, 0.12) }}><CategoryGlyph icon={c.icon} className="h-6 w-6" style={{ color }} /></span>
            <span className="topic-body"><span className="block font-semibold text-ink">{pickLocalized(c.name, locale)}</span><span className="line-clamp-4 text-sm leading-relaxed text-muted">{pickLocalized(c.description, locale)}</span><span className="topic-count">{t('common.articleCount', { count: c.articleCount, n: formatNumber(c.articleCount, locale) })}</span></span>
            <ArrowUpRight size={18} className="text-muted group-hover:text-primary-ink" aria-hidden="true" />
          </Link></li>;
        })}
          <li className="topic-all"><Link href="/articles" className="topic-tile topic-tile-all group">
            <span className="topic-body"><span className="block font-semibold text-primary-ink">{t('nav.allTopics')}</span><span className="text-sm leading-relaxed text-muted">{t('home.exploreLibrary')}</span></span>
            <ArrowRight size={18} className="text-primary-ink" aria-hidden="true" />
          </Link></li>
        </ul>
      )}
    </section>
  );
}

export function TrendingSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="home-trending" className={container + ' home-section'}>
      <p className="section-kicker">{t('home.readingLabel')}</p>
      <SectionHeader id="home-trending" title={t('home.trendingTitle')} subtitle={t('home.trendingSubtitle')} link={{ to: '/articles?sort=trending', label: t('common.seeAll') }} />
      {!articles ? <RefreshErrorState /> : articles.length === 0 ? <EmptyState title={t('home.trendingEmpty')} /> : (
        <div className="featured-layout">
          <div className="featured-lead"><ArticleCard article={articles[0]!} showSummary /></div>
          {articles.length > 1 && <div className="featured-reading-list"><p className="editorial-eyebrow">{t('home.keepExploring')}</p><ul>{articles.slice(1).map(a => <li key={a.id}><ArticleRow article={a} /></li>)}</ul><Link href="/articles?sort=trending" className="reading-all">{t('home.exploreLibrary')}<ArrowRight size={17} aria-hidden="true" /></Link></div>}
        </div>
      )}
    </section>
  );
}

/** Prioritize populated paths from distinct topics, then fill remaining slots. */
function featuredPaths(paths: LearningPathCard[]) {
  const populated = orderLearningPaths(paths).filter(p => p.articleCount > 0);
  const selected: LearningPathCard[] = [];
  const topics = new Set<string>();
  for (const path of populated) {
    const topic = path.category?.slug ?? path.slug;
    if (!topics.has(topic)) { selected.push(path); topics.add(topic); }
    if (selected.length === 3) return selected;
  }
  return [...selected, ...populated.filter(p => !selected.includes(p))].slice(0, 3);
}

export function PathsSection({ paths }: { paths: LearningPathCard[] | null }) {
  const { t } = useTranslation();
  const featured = paths ? featuredPaths(paths) : [];
  return (
    <section aria-labelledby="home-paths" className="paths-band">
      <div className={container}>
        <p className="section-kicker">{t('home.learningLabel')}</p>
        <SectionHeader id="home-paths" title={t('home.pathsTitle')} subtitle={t('home.pathsSubtitle')} link={{ to: '/learning-paths', label: t('home.pathsAll') }} />
        {!paths ? <RefreshErrorState /> : featured.length === 0 ? <EmptyState title={t('home.pathsEmpty')} /> : <ul className="path-grid">{featured.map(p => <li key={p.id}><PathCard path={p} /></li>)}</ul>}
      </div>
    </section>
  );
}

export function LatestSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="home-latest" className={container + ' home-section'}>
      <p className="section-kicker">{t('home.latestLabel')}</p>
      <SectionHeader id="home-latest" title={t('home.latestTitle')} subtitle={t('home.latestSubtitle')} link={{ to: '/articles?sort=latest', label: t('common.seeAll') }} />
      {!articles ? <RefreshErrorState /> : articles.length === 0 ? <EmptyState title={t('home.latestEmpty')} /> : <ul className="latest-grid">{articles.map(a => <li key={a.id}><ArticleRow article={a} /></li>)}</ul>}
    </section>
  );
}
