'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ArrowUpRight, ArrowRight } from 'lucide-react';
import { CategoryGlyph } from '@tkh/ui';
import { formatNumber, pickLocalized, type ArticleCard as ArticleCardData, type Category, type LearningPathCard } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { safeColor, withAlpha } from '../../lib/format';
import { container } from '../../lib/ui';
import { ArticleCard, ArticleRow } from '../common/ArticleCard';
import { SectionHeader } from '../common/PageHeader';
import { PathCard } from '../common/PathCard';
import { EmptyState, RefreshErrorState } from '../common/States';

export function TopicTiles({ categories }: { categories: Category[] | null }) {
  const { t } = useTranslation();
  const locale = useLocale();
  const available = categories?.filter(c => !c.parentId && c.articleCount > 0).sort((a, b) => b.articleCount - a.articleCount);
  const upcoming = categories?.filter(c => !c.parentId && c.articleCount === 0);
  return (
    <section aria-labelledby="home-topics" className={container + ' home-section topic-section'}>
      <SectionHeader id="home-topics" title={t('home.topicsTitle')} subtitle={t('home.topicsSubtitle')} />
      {!categories ? <RefreshErrorState compact /> : !available?.length ? <EmptyState title={t('home.topicsEmpty')} /> : (
        <ul className="topic-grid">{available.map(c => {
          const color = safeColor(c.color);
          return <li key={c.id}><Link href={'/topics/' + encodeURIComponent(c.slug)} className="topic-tile group">
            <span className="topic-icon" style={{ backgroundColor: withAlpha(color, 0.12) }}><CategoryGlyph icon={c.icon} className="h-6 w-6" style={{ color }} /></span>
            <span className="min-w-0 flex-1"><span className="block font-semibold text-ink">{pickLocalized(c.name, locale)}</span><span className="mt-1 block text-xs text-muted">{t('common.articleCount', { count: c.articleCount, n: formatNumber(c.articleCount, locale) })}</span></span>
            <ArrowUpRight size={18} className="text-muted group-hover:text-primary-ink" aria-hidden="true" />
          </Link></li>;
        })}</ul>
      )}
      {!!upcoming?.length && <p className="topic-upcoming"><span>{t('home.upcomingTopics')}</span>{upcoming.map(c => <Link key={c.id} href={'/topics/' + encodeURIComponent(c.slug)}>{pickLocalized(c.name, locale)}</Link>)}</p>}
    </section>
  );
}

export function TrendingSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="home-trending" className={container + ' home-section'}>
      <p className="section-kicker">01 / {t('home.readingLabel')}</p>
      <SectionHeader id="home-trending" title={t('home.trendingTitle')} subtitle={t('home.trendingSubtitle')} link={{ to: '/articles?sort=trending', label: t('common.seeAll') }} />
      {!articles ? <RefreshErrorState /> : articles.length === 0 ? <EmptyState title={t('home.trendingEmpty')} /> : (
        <div className="featured-layout">
          <div className="featured-lead"><ArticleCard article={articles[0]!} showSummary /></div>
          {articles.length > 1 && <div className="featured-reading-list"><p className="editorial-eyebrow">{t('home.keepExploring')}</p><ol>{articles.slice(1).map((a, i) => <li key={a.id}><span className="reading-index" aria-hidden="true">0{i + 2}</span><ArticleRow article={a} /></li>)}</ol><Link href="/articles?sort=popular" className="reading-all">{t('home.exploreLibrary')}<ArrowRight size={17} aria-hidden="true" /></Link></div>}
        </div>
      )}
    </section>
  );
}

/** Prioritize populated paths from distinct topics, then fill remaining slots. */
function featuredPaths(paths: LearningPathCard[]) {
  const populated = paths.filter(p => p.articleCount > 0);
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
        <p className="section-kicker">02 / {t('home.learningLabel')}</p>
        <SectionHeader id="home-paths" title={t('home.pathsTitle')} subtitle={t('home.pathsSubtitle')} link={{ to: '/learning-paths', label: t('home.pathsAll') }} />
        {!paths ? <RefreshErrorState /> : featured.length === 0 ? <EmptyState title={t('home.pathsEmpty')} /> : <ol className="path-grid">{featured.map((p, i) => <li key={p.id}><div className="path-number" aria-hidden="true">0{i + 1}<span> / {p.category ? pickLocalized(p.category.name, 'en') : 'LEARNING PATH'}</span></div><PathCard path={p} /></li>)}</ol>}
      </div>
    </section>
  );
}

export function LatestSection({ articles }: { articles: ArticleCardData[] | null }) {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="home-latest" className={container + ' home-section'}>
      <p className="section-kicker">03 / {t('home.latestLabel')}</p>
      <SectionHeader id="home-latest" title={t('home.latestTitle')} subtitle={t('home.latestSubtitle')} link={{ to: '/articles', label: t('common.seeAll') }} />
      {!articles ? <RefreshErrorState /> : articles.length === 0 ? <EmptyState title={t('home.latestEmpty')} /> : <ul className="latest-grid">{articles.map(a => <li key={a.id}><ArticleCard article={a} showSummary /></li>)}</ul>}
    </section>
  );
}
