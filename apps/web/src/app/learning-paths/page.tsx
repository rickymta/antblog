import type { Metadata } from 'next';
import { SearchX } from 'lucide-react';
import { BrowseProvider, BrowseResults } from '../../components/browse/BrowseContext';
import { PathFilters } from '../../components/browse/PathFilters';
import { PageBanner } from '../../components/common/PageBanner';
import { PathCard } from '../../components/common/PathCard';
import { EmptyState, RefreshErrorState } from '../../components/common/States';
import { getServerI18n } from '../../i18n/server';
import { firstParam, parseLevel, type SearchParamsRecord } from '../../lib/articleQuery';
import { pageMetadata } from '../../lib/metadata';
import { serverApi, settle } from '../../lib/server-api';
import { container } from '../../lib/ui';

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
  const { t } = await getServerI18n();
  const query = await parse(searchParams);
  const paths = await settle(serverApi.learningPaths(query));

  return (
    <div className={container}>
      <PageBanner variant="paths" title={t('paths.title')} subtitle={t('paths.subtitle')} />
      <BrowseProvider>
        <PathFilters query={query} />
        <div className="mt-8">
          {!paths ? (
            <RefreshErrorState />
          ) : paths.length === 0 ? (
            <EmptyState icon={<SearchX aria-hidden="true" className="h-7 w-7 text-muted" />} title={t('paths.empty')} />
          ) : (
            <BrowseResults>
              <ul className="grid gap-5 sm:grid-cols-2 xl:grid-cols-4">
                {paths.map((p) => (
                  <li key={p.id}>
                    <PathCard path={p} showDate />
                  </li>
                ))}
              </ul>
            </BrowseResults>
          )}
        </div>
      </BrowseProvider>
    </div>
  );
}
