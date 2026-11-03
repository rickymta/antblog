import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { api, formatDate, formatNumber } from '@tkh/shared';
import { Card, EmptyState, ErrorState, PageHeader, Pagination, TableSkeleton, TableWrap, Td, Th } from '../components/ui';
import { useUrlFilters } from '../lib/hooks';
import { errorMessage } from '../lib/errors';
import { useLocale } from '../i18n';

export function SubscribersPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const f = useUrlFilters([] as const);
  const list = useQuery({
    queryKey: ['admin', 'subscribers', f.page],
    queryFn: () => api.admin.subscribers.list(f.page),
    placeholderData: keepPreviousData,
  });

  return (
    <>
      <PageHeader
        title={t('subscribers.title')}
        subtitle={list.data ? t('subscribers.total', { count: list.data.total }) : t('subscribers.subtitle')}
      />
      <Card>
        {list.isPending ? (
          <TableSkeleton cols={2} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.items.length === 0 ? (
          <EmptyState title={t('subscribers.empty')} />
        ) : (
          <>
            <TableWrap label={t('subscribers.title')}>
              <thead>
                <tr>
                  <Th className="w-16">#</Th>
                  <Th>{t('subscribers.colEmail')}</Th>
                  <Th>{t('subscribers.colCreated')}</Th>
                </tr>
              </thead>
              <tbody className={list.isPlaceholderData ? 'opacity-60' : undefined}>
                {list.data.items.map((s, i) => (
                  <tr key={s.id} className="hover:bg-surface-2/60">
                    <Td className="text-muted tabular-nums">{formatNumber((list.data.page - 1) * list.data.pageSize + i + 1, locale)}</Td>
                    <Td className="font-medium">
                      <a href={`mailto:${s.email}`} className="text-primary-ink hover:underline">
                        {s.email}
                      </a>
                    </Td>
                    <Td className="whitespace-nowrap text-muted">{formatDate(s.createdAt, locale)}</Td>
                  </tr>
                ))}
              </tbody>
            </TableWrap>
            <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={f.setPage} />
          </>
        )}
      </Card>
    </>
  );
}
