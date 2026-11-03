import { useCallback, useDeferredValue, useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import { Link, useBlocker, useNavigate, useParams } from 'react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Archive, ArrowLeft, Columns2, Eye, ExternalLink, PenLine, RefreshCw, Save, Send, Undo2, X } from 'lucide-react';
import {
  api,
  formatDate,
  LEVELS,
  pickLocalized,
  slugify,
  type AdminArticle,
  type ArticleInput,
  type Level,
  type Locale,
} from '@tkh/shared';
import { ArticleContent } from '@tkh/ui';
import { Button, Card, ConfirmDialog, ErrorState, Field, Spinner, cx, inputClass, selectClass } from '../components/ui';
import { ArticleStatusBadge } from '../components/domain';
import { errorMessage, hasCode, hasStatus, isApiError, topLevelFields } from '../lib/errors';
import { HTTP_URL_RE, SLUG_RE, siteArticleUrl } from '../lib/site';
import { useCategoryOptions } from '../lib/queries';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const MAX_TAGS = 10;
const SUMMARY_MAX = 400;
type Mode = 'edit' | 'preview' | 'split';
const MODE_KEY = 'tkh-admin-editor-mode';

interface FormState {
  title: string;
  slug: string;
  summary: string;
  categoryId: string;
  level: Level;
  language: Locale;
  tags: string[];
  coverImageUrl: string;
  content: string;
}

type FieldKey = keyof FormState;
type Errors = Partial<Record<FieldKey, string>>;

const EMPTY: FormState = {
  title: '',
  slug: '',
  summary: '',
  categoryId: '',
  level: 'beginner',
  language: 'vi',
  tags: [],
  coverImageUrl: '',
  content: '',
};

function fromArticle(a: AdminArticle): FormState {
  return {
    title: a.title,
    slug: a.slug,
    summary: a.summary ?? '',
    categoryId: a.category?.id ?? '',
    level: a.level,
    language: a.language,
    tags: a.tags ?? [],
    coverImageUrl: a.coverImageUrl ?? '',
    content: a.content ?? '',
  };
}

function toInput(f: FormState): ArticleInput {
  return {
    title: f.title.trim(),
    slug: f.slug.trim() || undefined,
    summary: f.summary.trim(),
    content: f.content,
    coverImageUrl: f.coverImageUrl.trim() || undefined,
    language: f.language,
    categoryId: f.categoryId,
    level: f.level,
    tags: f.tags,
  };
}

function normalizeTag(raw: string): string {
  return raw.trim().replace(/^#+/, '').toLowerCase().replace(/\s+/g, '-');
}

function readMode(): Mode {
  try {
    const v = localStorage.getItem(MODE_KEY);
    if (v === 'edit' || v === 'preview' || v === 'split') return v;
  } catch {
    /* bỏ qua */
  }
  return 'split';
}

// ---------------- Trang: tải bài rồi dựng trình soạn ----------------

export function ArticleEditorPage() {
  const { id } = useParams();
  const { t, i18n } = useTranslation();
  const detail = useQuery({
    queryKey: ['admin', 'articles', 'detail', id],
    queryFn: () => api.admin.articles.get(id ?? ''),
    enabled: Boolean(id),
  });

  if (!id) return <ArticleEditor key="new" article={undefined} />;
  if (detail.isPending) return <Spinner label={t('common.loading')} />;
  if (detail.isError) {
    const forbidden = hasStatus(detail.error, 403);
    const notFound = hasStatus(detail.error, 404);
    return (
      <Card>
        <ErrorState
          title={forbidden ? t('editor.forbiddenTitle') : notFound ? t('editor.notFoundTitle') : undefined}
          message={forbidden ? t('editor.forbiddenBody') : errorMessage(detail.error, t, i18n.language)}
          onRetry={forbidden || notFound ? undefined : () => void detail.refetch()}
        />
        <div className="flex justify-center pb-8">
          <Link to="/articles" className="text-sm font-medium text-primary-ink hover:underline">
            {t('common.back')}
          </Link>
        </div>
      </Card>
    );
  }
  return <ArticleEditor key={id} article={detail.data} />;
}

// ---------------- Trình soạn ----------------

function ArticleEditor({ article: initialArticle }: { article: AdminArticle | undefined }) {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const navigate = useNavigate();
  const client = useQueryClient();
  const toast = useToast();
  const categories = useCategoryOptions();

  const [article, setArticle] = useState<AdminArticle | undefined>(initialArticle);
  const [baseline, setBaseline] = useState<FormState>(() => (initialArticle ? fromArticle(initialArticle) : EMPTY));
  const [form, setForm] = useState<FormState>(baseline);
  const [slugTouched, setSlugTouched] = useState(Boolean(initialArticle));
  const [errors, setErrors] = useState<Errors>({});
  const [showErrors, setShowErrors] = useState(false);
  const [mode, setModeState] = useState<Mode>(readMode);
  const [confirm, setConfirm] = useState<'unpublish' | 'archive' | null>(null);
  const [tagText, setTagText] = useState('');
  const [coverBroken, setCoverBroken] = useState(false);
  const allowNav = useRef(false);

  const isNew = !article;
  const status = article?.status ?? 'draft';
  const dirty = useMemo(() => JSON.stringify(form) !== JSON.stringify(baseline), [form, baseline]);

  const setMode = (m: Mode) => {
    setModeState(m);
    try {
      localStorage.setItem(MODE_KEY, m);
    } catch {
      /* bỏ qua */
    }
  };

  const update = <K extends FieldKey>(key: K, value: FormState[K]) => {
    setForm((f) => {
      const next = { ...f, [key]: value };
      if (key === 'title' && !slugTouched) next.slug = slugify(String(value));
      return next;
    });
    setErrors((e) => {
      if (!e[key] && !(key === 'title' && e.slug)) return e;
      const next = { ...e };
      delete next[key];
      if (key === 'title' && !slugTouched) delete next.slug;
      return next;
    });
  };

  // ---- Kiểm phía giao diện (API vẫn kiểm lại) ----
  const validate = useCallback(
    (f: FormState): Errors => {
      const e: Errors = {};
      const title = f.title.trim();
      if (title.length < 3 || title.length > 200) e.title = t('editor.errTitle');
      if (f.slug.trim() && !SLUG_RE.test(f.slug.trim())) e.slug = t('editor.errSlug');
      if (f.summary.trim().length > SUMMARY_MAX) e.summary = t('editor.errSummary');
      if (!f.categoryId) e.categoryId = t('editor.errCategory');
      if (f.coverImageUrl.trim() && !HTTP_URL_RE.test(f.coverImageUrl.trim())) e.coverImageUrl = t('editor.errCover');
      if (f.tags.length > MAX_TAGS) e.tags = t('editor.errTags');
      return e;
    },
    [t],
  );

  const visibleErrors: Errors = showErrors ? { ...validate(form), ...errors } : errors;

  // ---- Rời trang khi chưa lưu ----
  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) =>
      dirty && !allowNav.current && currentLocation.pathname !== nextLocation.pathname,
  );
  useEffect(() => {
    if (!dirty) return;
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      e.preventDefault();
    };
    window.addEventListener('beforeunload', onBeforeUnload);
    return () => window.removeEventListener('beforeunload', onBeforeUnload);
  }, [dirty]);

  // ---- Lưu / xuất bản ----
  const formRef = useRef(form);
  formRef.current = form;

  /** Nhận bài API trả về: giữ chữ người dùng đang thấy, chỉ lấy slug do API sinh/chuẩn hoá. */
  const applyServerArticle = (saved: AdminArticle) => {
    const kept: FormState = { ...formRef.current, slug: saved.slug ?? formRef.current.slug };
    const withContent: AdminArticle = { ...saved, content: saved.content ?? kept.content };
    setArticle(withContent);
    setBaseline(kept);
    setForm(kept);
    setSlugTouched(true);
    client.setQueryData(['admin', 'articles', 'detail', saved.id], withContent);
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'list'] });
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'count'] });
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'recent'] });
    void client.invalidateQueries({ queryKey: ['admin', 'stats'] });
  };

  const handleError = (err: unknown) => {
    if (isApiError(err) && err.fields) {
      setErrors(topLevelFields(err.fields) as Errors);
      toast.error(t('errors.validation'));
      return;
    }
    if (hasCode(err, 'SLUG_TAKEN')) {
      setErrors((e) => ({ ...e, slug: t('editor.slugTaken') }));
      toast.error(t('editor.slugTaken'));
      return;
    }
    toast.error(errorMessage(err, t, i18n.language));
  };

  /** Lưu nội dung hiện tại; trả bài đã lưu, hoặc undefined nếu dữ liệu chưa hợp lệ. */
  const persist = async (): Promise<AdminArticle | undefined> => {
    setShowErrors(true);
    const clientErrors = validate(form);
    if (Object.keys(clientErrors).length > 0) {
      toast.error(t('errors.validation'));
      return undefined;
    }
    const input = toInput(form);
    const saved = article ? await api.admin.articles.update(article.id, input) : await api.admin.articles.create(input);
    applyServerArticle(saved);
    return saved;
  };

  /** Bài mới vừa được tạo ⇒ chuyển sang URL sửa (không hỏi "rời trang"). */
  const goToCreated = (saved: AdminArticle, wasNew: boolean) => {
    if (!wasNew) return;
    allowNav.current = true;
    navigate(`/articles/${saved.id}`, { replace: true });
  };

  const save = useMutation({
    mutationFn: async () => ({ wasNew: isNew, saved: await persist() }),
    onSuccess: ({ wasNew, saved }) => {
      if (!saved) return;
      toast.success(wasNew ? t('editor.created') : t('editor.saved'));
      goToCreated(saved, wasNew);
    },
    onError: handleError,
  });

  const publish = useMutation({
    mutationFn: async () => {
      const wasNew = isNew;
      const saved = dirty || wasNew ? await persist() : article;
      if (!saved) return { wasNew, published: undefined };
      return { wasNew, published: await api.admin.articles.publish(saved.id) };
    },
    onSuccess: ({ wasNew, published }) => {
      if (!published) return;
      applyServerArticle(published);
      toast.success(t('editor.published'));
      goToCreated(published, wasNew);
    },
    onError: handleError,
  });

  const unpublish = useMutation({
    mutationFn: () => api.admin.articles.unpublish(article?.id ?? ''),
    onSuccess: (a) => {
      applyServerArticle(a);
      setConfirm(null);
      toast.success(t('editor.unpublished'));
    },
    onError: (err) => {
      setConfirm(null);
      handleError(err);
    },
  });

  const archive = useMutation({
    mutationFn: () => api.admin.articles.archive(article?.id ?? ''),
    onSuccess: () => {
      if (article) {
        setArticle({ ...article, status: 'archived' });
        client.setQueryData(['admin', 'articles', 'detail', article.id], { ...article, status: 'archived' });
      }
      void client.invalidateQueries({ queryKey: ['admin', 'articles'] });
      setConfirm(null);
      toast.success(t('editor.archived'));
    },
    onError: (err) => {
      setConfirm(null);
      handleError(err);
    },
  });

  const busy = save.isPending || publish.isPending || unpublish.isPending || archive.isPending;

  // Ctrl/Cmd+S để lưu.
  const saveRef = useRef(() => {});
  saveRef.current = () => {
    if (!busy && (dirty || isNew)) save.mutate();
  };
  useEffect(() => {
    const onKey = (e: globalThis.KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        saveRef.current();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // ---- Tag ----
  const addTag = (raw: string) => {
    const tag = normalizeTag(raw);
    if (!tag) return;
    if (form.tags.includes(tag) || form.tags.length >= MAX_TAGS) {
      setTagText('');
      return;
    }
    update('tags', [...form.tags, tag]);
    setTagText('');
  };
  const onTagKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      addTag(tagText);
    } else if (e.key === 'Backspace' && tagText === '' && form.tags.length > 0) {
      update('tags', form.tags.slice(0, -1));
    }
  };

  // ---- Xem trước ----
  const deferredContent = useDeferredValue(form.content);
  const words = useMemo(() => form.content.split(/\s+/).filter(Boolean).length, [form.content]);
  const minutes = Math.max(1, Math.ceil(words / 200));

  const categoryOptions = useMemo(() => {
    const opts = (categories.data ?? []).map((c) => ({ id: c.id, label: pickLocalized(c.name, locale) }));
    if (article?.category && !opts.some((o) => o.id === article.category.id)) {
      opts.push({ id: article.category.id, label: pickLocalized(article.category.name, locale) });
    }
    return opts;
  }, [categories.data, article, locale]);

  const coverUrl = form.coverImageUrl.trim();
  useEffect(() => setCoverBroken(false), [coverUrl]);

  return (
    <>
      {/* Đầu trang */}
      <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <Link to="/articles" className="mb-2 inline-flex items-center gap-1 text-sm font-medium text-muted hover:text-ink">
            <ArrowLeft className="size-4" aria-hidden />
            {t('nav.articles')}
          </Link>
          <div className="flex flex-wrap items-center gap-3">
            <h1 className="text-2xl font-bold tracking-tight text-ink">{isNew ? t('editor.newTitle') : t('editor.editTitle')}</h1>
            {!isNew && <ArticleStatusBadge status={status} />}
          </div>
          <p className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-muted" aria-live="polite">
            <span className={dirty ? 'font-medium text-amber-700 dark:text-amber-400' : undefined}>
              {dirty ? t('editor.unsaved') : isNew ? '' : t('editor.allSaved')}
            </span>
            {article?.publishedAt && status === 'published' && (
              <span>{t('editor.publishedAt', { date: formatDate(article.publishedAt, locale) })}</span>
            )}
            {article && status === 'published' && (
              <a href={siteArticleUrl(article.slug)} target="_blank" rel="noreferrer" className="inline-flex items-center gap-1 font-medium text-primary-ink hover:underline">
                <ExternalLink className="size-3.5" aria-hidden />
                {t('common.openInSite')}
              </a>
            )}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {!isNew && status !== 'archived' && (
            <Button icon={Archive} variant="ghost" onClick={() => setConfirm('archive')} disabled={busy}>
              {t('editor.archive')}
            </Button>
          )}
          {status === 'published' && (
            <Button icon={Undo2} onClick={() => setConfirm('unpublish')} disabled={busy}>
              {t('editor.unpublish')}
            </Button>
          )}
          <Button icon={Save} onClick={() => save.mutate()} loading={save.isPending} disabled={busy || (!dirty && !isNew)}>
            {status === 'draft' ? t('editor.saveDraft') : t('editor.saveChanges')}
          </Button>
          {status !== 'published' && (
            <Button icon={Send} variant="primary" onClick={() => publish.mutate()} loading={publish.isPending} disabled={busy}>
              {t('editor.publish')}
            </Button>
          )}
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]">
        {/* Cột trái — thông tin bài */}
        <Card className="h-fit space-y-4 p-4 sm:p-5">
          <h2 className="text-base font-semibold text-ink">{t('editor.formSection')}</h2>
          <Field label={t('editor.fieldTitle')} error={visibleErrors.title} required>
            {(a) => (
              <input {...a} className={inputClass} value={form.title} maxLength={200} onChange={(e) => update('title', e.target.value)} />
            )}
          </Field>
          <Field
            label={t('editor.fieldSlug')}
            error={visibleErrors.slug}
            hint={t('editor.slugHint')}
            aside={
              slugTouched && form.title.trim() ? (
                <button
                  type="button"
                  className="inline-flex items-center gap-1 text-xs font-medium text-primary-ink hover:underline"
                  onClick={() => {
                    setSlugTouched(false);
                    update('slug', slugify(form.title));
                  }}
                >
                  <RefreshCw className="size-3" aria-hidden />
                  {t('editor.slugReset')}
                </button>
              ) : undefined
            }
          >
            {(a) => (
              <input
                {...a}
                className={cx(inputClass, 'font-mono')}
                value={form.slug}
                spellCheck={false}
                onChange={(e) => {
                  setSlugTouched(true);
                  update('slug', e.target.value.toLowerCase());
                }}
              />
            )}
          </Field>
          <Field
            label={t('editor.fieldSummary')}
            error={visibleErrors.summary}
            aside={
              <span className={cx('text-xs tabular-nums', form.summary.length > SUMMARY_MAX ? 'font-semibold text-red-700 dark:text-red-400' : 'text-muted')}>
                {form.summary.length}/{SUMMARY_MAX}
              </span>
            }
          >
            {(a) => <textarea {...a} rows={4} className={inputClass} value={form.summary} onChange={(e) => update('summary', e.target.value)} />}
          </Field>
          <Field label={t('editor.fieldCategory')} error={visibleErrors.categoryId} required>
            {(a) => (
              <select {...a} className={selectClass} value={form.categoryId} onChange={(e) => update('categoryId', e.target.value)}>
                <option value="">{t('editor.chooseCategory')}</option>
                {categoryOptions.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.label}
                  </option>
                ))}
              </select>
            )}
          </Field>
          <div className="grid grid-cols-2 gap-3">
            <Field label={t('editor.fieldLevel')} error={visibleErrors.level}>
              {(a) => (
                <select {...a} className={selectClass} value={form.level} onChange={(e) => update('level', e.target.value as Level)}>
                  {LEVELS.map((l) => (
                    <option key={l} value={l}>
                      {t(`level.${l}`)}
                    </option>
                  ))}
                </select>
              )}
            </Field>
            <Field label={t('editor.fieldLanguage')} error={visibleErrors.language}>
              {(a) => (
                <select {...a} className={selectClass} value={form.language} onChange={(e) => update('language', e.target.value as Locale)}>
                  <option value="vi">{t('language.vi')}</option>
                  <option value="en">{t('language.en')}</option>
                </select>
              )}
            </Field>
          </div>
          <Field
            label={t('editor.fieldTags')}
            error={visibleErrors.tags}
            hint={form.tags.length >= MAX_TAGS ? t('editor.tagsFull', { max: MAX_TAGS }) : t('editor.tagsHint', { max: MAX_TAGS })}
            aside={<span className="text-xs text-muted tabular-nums">{form.tags.length}/{MAX_TAGS}</span>}
          >
            {(a) => (
              <div className="flex flex-wrap items-center gap-1.5 rounded-ui border border-line bg-surface p-1.5 focus-within:border-primary focus-within:ring-2 focus-within:ring-primary/30">
                {form.tags.map((tag) => (
                  <span key={tag} className="inline-flex items-center gap-1 rounded-ui bg-primary-soft py-0.5 pr-1 pl-2 text-xs font-medium text-primary-ink">
                    #{tag}
                    <button
                      type="button"
                      aria-label={t('editor.removeTag', { tag })}
                      className="rounded p-0.5 hover:bg-primary/15"
                      onClick={() => update('tags', form.tags.filter((x) => x !== tag))}
                    >
                      <X className="size-3" aria-hidden />
                    </button>
                  </span>
                ))}
                <input
                  {...a}
                  className="min-w-[6rem] flex-1 bg-transparent px-1.5 py-1 text-sm text-ink placeholder:text-muted focus:outline-none disabled:opacity-60"
                  value={tagText}
                  placeholder={form.tags.length >= MAX_TAGS ? '' : t('editor.tagsPlaceholder')}
                  disabled={form.tags.length >= MAX_TAGS}
                  onChange={(e) => setTagText(e.target.value.replace(',', ''))}
                  onKeyDown={onTagKey}
                  onBlur={() => tagText.trim() && addTag(tagText)}
                />
              </div>
            )}
          </Field>
          <Field label={t('editor.fieldCover')} error={visibleErrors.coverImageUrl} hint={t('editor.coverHint')}>
            {(a) => (
              <input
                {...a}
                type="url"
                inputMode="url"
                className={inputClass}
                value={form.coverImageUrl}
                placeholder="https://"
                onChange={(e) => update('coverImageUrl', e.target.value)}
              />
            )}
          </Field>
          {coverUrl && HTTP_URL_RE.test(coverUrl) && !coverBroken && (
            <img
              src={coverUrl}
              alt={t('editor.coverPreview')}
              className="aspect-video w-full rounded-ui border border-line object-cover"
              onError={() => setCoverBroken(true)}
            />
          )}
          {article?.learningPathId && <p className="rounded-ui bg-surface-2 p-3 text-xs text-muted">{t('editor.learningPathNote')}</p>}
        </Card>

        {/* Cột phải — Markdown */}
        <Card className="flex min-w-0 flex-col">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-line px-4 py-3">
            <h2 className="text-base font-semibold text-ink">{t('editor.fieldContent')}</h2>
            <div role="radiogroup" aria-label={t('editor.editorMode')} className="inline-flex rounded-ui border border-line bg-surface-2 p-1">
              {(
                [
                  ['edit', PenLine, t('editor.modeEdit')],
                  ['split', Columns2, t('editor.modeSplit')],
                  ['preview', Eye, t('editor.modePreview')],
                ] as const
              ).map(([m, Icon, label]) => (
                <button
                  key={m}
                  type="button"
                  role="radio"
                  aria-checked={mode === m}
                  onClick={() => setMode(m)}
                  className={cx(
                    'inline-flex items-center gap-1.5 rounded-[6px] px-2.5 py-1 text-sm font-medium',
                    mode === m ? 'bg-primary text-white' : 'text-ink hover:bg-surface',
                  )}
                >
                  <Icon className="size-4" aria-hidden />
                  {label}
                </button>
              ))}
            </div>
          </div>
          <div className={cx('grid min-h-[32rem] flex-1', mode === 'split' && 'xl:grid-cols-2')}>
            {mode !== 'preview' && (
              <div className={cx('flex flex-col', mode === 'split' && 'border-b border-line xl:border-r xl:border-b-0')}>
                <label htmlFor="article-content" className="sr-only">
                  {t('editor.fieldContent')}
                </label>
                <textarea
                  id="article-content"
                  aria-invalid={Boolean(visibleErrors.content)}
                  className="min-h-[32rem] w-full flex-1 resize-y bg-transparent p-4 font-mono text-sm leading-6 text-ink placeholder:text-muted focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/40 focus-visible:ring-inset"
                  placeholder={t('editor.contentPlaceholder')}
                  spellCheck={false}
                  value={form.content}
                  onChange={(e) => update('content', e.target.value)}
                />
              </div>
            )}
            {mode !== 'edit' && (
              <div className="min-w-0 overflow-auto bg-page p-4 sm:p-6" aria-label={t('editor.modePreview')} role="region">
                {deferredContent.trim() ? (
                  <ArticleContent markdown={deferredContent} />
                ) : (
                  <p className="text-sm text-muted italic">{t('editor.previewEmpty')}</p>
                )}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-center justify-between gap-2 border-t border-line px-4 py-2 text-xs text-muted">
            <span>{t('editor.words', { count: words, minutes })}</span>
            {visibleErrors.content ? (
              <span className="font-medium text-red-700 dark:text-red-400">{visibleErrors.content}</span>
            ) : (
              <span>{t('editor.previewNote')}</span>
            )}
          </div>
        </Card>
      </div>

      <ConfirmDialog
        open={confirm === 'unpublish'}
        title={t('editor.confirmUnpublishTitle')}
        body={t('editor.confirmUnpublishBody')}
        confirmLabel={t('editor.unpublish')}
        tone="primary"
        loading={unpublish.isPending}
        onConfirm={() => unpublish.mutate()}
        onCancel={() => setConfirm(null)}
      />
      <ConfirmDialog
        open={confirm === 'archive'}
        title={t('editor.confirmArchiveTitle')}
        body={t('editor.confirmArchiveBody')}
        confirmLabel={t('editor.archive')}
        loading={archive.isPending}
        onConfirm={() => archive.mutate()}
        onCancel={() => setConfirm(null)}
      />
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
