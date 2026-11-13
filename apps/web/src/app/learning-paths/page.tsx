import { pickLocalized, type LearningPathCard } from '@tkh/shared';
import type { Metadata } from 'next';
import { SearchX } from 'lucide-react';
import { BrowseProvider, BrowseResults } from '../../components/browse/BrowseContext';
import { PathFilters } from '../../components/browse/PathFilters';
import { PageBanner } from '../../components/common/PageBanner';
import { PathCard } from '../../components/common/PathCard';
import { Cover } from '../../components/common/Cover';
import { EmptyState, RefreshErrorState } from '../../components/common/States';
import { getServerI18n } from '../../i18n/server';
import { firstParam, parseLevel, type SearchParamsRecord } from '../../lib/articleQuery';
import { pageMetadata } from '../../lib/metadata';
import { serverApi, settle } from '../../lib/server-api';
import { orderLearningPaths, orderTopicParts } from '../../lib/learning-paths';
import { container } from '../../lib/ui';
import { pathCover } from '../../lib/path-cover';

type Props = { searchParams: Promise<SearchParamsRecord> };

async function parse(searchParams: Props['searchParams']) {
  const sp = await searchParams;
  const level = parseLevel(firstParam(sp.level));
  const q = firstParam(sp.q)?.trim().slice(0, 200) || undefined;
  return { ...(level ? { level } : {}), ...(q ? { q } : {}) };
}

export async function generateMetadata({ searchParams }: Props): Promise<Metadata> {
  const { t, locale } = await getServerI18n();
  const query = await parse(searchParams);
  // Lọc cấp độ / tìm kiếm chỉ là cách xem khác của cùng danh sách ⇒ canonical luôn là /learning-paths.
  return pageMetadata({ title: t('nav.paths'), description: t('paths.subtitle'), path: '/learning-paths', locale, noindex: Boolean(query.q) });
}

export default async function LearningPathsPage({ searchParams }: Props) {
  const { t, locale } = await getServerI18n();
  const query = await parse(searchParams);
  const paths = await settle(serverApi.learningPaths(query));
  const groups = new Map<string, { title: string; paths: LearningPathCard[] }>();
  for (const path of orderLearningPaths(paths ?? [])) {
    const key = path.category?.slug ?? 'other';
    const group = groups.get(key) ?? { title: path.category ? pickLocalized(path.category.name, locale) : t('nav.paths'), paths: [] };
    group.paths.push(path);
    groups.set(key, group);
  }

  return (
    <div className={container}>
      <PageBanner variant="paths" title={t('paths.title')} subtitle={t('paths.subtitle')} />
      <BrowseProvider>
        <p className="mb-5 max-w-3xl text-base leading-relaxed text-muted">{t('paths.chooseGuide')}</p>
        <PathFilters query={query} />
        <div className="mt-8">
          {!paths ? (
            <RefreshErrorState />
          ) : paths.length === 0 ? (
            <EmptyState icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />} title={t('paths.empty')} />
          ) : (
            <BrowseResults>
              <div className="space-y-10">
                <nav aria-label={t('paths.byTopic')} className="flex flex-wrap gap-2">
                  {Array.from(groups, ([key, group]) => <a key={key} href={`#paths-${key}`} className="inline-flex min-h-11 items-center rounded-full border border-line px-4 py-2 text-sm text-primary-ink hover:bg-primary-soft">{group.title} ({group.paths.length})</a>)}
                </nav>
                {Array.from(groups, ([key, group]) => (
                  <section key={key} aria-labelledby={`paths-${key}`}>
                    <div className="relative isolate mb-5 overflow-hidden rounded-card border border-line bg-slate-950 text-white">
                      {group.paths[0]?.category && <div className="absolute inset-0 -z-10">
                        <Cover src={pathCover(key)} category={group.paths[0].category} className="object-right" />
                      </div>}
                      <div className="pointer-events-none absolute inset-0 -z-10 bg-gradient-to-r from-slate-950/90 via-slate-950/60 to-transparent" />
                      <div className="flex min-h-44 flex-col justify-center gap-3 p-6 sm:min-h-52 sm:p-8 lg:min-h-60 lg:p-10">
                        <p className="text-xs font-semibold tracking-widest text-teal-200 uppercase">{t('topic.eyebrow')}</p>
                        <h2 id={`paths-${key}`} className="max-w-[75%] scroll-mt-28 text-2xl font-bold sm:max-w-[55%] sm:text-3xl">{group.title}</h2>
                        <p className="max-w-[75%] text-sm leading-relaxed text-slate-200 sm:max-w-[48%]">{t('paths.topicGuide')}</p>
                        <div className="flex flex-wrap gap-2 text-sm font-medium">
                          <span className="rounded-full bg-white/10 px-3 py-1 ring-1 ring-white/20">{t('paths.count', { count: group.paths.length, n: group.paths.length })}</span>
                          <span className="rounded-full bg-white/10 px-3 py-1 ring-1 ring-white/20">{t('common.lessonCount', { count: group.paths.reduce((sum, p) => sum + p.articleCount, 0), n: group.paths.reduce((sum, p) => sum + p.articleCount, 0) })}</span>
                        </div>
                      </div>
                    </div>
                    <ul className="grid gap-5 md:grid-cols-2 xl:grid-cols-3">
                      {orderTopicParts(group.paths).map(p => <li key={p.id}><PathCard path={p} showDate /></li>)}
                    </ul>
                  </section>
                ))}
              </div>
            </BrowseResults>
          )}
        </div>
      </BrowseProvider>
    </div>
  );
}
