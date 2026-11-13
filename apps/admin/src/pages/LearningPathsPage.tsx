import { useState } from 'react';
import { Link } from 'react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Pencil, Plus, Trash2 } from 'lucide-react';
import { api, comparePathOrder, formatDate, formatNumber, pickLocalized, type AdminLearningPath } from '@tkh/shared';
import {
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  IconButton,
  PageHeader,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  buttonClass,
} from '../components/ui';
import { CategoryChip, LevelBadge, PathStatusBadge } from '../components/domain';
import { errorMessage } from '../lib/errors';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';
import { formatMinutes } from '../lib/format';

export const PATHS_KEY = ['admin', 'learning-paths'] as const;

export function LearningPathsPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const client = useQueryClient();
  const toast = useToast();
  const list = useQuery({ queryKey: [...PATHS_KEY, 'list'], queryFn: () => api.admin.learningPaths.list() });
  // Nhóm theo chủ đề, trong mỗi chủ đề theo đúng thứ tự trang đọc hiển thị (ưu tiên → cấp độ → slug); lộ trình không chủ đề xếp cuối.
  const rows = [...(list.data ?? [])].sort((a, b) =>
    Number(!a.category) - Number(!b.category) ||
    (a.category?.slug ?? '').localeCompare(b.category?.slug ?? '') ||
    comparePathOrder(a, b),
  );
  const [deleting, setDeleting] = useState<AdminLearningPath | null>(null);
  const [deleteError, setDeleteError] = useState<string | undefined>();

  const remove = useMutation({
    mutationFn: (p: AdminLearningPath) => api.admin.learningPaths.remove(p.id),
    onSuccess: () => {
      setDeleting(null);
      toast.success(t('paths.deleted'));
      void client.invalidateQueries({ queryKey: PATHS_KEY });
    },
    onError: (err) => setDeleteError(errorMessage(err, t, i18n.language)),
  });

  const newLink = (
    <Link to="/learning-paths/new" className={buttonClass('primary')}>
      <Plus className="size-4" aria-hidden />
      {t('paths.newPath')}
    </Link>
  );

  return (
    <>
      <PageHeader title={t('paths.title')} subtitle={t('paths.subtitle')} actions={newLink} />
      <Card>
        {list.isPending ? (
          <TableSkeleton cols={6} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.length === 0 ? (
          <EmptyState title={t('paths.empty')} action={newLink} />
        ) : (
          <TableWrap label={t('paths.title')}>
            <thead>
              <tr>
                <Th>{t('paths.colTitle')}</Th>
                <Th>{t('paths.colLevel')}</Th>
                <Th className="text-right">{t('paths.colPriority')}</Th>
                <Th>{t('paths.colCategory')}</Th>
                <Th className="text-right">{t('paths.colArticles')}</Th>
                <Th className="text-right">{t('paths.colMinutes')}</Th>
                <Th>{t('paths.colStatus')}</Th>
                <Th>{t('common.updated')}</Th>
                <Th className="text-right">{t('common.actions')}</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((p) => (
                <tr key={p.id} className="hover:bg-surface-2/60">
                  <Td className="max-w-[22rem]">
                    <Link to={`/learning-paths/${p.id}`} className="font-medium text-primary-ink hover:underline">
                      {p.title}
                    </Link>
                    <span className="block truncate font-mono text-xs text-muted">/{p.slug}</span>
                  </Td>
                  <Td>
                    <LevelBadge level={p.level} />
                  </Td>
                  <Td className="text-right tabular-nums">{p.priority ?? t('common.none')}</Td>
                  <Td>{p.category ? <CategoryChip name={pickLocalized(p.category.name, locale)} color={p.category.color} /> : t('common.none')}</Td>
                  <Td className="text-right tabular-nums">{formatNumber(p.articleCount ?? p.articleIds?.length ?? 0, locale)}</Td>
                  <Td className="text-right whitespace-nowrap tabular-nums">{formatMinutes(p.totalMinutes ?? 0, t)}</Td>
                  <Td>
                    <PathStatusBadge status={p.status} />
                  </Td>
                  <Td className="whitespace-nowrap text-muted">{p.updatedAt ? formatDate(p.updatedAt, locale) : t('common.none')}</Td>
                  <Td className="text-right whitespace-nowrap">
                    <Link
                      to={`/learning-paths/${p.id}`}
                      aria-label={`${t('common.edit')}: ${p.title}`}
                      title={t('common.edit')}
                      className="inline-flex size-9 items-center justify-center rounded-ui text-muted hover:bg-surface-2 hover:text-ink"
                    >
                      <Pencil className="size-4.5" aria-hidden />
                    </Link>
                    <IconButton
                      icon={Trash2}
                      label={`${t('common.delete')}: ${p.title}`}
                      className="hover:text-red-700 dark:hover:text-red-400"
                      onClick={() => {
                        setDeleteError(undefined);
                        setDeleting(p);
                      }}
                    />
                  </Td>
                </tr>
              ))}
            </tbody>
          </TableWrap>
        )}
      </Card>
      <ConfirmDialog
        open={deleting !== null}
        title={t('paths.confirmDeleteTitle', { name: deleting?.title ?? '' })}
        body={t('paths.confirmDeleteBody')}
        confirmLabel={t('common.delete')}
        loading={remove.isPending}
        error={deleteError}
        onConfirm={() => deleting && remove.mutate(deleting)}
        onCancel={() => setDeleting(null)}
      />
    </>
  );
}
