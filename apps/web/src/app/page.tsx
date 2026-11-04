import type { Metadata } from 'next';
import { CommunitySection } from '../components/home/CommunitySection';
import { Hero } from '../components/home/Hero';
import { LatestSection, PathsSection, TopicTiles, TrendingSection } from '../components/home/Sections';
import { getServerI18n } from '../i18n/server';
import { JsonLd } from '../lib/jsonld';
import { pageMetadata } from '../lib/metadata';
import { serverApi, settle } from '../lib/server-api';
import { absoluteUrl, LOGO_PATH, SITE_NAME, SITE_URL } from '../lib/site';

export async function generateMetadata(): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  return pageMetadata({ absoluteTitle: `${SITE_NAME} — ${t('home.heroEyebrow')}`, description: t('meta.description'), path: '/', locale });
}

export default async function HomePage() {
  const { t, locale } = await getServerI18n();
  // Các khối tải song song; khối nào lỗi thì chỉ khối đó hiện "Thử lại".
  const [summary, categories, trending, paths, latest] = await Promise.all([
    settle(serverApi.summary()),
    settle(serverApi.categories()),
    settle(serverApi.trending(4)),
    settle(serverApi.learningPaths()),
    settle(serverApi.articles({ sort: 'latest', pageSize: 6 })),
  ]);

  const organization = {
    '@type': 'Organization',
    '@id': `${SITE_URL}/#organization`,
    name: SITE_NAME,
    url: `${SITE_URL}/`,
    logo: { '@type': 'ImageObject', url: absoluteUrl(LOGO_PATH), width: 512, height: 512 },
  };
  const website = {
    '@type': 'WebSite',
    '@id': `${SITE_URL}/#website`,
    name: SITE_NAME,
    url: `${SITE_URL}/`,
    description: t('meta.description'),
    inLanguage: locale === 'en' ? 'en' : 'vi',
    publisher: { '@id': `${SITE_URL}/#organization` },
    potentialAction: {
      '@type': 'SearchAction',
      target: { '@type': 'EntryPoint', urlTemplate: `${SITE_URL}/articles?q={search_term_string}` },
      'query-input': 'required name=search_term_string',
    },
  };

  return (
    <>
      <JsonLd data={{ '@context': 'https://schema.org', '@graph': [website, organization] }} />
      <Hero summary={summary} />
      <TopicTiles categories={categories} />
      <TrendingSection articles={trending} />
      <PathsSection paths={paths} />
      <LatestSection articles={latest?.items ?? null} />
      <CommunitySection memberCount={summary?.memberCount} />
    </>
  );
}
