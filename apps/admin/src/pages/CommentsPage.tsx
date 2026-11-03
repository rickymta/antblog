import { useState } from 'react';
import { keepPreviousData, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { ExternalLink, Eye, EyeOff, Filter, Search, Trash2, X } from 'lucide-react';
import { api, formatDate, formatRelative, type AdminComment } from '@tkh/shared';
import {
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  FilterBar,
  IconButton,
  PageHeader,
  Pagination,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  cx,
  inputClass,
  selectClass,
} from '../components/ui';
import { CommentStatusBadge } from '../components/domain';
import { useDebounced, useSearchBox, useUrlFilters } from '../lib/hooks';
import { errorMessage } from '../lib/errors';
import { siteArticleUrl } from '../lib/site';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const PAGE_SIZE = 20;
const STATUSES = ['visible', 'hidden', 'deleted'] as const;

export function CommentsPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const client = useQueryClient();
  const toast = useToast();
  const f = useUrlFilters(['status', 'articleId', 'articleTitle', 'q'] as const);
  const [text, setText] = useSearchBox(f.values.q, (v) => f.set('q', v));
  const [deleting, setDeleting] = useState<AdminComment | null>(null);
  const [deleteError, setDeleteError] = useState<string | undefined>();

  const query = { status: f.values.status, articleId: f.values.articleId, q: f.values.q, page: f.page, pageSize: PAGE_SIZE };
  const list = useQuery({
    queryKey: ['admin', 'comments', query],
    queryFn: () => api.admin.comments.list(query),
    placeholderData: keepPreviousData,
  });

  const invalidate = () => {
    void client.invalidateQueries({ queryKey: ['admin', 'comments'] });
    void client.invalidateQueries({ queryKey: ['admin', 'stats'] });
    void client.invalidateQueries({ queryKey: ['admin', 'articles'] });
  };

  const setStatus = useMutation({
    mutationFn: ({ c, status }: { c: AdminComment; status: 'visible' | 'hidden' }) => api.admin.comments.setStatus(c.id, status),
    onSuccess: (_d, { status }) => {
      toast.success(status === 'hidden' ? t('comments.hidden') : t('comments.shown'));
      invalidate();
    },
    onError: (err) => toast.error(errorMessage(err, t, i18n.language)),
  });

  const remove = useMutation({
    mutationFn: (c: AdminComment) => api.admin.comments.remove(c.id),
    onSuccess: () => {
      setDeleting(null);
      toast.success(t('comments.deleted'));
      invalidate();
    },
    onError: (err) => setDeleteError(errorMessage(err, t, i18n.language)),
  });

  const pendingId = setStatus.isPending ? setStatus.variables?.c.id : undefined;

  const filterArticle = (a: { id: string; title: string }) => {
    f.setMany({ articleId: a.id, articleTitle: a.title });
  };

  return (
    <>
      <PageHeader title={t('comments.title')} subtitle={t('comments.subtitle')} />
      <Card>
        <FilterBar>
          <label className="relative min-w-[14rem] flex-[2]">
            <span className="sr-only">{t('common.search')}</span>
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
            <input
              type="search"
              className={cx(inputClass, 'h-10 pl-9')}
              placeholder={t('comments.searchPlaceholder')}
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
          </label>
          <label className="min-w-[10rem] flex-1">
            <span className="mb-1 block text-xs font-medium text-muted">{t('comments.filterStatus')}</span>
            <select className={selectClass} value={f.values.status} onChange={(e) => f.set('status', e.target.value)}>
              <option value="">{t('common.all')}</option>
              {STATUSES.map((s) => (
                <option key={s} value={s}>
                  {t(`commentStatus.${s}`)}
                </option>
              ))}
            </select>
          </label>
          <div className="min-w-[14rem] flex-[2]">
            <span className="mb-1 block text-xs font-medium text-muted">{t('comments.filterArticle')}</span>
            {f.values.articleId ? (
              <div className="flex h-10 items-center gap-2 rounded-ui border border-primary bg-primary-soft px-3 text-sm text-primary-ink">
                <span className="min-w-0 flex-1 truncate font-medium">{f.values.articleTitle || f.values.articleId}</span>
                <button
                  type="button"
                  aria-label={t('comments.clearArticle')}
                  className="rounded p-0.5 hover:bg-primary/15"
                  onClick={() => f.setMany({ articleId: '', articleTitle: '' })}
                >
                  <X className="size-4" aria-hidden />
                </button>
              </div>
            ) : (
              <ArticleFilterBox onPick={filterArticle} />
            )}
          </div>
          {f.active && (
            <Button variant="ghost" onClick={f.clear}>
              {t('common.clearFilters')}
            </Button>
          )}
        </FilterBar>

        {list.isPending ? (
          <TableSkeleton cols={5} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.items.length === 0 ? (
          <EmptyState title={f.active ? t('states.emptyFiltered') : t('comments.empty')} />
        ) : (
          <>
            <TableWrap label={t('comments.title')}>
              <thead>
                <tr>
                  <Th>{t('comments.colAuthor')}</Th>
                  <Th className="w-[36%] min-w-[14rem]">{t('comments.colBody')}</Th>
                  <Th>{t('comments.colArticle')}</Th>
                  <Th>{t('comments.colStatus')}</Th>
                  <Th>{t('comments.colCreated')}</Th>
                  <Th className="text-right">{t('common.actions')}</Th>
                </tr>
              </thead>
              <tbody className={list.isPlaceholderData ? 'opacity-60' : undefined}>
                {list.data.items.map((c) => (
                  <tr key={c.id} className="align-top hover:bg-surface-2/60">
                    <Td className="min-w-[9rem] align-top">
                      <p className="font-medium">{c.author.displayName}</p>
                      {c.parentId && (
                        <p className="text-xs text-muted">
                          {c.replyTo ? t('comments.replyTo', { name: c.replyTo.displayName }) : t('comments.reply')}
                        </p>
                      )}
                    </Td>
                    <Td className="align-top">
                      {c.status === 'deleted' || !c.body ? (
                        <p className="text-sm text-muted italic">{t('comments.deletedBody')}</p>
                      ) : (
                        <p className="line-clamp-4 text-sm break-words whitespace-pre-line">{c.body}</p>
                      )}
                      {c.editedAt && <p className="mt-0.5 text-xs text-muted">({t('comments.edited')})</p>}
                    </Td>
                    <Td className="min-w-[12rem] max-w-[16rem] align-top">
                      <a
                        href={siteArticleUrl(c.article.slug)}
                        target="_blank"
                        rel="noreferrer"
                        className="inline-flex items-start gap-1 text-sm font-medium text-primary-ink hover:underline"
                      >
                        <span className="line-clamp-2">{c.article.title}</span>
                        <ExternalLink className="mt-0.5 size-3.5 shrink-0" aria-hidden />
                        <span className="sr-only">({t('common.openInSite')})</span>
                      </a>
                      {f.values.articleId !== c.article.id && (
                        <button
                          type="button"
                          onClick={() => filterArticle(c.article)}
                          className="mt-1 inline-flex items-center gap-1 text-xs font-medium text-muted hover:text-ink"
                        >
                          <Filter className="size-3" aria-hidden />
                          {t('comments.filterByArticle')}
                        </button>
                      )}
                    </Td>
                    <Td className="align-top">
                      <CommentStatusBadge status={c.status} />
                    </Td>
                    <Td className="align-top whitespace-nowrap text-muted">
                      <time dateTime={c.createdAt} title={formatDate(c.createdAt, locale)}>
                        {formatRelative(c.createdAt, locale)}
                      </time>
                    </Td>
                    <Td className="text-right align-top whitespace-nowrap">
                      {c.status === 'visible' && (
                        <IconButton
                          icon={EyeOff}
                          label={`${t('comments.hide')}: ${c.author.displayName}`}
                          disabled={pendingId === c.id}
                          onClick={() => setStatus.mutate({ c, status: 'hidden' })}
                        />
                      )}
                      {c.status === 'hidden' && (
                        <IconButton
                          icon={Eye}
                          label={`${t('comments.show')}: ${c.author.displayName}`}
                          disabled={pendingId === c.id}
                          onClick={() => setStatus.mutate({ c, status: 'visible' })}
                        />
                      )}
                      {c.status !== 'deleted' && (
                        <IconButton
                          icon={Trash2}
                          label={`${t('common.delete')}: ${c.author.displayName}`}
                          className="hover:text-red-700 dark:hover:text-red-400"
                          onClick={() => {
                            setDeleteError(undefined);
                            setDeleting(c);
                          }}
                        />
                      )}
                    </Td>
                  </tr>
                ))}
              </tbody>
            </TableWrap>
            <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={f.setPage} />
          </>
        )}
      </Card>
      <ConfirmDialog
        open={deleting !== null}
        title={t('comments.confirmDeleteTitle')}
        body={
          <>
            <p>{t('comments.confirmDeleteBody')}</p>
            {deleting?.body && (
              <blockquote className="mt-3 line-clamp-4 border-l-4 border-primary pl-3 whitespace-pre-line text-ink">{deleting.body}</blockquote>
            )}
          </>
        }
        confirmLabel={t('common.delete')}
        loading={remove.isPending}
        error={deleteError}
        onConfirm={() => deleting && remove.mutate(deleting)}
        onCancel={() => setDeleting(null)}
      />
    </>
  );
}

/** Ô gõ tìm bài viết để lọc bình luận (gợi ý từ danh sách bài quản trị). */
function ArticleFilterBox({ onPick }: { onPick: (a: { id: string; title: string }) => void }) {
  const { t } = useTranslation();
  const [text, setText] = useState('');
  const [open, setOpen] = useState(false);
  const q = useDebounced(text.trim(), 300);
  const results = useQuery({
    queryKey: ['admin', 'articles', 'filter', q],
    queryFn: () => api.admin.articles.list({ q, pageSize: 8 }),
    enabled: open && q.length > 0,
  });
  const listId = 'comment-article-options';
  const items = results.data?.items ?? [];
  return (
    <div className="relative">
      <input
        type="search"
        role="combobox"
        aria-expanded={open && q.length > 0}
        aria-controls={listId}
        aria-autocomplete="list"
        aria-label={t('comments.filterArticle')}
        className={cx(inputClass, 'h-10')}
        placeholder={t('comments.articlePlaceholder')}
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onBlur={() => window.setTimeout(() => setOpen(false), 150)}
        onKeyDown={(e) => {
          if (e.key === 'Escape') setOpen(false);
        }}
      />
      {open && q.length > 0 && (
        <ul id={listId} role="listbox" className="absolute z-20 mt-1 max-h-64 w-full overflow-y-auto rounded-ui border border-line bg-surface py-1 shadow-lg">
          {results.isPending ? (
            <li className="px-3 py-2 text-sm text-muted">{t('common.loading')}</li>
          ) : results.isError ? (
            <li className="px-3 py-2 text-sm text-red-700 dark:text-red-400">{t('states.errorTitle')}</li>
          ) : items.length === 0 ? (
            <li className="px-3 py-2 text-sm text-muted">{t('states.emptyFiltered')}</li>
          ) : (
            items.map((a) => (
              <li key={a.id} role="option" aria-selected={false}>
                <button
                  type="button"
                  className="block w-full truncate px-3 py-2 text-left text-sm text-ink hover:bg-surface-2 focus:bg-surface-2"
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => {
                    onPick({ id: a.id, title: a.title });
                    setText('');
                    setOpen(false);
                  }}
                >
                  {a.title}
                </button>
              </li>
            ))
          )}
        </ul>
      )}
    </div>
  );
}
