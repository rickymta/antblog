import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import { Link, useBlocker, useNavigate, useParams } from 'react-router';
import { useMutation, useQueries, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { ArrowDown, ArrowLeft, ArrowUp, Check, Plus, Save, Search, X } from 'lucide-react';
import {
  api,
  LEVELS,
  pickLocalized,
  slugify,
  type AdminArticle,
  type AdminLearningPath,
  type ArticleStatus,
  type Level,
  type LearningPathInput,
} from '@tkh/shared';
import { Button, Card, ConfirmDialog, ErrorState, Field, IconButton, Spinner, cx, inputClass, selectClass } from '../components/ui';
import { ArticleStatusBadge, LevelBadge } from '../components/domain';
import { errorMessage, hasCode, hasStatus, isApiError, topLevelFields } from '../lib/errors';
import { useDebounced } from '../lib/hooks';
import { useCategoryOptions } from '../lib/queries';
import { SLUG_RE } from '../lib/site';
import { useToast } from '../lib/toast';
import { formatMinutes } from '../lib/format';
import { useLocale } from '../i18n';
import { PATHS_KEY } from './LearningPathsPage';

interface ArticleInfo {
  id: string;
  title: string;
  readingMinutes: number;
  status?: ArticleStatus;
}

interface FormState {
  slug: string;
  title: string;
  description: string;
  level: Level;
  /** Chuỗi để ô số để trống được; rỗng ⇒ không đặt thứ tự ưu tiên. */
  priority: string;
  categoryId: string;
  status: 'draft' | 'published';
  articleIds: string[];
}
type Errors = Partial<Record<keyof FormState, string>>;

function fromPath(p?: AdminLearningPath): FormState {
  return {
    slug: p?.slug ?? '',
    title: p?.title ?? '',
    description: p?.description ?? '',
    level: p?.level ?? 'beginner',
    priority: p?.priority === undefined ? '' : String(p.priority),
    categoryId: p?.categoryId ?? '',
    status: p?.status ?? 'draft',
    articleIds: p?.articleIds ?? [],
  };
}

function toInfo(a: AdminArticle): ArticleInfo {
  return { id: a.id, title: a.title, readingMinutes: a.readingMinutes ?? 0, status: a.status };
}

export function LearningPathEditorPage() {
  const { id } = useParams();
  const { t, i18n } = useTranslation();
  const detail = useQuery({
    queryKey: [...PATHS_KEY, 'detail', id],
    queryFn: () => api.admin.learningPaths.get(id ?? ''),
    enabled: Boolean(id),
  });
  if (!id) return <PathEditor key="new" path={undefined} />;
  if (detail.isPending) return <Spinner label={t('common.loading')} />;
  if (detail.isError) {
    return (
      <Card>
        <ErrorState
          message={errorMessage(detail.error, t, i18n.language)}
          onRetry={hasStatus(detail.error, 404) ? undefined : () => void detail.refetch()}
        />
        <div className="flex justify-center pb-8">
          <Link to="/learning-paths" className="text-sm font-medium text-primary-ink hover:underline">
            {t('common.back')}
          </Link>
        </div>
      </Card>
    );
  }
  return <PathEditor key={id} path={detail.data} />;
}

function PathEditor({ path: initialPath }: { path: AdminLearningPath | undefined }) {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const navigate = useNavigate();
  const client = useQueryClient();
  const toast = useToast();
  const categories = useCategoryOptions();

  const [path, setPath] = useState(initialPath);
  const [baseline, setBaseline] = useState<FormState>(() => fromPath(initialPath));
  const [form, setForm] = useState<FormState>(baseline);
  const [slugTouched, setSlugTouched] = useState(Boolean(initialPath));
  const [errors, setErrors] = useState<Errors>({});
  const [known, setKnown] = useState<Record<string, ArticleInfo>>({});
  const allowNav = useRef(false);
  const dirty = JSON.stringify(form) !== JSON.stringify(baseline);

  // Tên/thời lượng của các bài đã có trong lộ trình (lộ trình chỉ trả id).
  const initialIds = initialPath?.articleIds ?? [];
  const infoQueries = useQueries({
    queries: initialIds.map((aid) => ({
      queryKey: ['admin', 'articles', 'detail', aid],
      queryFn: () => api.admin.articles.get(aid),
      staleTime: 60_000,
    })),
  });
  const infoOf = (aid: string): ArticleInfo | undefined => {
    if (known[aid]) return known[aid];
    const idx = initialIds.indexOf(aid);
    const data = idx >= 0 ? infoQueries[idx]?.data : undefined;
    return data ? toInfo(data) : undefined;
  };
  const infoState = (aid: string): 'loading' | 'error' | 'ok' => {
    if (known[aid]) return 'ok';
    const idx = initialIds.indexOf(aid);
    const q = idx >= 0 ? infoQueries[idx] : undefined;
    if (!q) return 'error';
    return q.isPending ? 'loading' : q.isError ? 'error' : 'ok';
  };

  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => {
    setForm((f) => {
      const next = { ...f, [k]: v };
      if (k === 'title' && !slugTouched) next.slug = slugify(String(v));
      return next;
    });
    setErrors((e) => ({ ...e, [k]: undefined }));
  };

  const move = (idx: number, delta: -1 | 1) => {
    const ids = [...form.articleIds];
    const j = idx + delta;
    if (j < 0 || j >= ids.length) return;
    [ids[idx], ids[j]] = [ids[j]!, ids[idx]!];
    set('articleIds', ids);
  };

  const addArticle = (a: AdminArticle) => {
    if (form.articleIds.includes(a.id) || form.articleIds.length >= 10) return;
    setKnown((k) => ({ ...k, [a.id]: toInfo(a) }));
    set('articleIds', [...form.articleIds, a.id]);
  };

  const publishedInfos = form.articleIds.map(infoOf).filter((x): x is ArticleInfo => Boolean(x) && x?.status === 'published');
  const totalMinutes = publishedInfos.reduce((s, a) => s + a.readingMinutes, 0);
  const hasUnpublished = form.articleIds.some((aid) => {
    const info = infoOf(aid);
    return info && info.status !== 'published';
  });

  // Rời trang khi chưa lưu.
  const blocker = useBlocker(({ currentLocation, nextLocation }) => dirty && !allowNav.current && currentLocation.pathname !== nextLocation.pathname);
  useEffect(() => {
    if (!dirty) return;
    const h = (e: BeforeUnloadEvent) => e.preventDefault();
    window.addEventListener('beforeunload', h);
    return () => window.removeEventListener('beforeunload', h);
  }, [dirty]);

  const save = useMutation({
    mutationFn: async (input: LearningPathInput) => {
      const wasNew = !path;
      const saved = path ? await api.admin.learningPaths.update(path.id, input) : await api.admin.learningPaths.create(input);
      return { wasNew, saved };
    },
    onSuccess: ({ wasNew, saved }) => {
      const next = { ...fromPath(saved), articleIds: saved.articleIds ?? form.articleIds };
      setPath(saved);
      setBaseline(next);
      setForm(next);
      setSlugTouched(true);
      client.setQueryData([...PATHS_KEY, 'detail', saved.id], saved);
      void client.invalidateQueries({ queryKey: [...PATHS_KEY, 'list'] });
      // Bài bị chuyển lộ trình ⇒ learningPathId đổi.
      void client.invalidateQueries({ queryKey: ['admin', 'articles'] });
      toast.success(wasNew ? t('paths.created') : t('paths.saved'));
      if (wasNew) {
        allowNav.current = true;
        navigate(`/learning-paths/${saved.id}`, { replace: true });
      }
    },
    onError: (err) => {
      if (isApiError(err) && err.fields) setErrors(topLevelFields(err.fields) as Errors);
      else if (hasCode(err, 'SLUG_TAKEN')) setErrors({ slug: errorMessage(err, t, i18n.language) });
      toast.error(errorMessage(err, t, i18n.language));
    },
  });

  const submit = (e?: FormEvent) => {
    e?.preventDefault();
    const v: Errors = {};
    const title = form.title.trim();
    if (form.articleIds.length > 10) v.articleIds = t("paths.partLimit");
    if (title.length < 3 || title.length > 200) v.title = t('paths.errTitle');
    if (!SLUG_RE.test(form.slug.trim())) v.slug = t('paths.errSlug');
    const priorityText = form.priority.trim();
    if (priorityText !== '' && !/^\d{1,6}$/.test(priorityText)) v.priority = t('paths.errPriority');
    else if (priorityText !== '' && Number(priorityText) > 100000) v.priority = t('paths.errPriority');
    setErrors(v);
    if (Object.keys(v).length > 0) {
      toast.error(t('errors.validation'));
      return;
    }
    save.mutate({
      slug: form.slug.trim(),
      title: form.title.trim(),
      description: form.description.trim(),
      level: form.level,
      // Vắng ⇒ API xoá thứ tự ưu tiên của lộ trình.
      priority: priorityText === '' ? undefined : Number(priorityText),
      categoryId: form.categoryId || undefined,
      articleIds: form.articleIds,
      status: form.status,
    });
  };

  return (
    <>
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div>
          <Link to="/learning-paths" className="mb-2 inline-flex items-center gap-1 text-sm font-medium text-muted hover:text-ink">
            <ArrowLeft className="size-4" aria-hidden />
            {t('nav.learningPaths')}
          </Link>
          <h1 className="text-2xl font-bold tracking-tight text-ink">{path ? t('paths.editPath') : t('paths.newPath')}</h1>
          <p className="mt-1 text-sm" aria-live="polite">
            {dirty && <span className="font-medium text-amber-700 dark:text-amber-400">{t('editor.unsaved')}</span>}
          </p>
        </div>
        <Button variant="primary" icon={Save} loading={save.isPending} disabled={!dirty && Boolean(path)} onClick={() => submit()}>
          {t('common.save')}
        </Button>
      </div>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]">
        <Card className="h-fit p-4 sm:p-5">
          <form className="space-y-4" onSubmit={submit} noValidate>
            <Field label={t('paths.fieldTitle')} error={errors.title} required>
              {(a) => <input {...a} className={inputClass} value={form.title} onChange={(e) => set('title', e.target.value)} />}
            </Field>
            <Field label={t('paths.fieldSlug')} error={errors.slug} required hint={t('editor.slugHint')}>
              {(a) => (
                <input
                  {...a}
                  className={cx(inputClass, 'font-mono')}
                  value={form.slug}
                  spellCheck={false}
                  onChange={(e) => {
                    setSlugTouched(true);
                    set('slug', e.target.value.toLowerCase());
                  }}
                />
              )}
            </Field>
            <Field label={t('paths.fieldDescription')} error={errors.description}>
              {(a) => <textarea {...a} rows={4} className={inputClass} value={form.description} onChange={(e) => set('description', e.target.value)} />}
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label={t('paths.fieldLevel')} error={errors.level}>
                {(a) => (
                  <select {...a} className={selectClass} value={form.level} onChange={(e) => set('level', e.target.value as Level)}>
                    {LEVELS.map((l) => (
                      <option key={l} value={l}>
                        {t(`level.${l}`)}
                      </option>
                    ))}
                  </select>
                )}
              </Field>
              <Field label={t('paths.fieldStatus')} error={errors.status}>
                {(a) => (
                  <select {...a} className={selectClass} value={form.status} onChange={(e) => set('status', e.target.value as 'draft' | 'published')}>
                    <option value="draft">{t('pathStatus.draft')}</option>
                    <option value="published">{t('pathStatus.published')}</option>
                  </select>
                )}
              </Field>
            </div>
            <Field label={t('paths.fieldCategory')} error={errors.categoryId}>
              {(a) => (
                <select {...a} className={selectClass} value={form.categoryId} onChange={(e) => set('categoryId', e.target.value)}>
                  <option value="">{t('paths.noCategory')}</option>
                  {categories.data?.map((c) => (
                    <option key={c.id} value={c.id}>
                      {pickLocalized(c.name, locale)}
                    </option>
                  ))}
                </select>
              )}
            </Field>
            <Field label={t('paths.fieldPriority')} error={errors.priority} hint={t('paths.priorityHint')}>
              {(a) => (
                <input
                  {...a}
                  type="number"
                  inputMode="numeric"
                  min={0}
                  max={100000}
                  step={1}
                  className={inputClass}
                  value={form.priority}
                  onChange={(e) => set('priority', e.target.value)}
                />
              )}
            </Field>
            {/* Enter trong form ⇒ lưu */}
            <button type="submit" className="hidden" tabIndex={-1} aria-hidden />
          </form>
        </Card>

        <Card className="min-w-0">
          <div className="border-b border-line px-4 py-3">
            <h2 className="text-base font-semibold text-ink">{t('paths.articlesTitle')}</h2>
            <p className="mt-0.5 text-xs text-muted">{t('paths.articlesHint')}</p>
          </div>
          <div className="space-y-4 p-4">
            <p className="text-sm text-muted">{form.articleIds.length}/10 · {t("paths.partLimit")}</p>
            <ArticlePicker selected={form.articleIds} onAdd={addArticle} />
            {errors.articleIds && <p className="text-xs font-medium text-red-700 dark:text-red-400">{errors.articleIds}</p>}
            {form.articleIds.length === 0 ? (
              <p className="rounded-ui border border-dashed border-line p-6 text-center text-sm text-muted">{t('paths.noArticles')}</p>
            ) : (
              <ol className="divide-y divide-line rounded-ui border border-line">
                {form.articleIds.map((aid, idx) => {
                  const info = infoOf(aid);
                  const st = infoState(aid);
                  const title = info?.title ?? (st === 'loading' ? t('common.loading') : t('paths.articleUnavailable'));
                  return (
                    <li key={aid} className="flex items-center gap-3 px-3 py-2">
                      <span className="w-6 shrink-0 text-right text-sm font-semibold text-muted tabular-nums">{idx + 1}.</span>
                      <div className="min-w-0 flex-1">
                        <p className={cx('truncate text-sm font-medium', info ? 'text-ink' : 'text-muted italic')}>{title}</p>
                        <p className="flex items-center gap-2 text-xs text-muted">
                          {info && <span>{t('common.minutes', { count: info.readingMinutes })}</span>}
                          {info?.status && info.status !== 'published' && <ArticleStatusBadge status={info.status} />}
                        </p>
                      </div>
                      <div className="flex shrink-0">
                        <IconButton icon={ArrowUp} label={`${t('common.moveUp')}: ${title}`} disabled={idx === 0} onClick={() => move(idx, -1)} />
                        <IconButton
                          icon={ArrowDown}
                          label={`${t('common.moveDown')}: ${title}`}
                          disabled={idx === form.articleIds.length - 1}
                          onClick={() => move(idx, 1)}
                        />
                        <IconButton
                          icon={X}
                          label={`${t('common.remove')}: ${title}`}
                          className="hover:text-red-700 dark:hover:text-red-400"
                          onClick={() => set('articleIds', form.articleIds.filter((x) => x !== aid))}
                        />
                      </div>
                    </li>
                  );
                })}
              </ol>
            )}
            <div className="rounded-ui bg-surface-2 px-3 py-2 text-sm">
              <p className="font-medium text-ink">
                {t('paths.totalMinutes', { minutes: formatMinutes(totalMinutes, t), count: publishedInfos.length })}
              </p>
              {hasUnpublished && <p className="mt-0.5 text-xs text-muted">{t('paths.notPublishedNote')}</p>}
            </div>
          </div>
        </Card>
      </div>

      <ConfirmDialog
        open={blocker.state === 'blocked'}
        title={t('editor.leaveTitle')}
        body={t('editor.leaveBody')}
        confirmLabel={t('editor.leave')}
        cancelLabel={t('editor.stay')}
        onConfirm={() => blocker.proceed?.()}
        onCancel={() => blocker.reset?.()}
      />
    </>
  );
}

/** Ô tìm bài đã xuất bản để thêm vào lộ trình. */
function ArticlePicker({ selected, onAdd }: { selected: string[]; onAdd: (a: AdminArticle) => void }) {
  const { t, i18n } = useTranslation();
  const [text, setText] = useState('');
  const q = useDebounced(text.trim(), 300);
  const results = useQuery({
    queryKey: ['admin', 'articles', 'picker', q],
    queryFn: () => api.admin.articles.list({ status: 'published', q, pageSize: 8 }),
  });
  const items = useMemo(() => results.data?.items ?? [], [results.data]);

  return (
    <div>
      <label htmlFor="path-picker" className="mb-1.5 block text-sm font-medium text-ink">
        {t('paths.pickerLabel')}
      </label>
      <div className="relative">
        <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
        <input
          id="path-picker"
          type="search"
          className={cx(inputClass, 'h-10 pl-9')}
          placeholder={t('paths.pickerPlaceholder')}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </div>
      <div className="mt-2 max-h-64 overflow-y-auto rounded-ui border border-line" aria-live="polite">
        {results.isPending ? (
          <p className="p-3 text-sm text-muted">{t('common.loading')}</p>
        ) : results.isError ? (
          <div className="flex items-center justify-between gap-2 p-3 text-sm">
            <span className="text-red-700 dark:text-red-400">{errorMessage(results.error, t, i18n.language)}</span>
            <Button size="sm" onClick={() => void results.refetch()}>
              {t('common.retry')}
            </Button>
          </div>
        ) : items.length === 0 ? (
          <p className="p-3 text-sm text-muted">{t('paths.pickerEmpty')}</p>
        ) : (
          <ul className="divide-y divide-line">
            {items.map((a) => {
              const added = selected.includes(a.id);
              return (
                <li key={a.id} className="flex items-center gap-3 px-3 py-2">
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium text-ink">{a.title}</p>
                    <p className="flex items-center gap-2 text-xs text-muted">
                      <LevelBadge level={a.level} />
                      {t('common.minutes', { count: a.readingMinutes ?? 0 })}
                    </p>
                  </div>
                  {added ? (
                    <span className="inline-flex items-center gap-1 text-xs font-medium text-emerald-700 dark:text-emerald-400">
                      <Check className="size-4" aria-hidden />
                      {t('paths.alreadyAdded')}
                    </span>
                  ) : (
                    <Button size="sm" disabled={selected.length >= 10} icon={Plus} onClick={() => onAdd(a)} aria-label={`${t('common.add')}: ${a.title}`}>
                      {t('common.add')}
                    </Button>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
