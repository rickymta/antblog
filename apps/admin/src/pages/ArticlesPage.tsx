import { useEffect, useMemo, useRef, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Archive, CopyPlus, ExternalLink, PenLine, Plus, Search, Send, Undo2 } from 'lucide-react';
import { api, formatDate, formatNumber, pickLocalized, type AdminArticle, type ArticleInput } from '@tkh/shared';
import {
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  FilterBar,
  PageHeader,
  Pagination,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  buttonClass,
  inputClass,
  selectClass,
} from '../components/ui';
import { ArticleStatusBadge, CategoryChip, LevelBadge } from '../components/domain';
import { Menu, type MenuItem } from '../components/Menu';
import { displayStatus, formatDateTime, type DisplayStatus } from '../lib/articles';
import { useSearchBox, useUrlFilters } from '../lib/hooks';
import { errorMessage } from '../lib/errors';
import { useCategoryOptions } from '../lib/queries';
import { useIsAdmin } from '../lib/session';
import { siteArticleUrl } from '../lib/site';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const PAGE_SIZE = 20;
const STATUSES: DisplayStatus[] = ['draft', 'scheduled', 'published', 'archived'];
type BulkAction = 'publish' | 'unpublish' | 'archive';
type RowConfirm = { kind: 'archive' | 'unpublish'; article: AdminArticle };

export function ArticlesPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const isAdmin = useIsAdmin();
  const navigate = useNavigate();
  const client = useQueryClient();
  const toast = useToast();
  const f = useUrlFilters(['status', 'category', 'q'] as const);
  const [text, setText] = useSearchBox(f.values.q, (v) => f.set('q', v));
  const categories = useCategoryOptions();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [bulk, setBulk] = useState<BulkAction | null>(null);
  const [rowConfirm, setRowConfirm] = useState<RowConfirm | null>(null);
  const selectAllRef = useRef<HTMLInputElement>(null);

  const query = { status: f.values.status, category: f.values.category, q: f.values.q, page: f.page, pageSize: PAGE_SIZE };
  const list = useQuery({
    queryKey: ['admin', 'articles', 'list', query],
    queryFn: () => api.admin.articles.list(query),
    placeholderData: keepPreviousData,
  });
  const items = useMemo(() => list.data?.items ?? [], [list.data]);

  // Đổi trang/bộ lọc ⇒ bỏ chọn (chỉ chọn trong trang đang xem).
  const queryKey = JSON.stringify(query);
  useEffect(() => setSelected(new Set()), [queryKey]);

  const allChecked = items.length > 0 && items.every((a) => selected.has(a.id));
  const someChecked = items.some((a) => selected.has(a.id));
  useEffect(() => {
    if (selectAllRef.current) selectAllRef.current.indeterminate = someChecked && !allChecked;
  }, [someChecked, allChecked]);

  const refresh = () => {
    void client.invalidateQueries({ queryKey: ['admin', 'articles'] });
    void client.invalidateQueries({ queryKey: ['admin', 'stats'] });
  };

  // ---- Thao tác một bài ----
  const rowAction = useMutation({
    mutationFn: async ({ kind, article }: { kind: 'publish' | 'unpublish' | 'archive'; article: AdminArticle }) => {
      if (kind === 'publish') await api.admin.articles.publish(article.id);
      else if (kind === 'unpublish') await api.admin.articles.unpublish(article.id);
      else await api.admin.articles.archive(article.id);
      return { kind, article };
    },
    onSuccess: ({ kind, article }) => {
      const key = kind === 'publish' ? 'articleActions.publishedOne' : kind === 'unpublish' ? 'articleActions.unpublishedOne' : 'articleActions.archivedOne';
      toast.success(t(key, { title: article.title }));
      setRowConfirm(null);
      refresh();
    },
    onError: (err) => {
      setRowConfirm(null);
      toast.error(errorMessage(err, t, i18n.language));
    },
  });

  /** Nhân bản thành nháp: đọc bài đầy đủ rồi tạo bài mới phía giao diện (slug tự sinh, tiêu đề thêm "(bản sao)"). */
  const duplicate = useMutation({
    mutationFn: async (a: AdminArticle) => {
      const full = await api.admin.articles.get(a.id);
      const suffix = ` ${t('articleActions.copySuffix')}`;
      const input: ArticleInput = {
        title: (full.title.slice(0, 200 - suffix.length) + suffix).trim(),
        summary: full.summary ?? '',
        content: full.content ?? '',
        coverImageUrl: full.coverImageUrl,
        language: full.language,
        categoryId: full.category.id,
        level: full.level,
        tags: full.tags ?? [],
        seo: full.seo,
      };
      return api.admin.articles.create(input);
    },
    onSuccess: (created) => {
      toast.success(t('articleActions.duplicated', { title: created.title }));
      refresh();
      navigate(`/articles/${created.id}`);
    },
    onError: (err) => toast.error(errorMessage(err, t, i18n.language)),
  });

  // ---- Thao tác hàng loạt (chỉ áp cho bài đổi được trạng thái; bài đã đúng trạng thái bỏ qua) ----
  const selectedArticles = items.filter((a) => selected.has(a.id));
  const runBulk = useMutation({
    mutationFn: async (action: BulkAction) => {
      const targets = selectedArticles.filter((a) => {
        const s = displayStatus(a);
        if (action === 'publish') return s !== 'published';
        if (action === 'unpublish') return s === 'published' || s === 'scheduled';
        return s !== 'archived';
      });
      const results = await Promise.allSettled(
        targets.map((a) =>
          action === 'publish'
            ? api.admin.articles.publish(a.id)
            : action === 'unpublish'
              ? api.admin.articles.unpublish(a.id)
              : api.admin.articles.archive(a.id),
        ),
      );
      const failed = results.filter((r): r is PromiseRejectedResult => r.status === 'rejected');
      return { ok: results.length - failed.length, failed };
    },
    onSuccess: ({ ok, failed }) => {
      if (ok) toast.success(t('articleActions.bulkDone', { count: ok }));
      if (failed.length) toast.error(t('articleActions.bulkFailed', { count: failed.length, message: errorMessage(failed[0]!.reason, t, i18n.language) }));
      setBulk(null);
      setSelected(new Set());
      refresh();
    },
    onError: (err) => {
      setBulk(null);
      toast.error(errorMessage(err, t, i18n.language));
    },
  });

  const toggleOne = (id: string) =>
    setSelected((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  const toggleAll = () => setSelected(allChecked ? new Set() : new Set(items.map((a) => a.id)));

  const rowMenu = (a: AdminArticle): MenuItem[] => {
    const s = displayStatus(a);
    const out: MenuItem[] = [{ key: 'edit', label: t('articleActions.edit'), icon: PenLine, onSelect: () => navigate(`/articles/${a.id}`) }];
    if (s === 'published') out.push({ key: 'view', label: t('articleActions.view'), icon: ExternalLink, href: siteArticleUrl(a.slug) });
    out.push({ key: 'dup', label: t('articleActions.duplicate'), icon: CopyPlus, onSelect: () => duplicate.mutate(a), disabled: duplicate.isPending });
    if (s !== 'published') out.push({ key: 'pub', label: t('articleActions.publish'), icon: Send, onSelect: () => rowAction.mutate({ kind: 'publish', article: a }) });
    if (s === 'published' || s === 'scheduled') {
      out.push({ key: 'unpub', label: t('articleActions.unpublish'), icon: Undo2, onSelect: () => setRowConfirm({ kind: 'unpublish', article: a }) });
    }
    if (s !== 'archived') out.push({ key: 'arch', label: t('articleActions.archive'), icon: Archive, danger: true, onSelect: () => setRowConfirm({ kind: 'archive', article: a }) });
    return out;
  };

  const bulkLabel = (b: BulkAction) => t(b === 'publish' ? 'articleActions.bulkPublish' : b === 'unpublish' ? 'articleActions.bulkUnpublish' : 'articleActions.bulkArchive');

  return (
    <>
      <PageHeader
        title={t('articles.title')}
        subtitle={isAdmin ? t('articles.subtitle') : t('articles.authorSubtitle')}
        actions={
          <Link to="/articles/new" className={buttonClass('primary')}>
            <Plus className="size-4" aria-hidden />
            {t('articles.newArticle')}
          </Link>
        }
      />
      <Card>
        <FilterBar>
          <label className="relative min-w-[14rem] flex-[2]">
            <span className="sr-only">{t('common.search')}</span>
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
            <input
              type="search"
              className={`${inputClass} h-10 pl-9`}
              placeholder={t('articles.searchPlaceholder')}
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
          </label>
          <label className="min-w-[10rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('articles.filterStatus')}</span>
            <select className={selectClass} value={f.values.status} onChange={(e) => f.set('status', e.target.value)}>
              {/* API: không lọc trạng thái ⇒ ẩn bài lưu trữ. */}
              <option value="">{t('articles.statusAllActive')}</option>
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {t(`articleStatus.${s}`)}
                </option>
              ))}
            </select>
          </label>
          <label className="min-w-[10rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('articles.filterCategory')}</span>
            <select className={selectClass} value={f.values.category} onChange={(e) => f.set('category', e.target.value)}>
              <option value="">{t('common.all')}</option>
              {categories.data?.map((c) => (
                <option key={c.id} value={c.slug}>
                  {pickLocalized(c.name, locale)}
                </option>
              ))}
            </select>
          </label>
          {f.active && (
            <Button variant="ghost" onClick={f.clear}>
              {t('common.clearFilters')}
            </Button>
          )}
        </FilterBar>

        {selected.size > 0 && (
          <div className="flex flex-wrap items-center gap-2 border-b border-line bg-primary-soft/60 px-4 py-2.5">
            <span className="mr-2 text-sm font-semibold text-ink" aria-live="polite">
              {t('articleActions.selectedCount', { count: selected.size })}
            </span>
            <Button size="sm" icon={Send} onClick={() => setBulk('publish')}>
              {t('articleActions.bulkPublish')}
            </Button>
            <Button size="sm" icon={Undo2} onClick={() => setBulk('unpublish')}>
              {t('articleActions.bulkUnpublish')}
            </Button>
            <Button size="sm" icon={Archive} onClick={() => setBulk('archive')}>
              {t('articleActions.bulkArchive')}
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setSelected(new Set())}>
              {t('articleActions.clearSelection')}
            </Button>
          </div>
        )}

        {list.isPending ? (
          <TableSkeleton cols={7} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : items.length === 0 ? (
          f.active ? (
            <EmptyState title={t('states.emptyFiltered')} action={<Button onClick={f.clear}>{t('common.clearFilters')}</Button>} />
          ) : (
            <EmptyState
              title={t('articles.empty')}
              body={t('articles.emptyHint')}
              action={
                <Link to="/articles/new" className={buttonClass('primary')}>
                  {t('articles.newArticle')}
                </Link>
              }
            />
          )
        ) : (
          <>
            <TableWrap label={t('articles.title')}>
              <thead>
                <tr>
                  <Th className="w-10">
                    <input
                      ref={selectAllRef}
                      type="checkbox"
                      className="size-4 accent-[var(--color-primary)]"
                      aria-label={t('articleActions.selectAll')}
                      checked={allChecked}
                      onChange={toggleAll}
                    />
                  </Th>
                  <Th>{t('articles.colTitle')}</Th>
                  <Th>{t('articles.colCategory')}</Th>
                  <Th>{t('articles.colLevel')}</Th>
                  <Th>{t('articles.colStatus')}</Th>
                  <Th>{t('articles.colUpdated')}</Th>
                  <Th className="text-right">{t('articles.colViews')}</Th>
                  <Th className="text-right">{t('articles.colComments')}</Th>
                  <Th className="text-right">
                    <span className="sr-only">{t('articleActions.colActions')}</span>
                  </Th>
                </tr>
              </thead>
              <tbody className={list.isPlaceholderData ? 'opacity-60' : undefined}>
                {items.map((a) => {
                  const s = displayStatus(a);
                  return (
                    <tr key={a.id} className={selected.has(a.id) ? 'bg-primary-soft/40' : 'hover:bg-surface-2/60'}>
                      <Td>
                        <input
                          type="checkbox"
                          className="size-4 accent-[var(--color-primary)]"
                          aria-label={t('articleActions.selectRow', { title: a.title })}
                          checked={selected.has(a.id)}
                          onChange={() => toggleOne(a.id)}
                        />
                      </Td>
                      <Td className="max-w-[26rem] min-w-[16rem]">
                        <Link to={`/articles/${a.id}`} className="line-clamp-2 font-medium text-primary-ink hover:underline">
                          {a.title}
                        </Link>
                        <span className="mt-0.5 block truncate font-mono text-xs text-muted">/{a.slug}</span>
                      </Td>
                      <Td>{a.category ? <CategoryChip name={pickLocalized(a.category.name, locale)} color={a.category.color} /> : t('common.none')}</Td>
                      <Td>
                        <LevelBadge level={a.level} />
                      </Td>
                      <Td className="whitespace-nowrap">
                        <ArticleStatusBadge status={s} />
                        {s === 'scheduled' && a.publishedAt && (
                          <span className="mt-1 block text-xs text-primary-ink">{t('articleActions.scheduledFor', { date: formatDateTime(a.publishedAt, locale) })}</span>
                        )}
                        {s === 'published' && a.publishedAt && <span className="mt-1 block text-xs text-muted">{formatDate(a.publishedAt, locale)}</span>}
                      </Td>
                      <Td className="whitespace-nowrap text-muted">{formatDate(a.updatedAt, locale)}</Td>
                      <Td className="text-right tabular-nums">{formatNumber(a.stats?.views ?? 0, locale)}</Td>
                      <Td className="text-right tabular-nums">{formatNumber(a.stats?.comments ?? 0, locale)}</Td>
                      <Td className="text-right whitespace-nowrap">
                        <Link to={`/articles/${a.id}`} className={buttonClass('ghost', 'sm', 'px-2')} aria-label={`${t('articleActions.edit')}: ${a.title}`}>
                          <PenLine className="size-4" aria-hidden />
                        </Link>
                        <Menu label={t('articleActions.more', { title: a.title })} items={rowMenu(a)} />
                      </Td>
                    </tr>
                  );
                })}
              </tbody>
            </TableWrap>
            <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={f.setPage} />
          </>
        )}
      </Card>

      <ConfirmDialog
        open={rowConfirm !== null}
        title={
          rowConfirm?.kind === 'archive'
            ? t('articleActions.confirmArchiveTitle', { title: rowConfirm.article.title })
            : t('articleActions.confirmUnpublishTitle', { title: rowConfirm?.article.title ?? '' })
        }
        body={rowConfirm?.kind === 'archive' ? t('articleActions.confirmArchiveBody') : t('articleActions.confirmUnpublishBody')}
        confirmLabel={rowConfirm?.kind === 'archive' ? t('articleActions.archive') : t('articleActions.unpublish')}
        tone={rowConfirm?.kind === 'archive' ? 'danger' : 'primary'}
        loading={rowAction.isPending}
        onConfirm={() => rowConfirm && rowAction.mutate({ kind: rowConfirm.kind, article: rowConfirm.article })}
        onCancel={() => setRowConfirm(null)}
      />
      <ConfirmDialog
        open={bulk !== null}
        title={bulk ? t('articleActions.confirmBulkTitle', { action: bulkLabel(bulk), count: selected.size }) : ''}
        body={
          bulk === 'publish'
            ? t('articleActions.confirmBulkPublishBody')
            : bulk === 'unpublish'
              ? t('articleActions.confirmBulkUnpublishBody')
              : t('articleActions.confirmBulkArchiveBody')
        }
        confirmLabel={bulk ? bulkLabel(bulk) : ''}
        tone={bulk === 'archive' ? 'danger' : 'primary'}
        loading={runBulk.isPending}
        onConfirm={() => bulk && runBulk.mutate(bulk)}
        onCancel={() => setBulk(null)}
      />
    </>
  );
}
