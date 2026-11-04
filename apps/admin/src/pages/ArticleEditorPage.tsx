// Màn soạn bài (CMS): CodeMirror + xem trước bằng đúng bộ hiển thị của trang đọc, các khung Thông tin / Ảnh bìa / SEO /
// Xuất bản (lên lịch) / Mục lục, tự sao lưu bản nháp vào trình duyệt, Ctrl+S lưu lên máy chủ.
import { useCallback, useDeferredValue, useEffect, useId, useMemo, useRef, useState, type KeyboardEvent, type ReactNode } from 'react';
import { Link, useBlocker, useNavigate, useParams } from 'react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import {
  Archive,
  ArrowLeft,
  CalendarClock,
  ChevronDown,
  Columns2,
  ExternalLink,
  Eye,
  ImagePlus,
  PenLine,
  RefreshCw,
  Save,
  Send,
  Trash,
  Undo2,
  X,
} from 'lucide-react';
import type { EditorView } from '@uiw/react-codemirror';
import { api, LEVELS, pickLocalized, slugify, type AdminArticle, type ArticleInput, type Level, type Locale } from '@tkh/shared';
import { ArticleContent, extractHeadings } from '@tkh/ui';
import { Button, Card, ConfirmDialog, ErrorState, Field, Spinner, cx, inputClass, selectClass } from '../components/ui';
import { ArticleStatusBadge } from '../components/domain';
import { MarkdownCodeMirror, MarkdownToolbar } from '../components/editor/MarkdownEditor';
import { goToLine, insertText, outlineLines } from '../components/editor/commands';
import { MediaPickerModal } from '../components/media/MediaPickerModal';
import { displayStatus, formatDateTime, fromLocalInput, toLocalInput, wordStats } from '../lib/articles';
import { draftKey, readDraft, removeDraft, writeDraft } from '../lib/drafts';
import { errorMessage, hasCode, hasStatus, isApiError } from '../lib/errors';
import { imageMarkdown } from '../lib/media';
import { IMAGE_URL_RE, SLUG_RE, siteArticleUrl } from '../lib/site';
import { useCategoryOptions } from '../lib/queries';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const MAX_TAGS = 10;
const SUMMARY_MAX = 400;
const SEO_TITLE_MAX = 70;
const SEO_DESC_MAX = 170;
const AUTOSAVE_MS = 3000;
type Mode = 'edit' | 'preview' | 'split';
const MODE_KEY = 'tkh-admin-editor-mode';
const PANELS_KEY = 'tkh-admin-editor-panels';

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
  seoTitle: string;
  seoDescription: string;
  seoOgImageUrl: string;
  seoNoindex: boolean;
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
  seoTitle: '',
  seoDescription: '',
  seoOgImageUrl: '',
  seoNoindex: false,
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
    seoTitle: a.seo?.title ?? '',
    seoDescription: a.seo?.description ?? '',
    seoOgImageUrl: a.seo?.ogImageUrl ?? '',
    seoNoindex: a.seo?.noindex === true,
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
    // Luôn gửi `seo`: trường rỗng ⇒ API gỡ ghi đè tương ứng.
    seo: {
      title: f.seoTitle.trim() || undefined,
      description: f.seoDescription.trim() || undefined,
      ogImageUrl: f.seoOgImageUrl.trim() || undefined,
      noindex: f.seoNoindex || undefined,
    },
  };
}

/** Lỗi trường từ API (`seo.title`, `tags.0`…) ⇒ khoá của form. */
function mapApiFields(fields: Record<string, string>): Errors {
  const out: Errors = {};
  const SEO: Record<string, FieldKey> = { 'seo.title': 'seoTitle', 'seo.description': 'seoDescription', 'seo.ogImageUrl': 'seoOgImageUrl', 'seo.noindex': 'seoNoindex' };
  for (const [k, v] of Object.entries(fields)) {
    const key = (SEO[k] ?? k.split('.')[0]) as FieldKey;
    if (key in EMPTY && !(key in out)) out[key] = v;
  }
  return out;
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

type PanelId = 'publish' | 'info' | 'cover' | 'seo' | 'outline';
const DEFAULT_PANELS: Record<PanelId, boolean> = { publish: true, info: true, cover: true, seo: false, outline: false };

function readPanels(): Record<PanelId, boolean> {
  try {
    const raw = localStorage.getItem(PANELS_KEY);
    if (raw) return { ...DEFAULT_PANELS, ...(JSON.parse(raw) as Partial<Record<PanelId, boolean>>) };
  } catch {
    /* bỏ qua */
  }
  return DEFAULT_PANELS;
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

// ---------------- Khung thu gọn được ở cột phải ----------------

function Panel({
  id,
  title,
  open,
  onToggle,
  children,
  badge,
}: {
  id: PanelId;
  title: string;
  open: boolean;
  onToggle: (id: PanelId) => void;
  children: ReactNode;
  badge?: ReactNode;
}) {
  const { t } = useTranslation();
  const bodyId = useId();
  return (
    <Card>
      <h2>
        <button
          type="button"
          aria-expanded={open}
          aria-controls={bodyId}
          title={t('cms.togglePanel', { name: title })}
          onClick={() => onToggle(id)}
          className="flex w-full items-center justify-between gap-2 rounded-card px-4 py-3 text-left text-sm font-semibold text-ink hover:bg-surface-2/60"
        >
          <span className="flex items-center gap-2">
            {title}
            {badge}
          </span>
          <ChevronDown className={cx('size-4 text-muted transition-transform', open && 'rotate-180')} aria-hidden />
        </button>
      </h2>
      <div id={bodyId} hidden={!open} className="space-y-4 border-t border-line px-4 py-4">
        {children}
      </div>
    </Card>
  );
}

function Counter({ value, max }: { value: number; max: number }) {
  const { t } = useTranslation();
  const over = value > max;
  return (
    <span className={cx('text-xs tabular-nums', over ? 'font-semibold text-red-700 dark:text-red-400' : value > max * 0.9 ? 'text-amber-700 dark:text-amber-400' : 'text-muted')}>
      {value}/{max}
      {over && <span className="sr-only"> — {t('cms.seoOver', { count: value - max })}</span>}
    </span>
  );
}

/** Ô xem ảnh nhỏ (ảnh bìa / ảnh chia sẻ) + nút chọn/gỡ. */
function ImageSlot({
  url,
  alt,
  onPick,
  onRemove,
  pickLabel,
  replaceLabel,
  removeLabel,
  emptyText,
}: {
  url: string;
  alt: string;
  onPick: () => void;
  onRemove: () => void;
  pickLabel: string;
  replaceLabel: string;
  removeLabel: string;
  emptyText: string;
}) {
  const { t } = useTranslation();
  const [broken, setBroken] = useState(false);
  useEffect(() => setBroken(false), [url]);
  const valid = url && IMAGE_URL_RE.test(url);
  return (
    <div className="space-y-2">
      {valid ? (
        broken ? (
          <p className="rounded-ui bg-red-50 p-3 text-xs font-medium text-red-800 dark:bg-red-950/50 dark:text-red-200">{t('cms.coverBroken')}</p>
        ) : (
          <img src={url} alt={alt} className="aspect-video w-full rounded-ui border border-line bg-surface-2 object-cover" onError={() => setBroken(true)} />
        )
      ) : (
        <p className="rounded-ui border border-dashed border-line p-4 text-center text-xs text-muted">{emptyText}</p>
      )}
      <div className="flex flex-wrap gap-2">
        <Button size="sm" icon={ImagePlus} onClick={onPick}>
          {url ? replaceLabel : pickLabel}
        </Button>
        {url && (
          <Button size="sm" variant="ghost" icon={Trash} onClick={onRemove}>
            {removeLabel}
          </Button>
        )}
      </div>
    </div>
  );
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
  const [panels, setPanels] = useState<Record<PanelId, boolean>>(readPanels);
  const [confirm, setConfirm] = useState<'unpublish' | 'archive' | null>(null);
  const [tagText, setTagText] = useState('');
  const [picker, setPicker] = useState<'cover' | 'og' | 'inline' | null>(null);
  const [scheduleOpen, setScheduleOpen] = useState(false);
  const [scheduleValue, setScheduleValue] = useState('');
  const [scheduleError, setScheduleError] = useState<string>();
  const [view, setView] = useState<EditorView | null>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const allowNav = useRef(false);

  const isNew = !article;
  const status = article ? displayStatus(article) : 'draft';
  const dirty = useMemo(() => JSON.stringify(form) !== JSON.stringify(baseline), [form, baseline]);
  const storageKey = draftKey(article?.id);

  const setMode = (m: Mode) => {
    setModeState(m);
    try {
      localStorage.setItem(MODE_KEY, m);
    } catch {
      /* bỏ qua */
    }
  };
  const togglePanel = useCallback((id: PanelId) => {
    setPanels((p) => {
      const next = { ...p, [id]: !p[id] };
      try {
        localStorage.setItem(PANELS_KEY, JSON.stringify(next));
      } catch {
        /* bỏ qua */
      }
      return next;
    });
  }, []);
  const openPanel = (id: PanelId) => {
    if (!panels[id]) togglePanel(id);
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
      if (f.coverImageUrl.trim() && !IMAGE_URL_RE.test(f.coverImageUrl.trim())) e.coverImageUrl = t('cms.errImageUrl');
      if (f.tags.length > MAX_TAGS) e.tags = t('editor.errTags');
      if (f.seoTitle.trim().length > SEO_TITLE_MAX) e.seoTitle = t('cms.errSeoTitle');
      if (f.seoDescription.trim().length > SEO_DESC_MAX) e.seoDescription = t('cms.errSeoDesc');
      if (f.seoOgImageUrl.trim() && !IMAGE_URL_RE.test(f.seoOgImageUrl.trim())) e.seoOgImageUrl = t('cms.errImageUrl');
      return e;
    },
    [t],
  );

  const visibleErrors: Errors = showErrors ? { ...validate(form), ...errors } : errors;

  // ---- Sao lưu cục bộ (localStorage) ----
  const formRef = useRef(form);
  formRef.current = form;
  const [localSavedAt, setLocalSavedAt] = useState<number>();
  const [restoreOffer, setRestoreOffer] = useState(() => {
    const backup = readDraft<FormState>(draftKey(initialArticle?.id));
    if (!backup) return undefined;
    const serverTime = initialArticle ? Date.parse(initialArticle.updatedAt) : 0;
    const base = initialArticle ? fromArticle(initialArticle) : EMPTY;
    const merged = { ...EMPTY, ...backup.form };
    if (backup.savedAt <= serverTime || JSON.stringify(merged) === JSON.stringify(base)) return undefined;
    return { form: merged, savedAt: backup.savedAt };
  });
  const autosaveTimer = useRef<number | undefined>(undefined);
  useEffect(() => {
    if (!dirty || autosaveTimer.current !== undefined) return;
    autosaveTimer.current = window.setTimeout(() => {
      autosaveTimer.current = undefined;
      const at = writeDraft(storageKey, formRef.current);
      if (at) setLocalSavedAt(at);
    }, AUTOSAVE_MS);
  }, [form, dirty, storageKey]);
  useEffect(() => () => window.clearTimeout(autosaveTimer.current), []);

  const restoreDraft = () => {
    if (!restoreOffer) return;
    setForm(restoreOffer.form);
    setSlugTouched(true);
    setRestoreOffer(undefined);
    toast.success(t('cms.restored'));
  };
  const discardDraft = () => {
    removeDraft(storageKey);
    setRestoreOffer(undefined);
  };

  // ---- Rời trang khi chưa lưu ----
  const blocker = useBlocker(
    ({ currentLocation, nextLocation }) => dirty && !allowNav.current && currentLocation.pathname !== nextLocation.pathname,
  );
  useEffect(() => {
    if (!dirty) return;
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      // Ghi bản sao ngay trước khi đóng tab (phòng khi người dùng vẫn rời đi).
      writeDraft(storageKey, formRef.current);
      e.preventDefault();
    };
    window.addEventListener('beforeunload', onBeforeUnload);
    return () => window.removeEventListener('beforeunload', onBeforeUnload);
  }, [dirty, storageKey]);

  // ---- Lưu / xuất bản ----

  /**
   * Nhận bài API trả về. `sent` = form đã gửi: phần người dùng gõ thêm trong lúc chờ API vẫn giữ (và vẫn tính là chưa lưu);
   * chỉ lấy slug do API sinh/chuẩn hoá.
   */
  const applyServerArticle = (saved: AdminArticle, sent: FormState = formRef.current) => {
    const kept: FormState = { ...sent, slug: saved.slug ?? sent.slug };
    const withContent: AdminArticle = { ...saved, content: saved.content ?? kept.content };
    setArticle(withContent);
    setBaseline(kept);
    setForm((cur) => (JSON.stringify(cur) === JSON.stringify(sent) ? kept : { ...cur, slug: kept.slug }));
    setSlugTouched(true);
    client.setQueryData(['admin', 'articles', 'detail', saved.id], withContent);
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'list'] });
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'count'] });
    void client.invalidateQueries({ queryKey: ['admin', 'articles', 'recent'] });
    void client.invalidateQueries({ queryKey: ['admin', 'stats'] });
  };

  const openPanelsFor = (e: Errors) => {
    if (e.seoTitle || e.seoDescription || e.seoOgImageUrl) openPanel('seo');
    if (e.coverImageUrl) openPanel('cover');
    if (e.title || e.slug || e.summary || e.categoryId || e.tags) openPanel('info');
  };

  const handleError = (err: unknown) => {
    if (hasCode(err, 'SLUG_TAKEN')) {
      setErrors((e) => ({ ...e, slug: t('editor.slugTaken') }));
      openPanel('info');
      toast.error(t('editor.slugTaken'));
      return;
    }
    if (isApiError(err) && err.fields && Object.keys(err.fields).length) {
      const mapped = mapApiFields(err.fields);
      setErrors(mapped);
      openPanelsFor(mapped);
      toast.error(t('errors.validation'));
      return;
    }
    toast.error(errorMessage(err, t, i18n.language));
  };

  /** Lưu nội dung hiện tại; trả bài đã lưu, hoặc undefined nếu dữ liệu chưa hợp lệ. */
  const persist = async (): Promise<AdminArticle | undefined> => {
    setShowErrors(true);
    const snapshot = formRef.current;
    const clientErrors = validate(snapshot);
    if (Object.keys(clientErrors).length > 0) {
      openPanelsFor(clientErrors);
      toast.error(t('errors.validation'));
      return undefined;
    }
    const input = toInput(snapshot);
    const saved = article ? await api.admin.articles.update(article.id, input) : await api.admin.articles.create(input);
    removeDraft(storageKey);
    setLocalSavedAt(undefined);
    applyServerArticle(saved, snapshot);
    return saved;
  };

  /** Bài mới vừa được tạo ⇒ chuyển sang URL sửa (không hỏi "rời trang"). */
  const goToCreated = (saved: AdminArticle, wasNew: boolean) => {
    if (!wasNew) return;
    removeDraft(draftKey(undefined));
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
    mutationFn: async (publishAt: string | undefined) => {
      const wasNew = isNew;
      const saved = dirty || wasNew ? await persist() : article;
      if (!saved) return { wasNew, published: undefined, publishAt };
      return { wasNew, published: await api.admin.articles.publish(saved.id, publishAt ? { publishAt } : {}), publishAt };
    },
    onSuccess: ({ wasNew, published, publishAt }) => {
      if (!published) return;
      applyServerArticle(published);
      setScheduleOpen(false);
      setScheduleError(undefined);
      toast.success(publishAt ? t('cms.scheduledToast', { date: formatDateTime(publishAt, locale) }) : t('editor.published'));
      goToCreated(published, wasNew);
    },
    onError: (err) => {
      if (isApiError(err) && err.fields?.publishAt) {
        setScheduleError(err.fields.publishAt);
        return;
      }
      handleError(err);
    },
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

  const submitSchedule = () => {
    const iso = fromLocalInput(scheduleValue);
    if (!iso) return setScheduleError(t('cms.errScheduleEmpty'));
    if (Date.parse(iso) <= Date.now()) return setScheduleError(t('cms.errSchedulePast'));
    setScheduleError(undefined);
    publish.mutate(iso);
  };

  const openSchedule = () => {
    const current = article?.publishedAt && Date.parse(article.publishedAt) > Date.now() ? new Date(article.publishedAt) : undefined;
    const base = current ?? new Date(Date.now() + 3600_000);
    if (!current) base.setMinutes(0, 0, 0);
    setScheduleValue(toLocalInput(base));
    setScheduleError(undefined);
    setScheduleOpen(true);
  };

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

  // ---- Xem trước, số từ, mục lục ----
  const deferredContent = useDeferredValue(form.content);
  const stats = useMemo(() => wordStats(deferredContent), [deferredContent]);
  const outline = useMemo(() => {
    const lines = outlineLines(deferredContent);
    const ids = extractHeadings(deferredContent);
    return lines.map((l, i) => ({ ...l, id: ids[i]?.id }));
  }, [deferredContent]);

  const jumpTo = (item: { line: number; id?: string }) => {
    if (mode !== 'edit' && item.id && previewRef.current) {
      const box = previewRef.current;
      const target = box.querySelector<HTMLElement>(`#${CSS.escape(item.id)}`);
      if (target) box.scrollTo({ top: box.scrollTop + target.getBoundingClientRect().top - box.getBoundingClientRect().top - 12, behavior: 'smooth' });
    }
    if (mode !== 'preview' && view) goToLine(view, item.line);
  };

  // Cuộn đồng bộ theo tỉ lệ ở chế độ chia đôi (bên nào người dùng đang cuộn thì bên kia đi theo).
  useEffect(() => {
    if (mode !== 'split' || !view || !previewRef.current) return;
    const a = view.scrollDOM;
    const b = previewRef.current;
    let lock: 'a' | 'b' | null = null;
    let release = 0;
    const sync = (from: HTMLElement, to: HTMLElement, who: 'a' | 'b') => () => {
      if (lock && lock !== who) return;
      lock = who;
      const max = from.scrollHeight - from.clientHeight;
      const ratio = max > 0 ? from.scrollTop / max : 0;
      to.scrollTop = ratio * (to.scrollHeight - to.clientHeight);
      window.clearTimeout(release);
      release = window.setTimeout(() => (lock = null), 120);
    };
    const onA = sync(a, b, 'a');
    const onB = sync(b, a, 'b');
    a.addEventListener('scroll', onA, { passive: true });
    b.addEventListener('scroll', onB, { passive: true });
    return () => {
      a.removeEventListener('scroll', onA);
      b.removeEventListener('scroll', onB);
      window.clearTimeout(release);
    };
  }, [mode, view]);

  const categoryOptions = useMemo(() => {
    const opts = (categories.data ?? []).map((c) => ({ id: c.id, label: pickLocalized(c.name, locale) }));
    if (article?.category && !opts.some((o) => o.id === article.category.id)) {
      opts.push({ id: article.category.id, label: pickLocalized(article.category.name, locale) });
    }
    return opts;
  }, [categories.data, article, locale]);

  const onPicked = (item: { url: string }, alt?: string) => {
    if (picker === 'cover') update('coverImageUrl', item.url);
    else if (picker === 'og') update('seoOgImageUrl', item.url);
    else if (picker === 'inline' && view) insertText(view, imageMarkdown(item.url, alt ?? ''));
    setPicker(null);
  };

  const tz = useMemo(() => Intl.DateTimeFormat().resolvedOptions().timeZone, []);
  const live = status === 'published';
  const scheduledAt = status === 'scheduled' ? article?.publishedAt : undefined;
  const snippetTitle = form.seoTitle.trim() || form.title.trim() || t('editor.newTitle');
  const snippetDesc = form.seoDescription.trim() || form.summary.trim();

  return (
    <>
      {/* Đầu trang */}
      <div className="mb-4 flex flex-wrap items-start justify-between gap-4">
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
            {dirty && localSavedAt && <span className="text-xs">{t('cms.localSaved', { time: formatDateTime(new Date(localSavedAt).toISOString(), locale) })}</span>}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {article && live && (
            <a href={siteArticleUrl(article.slug)} target="_blank" rel="noreferrer" className="inline-flex h-10 items-center gap-1.5 rounded-ui px-3 text-sm font-medium text-primary-ink hover:bg-surface-2">
              <ExternalLink className="size-4" aria-hidden />
              {t('cms.viewOnSite')}
            </a>
          )}
          <Button icon={Save} onClick={() => save.mutate()} loading={save.isPending} disabled={busy || (!dirty && !isNew)} title="Ctrl+S">
            {status === 'draft' ? t('editor.saveDraft') : t('editor.saveChanges')}
          </Button>
          {!live && status !== 'scheduled' && (
            <Button icon={Send} variant="primary" onClick={() => publish.mutate(undefined)} loading={publish.isPending && !scheduleOpen} disabled={busy}>
              {t('editor.publish')}
            </Button>
          )}
        </div>
      </div>

      {restoreOffer && (
        <div role="status" className="mb-4 flex flex-wrap items-center gap-3 rounded-card border border-amber-300 bg-amber-50 p-3 text-sm dark:border-amber-700 dark:bg-amber-950/40">
          <div className="min-w-0 flex-1">
            <p className="font-semibold text-ink">{t('cms.restoreTitle')}</p>
            <p className="text-muted">{t('cms.restoreBody', { time: formatDateTime(new Date(restoreOffer.savedAt).toISOString(), locale) })}</p>
          </div>
          <Button size="sm" variant="primary" icon={RefreshCw} onClick={restoreDraft}>
            {t('cms.restore')}
          </Button>
          <Button size="sm" variant="ghost" onClick={discardDraft}>
            {t('cms.discard')}
          </Button>
        </div>
      )}

      <div className="grid gap-4 xl:grid-cols-[minmax(0,1fr)_22rem]">
        {/* Soạn + xem trước */}
        <Card className="flex min-w-0 flex-col xl:h-[calc(100dvh-10.5rem)]">
          <div className="sticky top-16 z-10 flex flex-wrap items-center justify-between gap-2 rounded-t-card border-b border-line bg-surface px-2 py-1.5 xl:static">
            {mode !== 'preview' ? (
              <MarkdownToolbar view={view} onImage={() => setPicker('inline')} />
            ) : (
              <span className="px-2 text-sm font-semibold text-ink">{t('editor.modePreview')}</span>
            )}
            <div role="radiogroup" aria-label={t('editor.editorMode')} className="inline-flex rounded-ui border border-line bg-surface-2 p-0.5">
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
                  aria-label={label}
                  onClick={() => setMode(m)}
                  className={cx('inline-flex items-center gap-1.5 rounded-[6px] px-2 py-1 text-xs font-medium', mode === m ? 'bg-primary text-white' : 'text-ink hover:bg-surface')}
                >
                  <Icon className="size-3.5" aria-hidden />
                  <span className="hidden sm:inline" aria-hidden>
                    {label}
                  </span>
                </button>
              ))}
            </div>
          </div>
          <div className={cx('grid min-h-0 xl:flex-1', mode === 'split' && 'md:grid-cols-2')}>
            {mode !== 'preview' && (
              <div className={cx('h-[70dvh] min-h-[24rem] min-w-0 xl:h-full', mode === 'split' && 'border-b border-line md:border-r md:border-b-0')}>
                <MarkdownCodeMirror
                  value={form.content}
                  onChange={(v) => update('content', v)}
                  onView={setView}
                  label={t('cms.editorLabel')}
                  placeholder={t('editor.contentPlaceholder')}
                  invalid={Boolean(visibleErrors.content)}
                />
              </div>
            )}
            {mode !== 'edit' && (
              <div
                ref={previewRef}
                className="h-[70dvh] min-h-[24rem] min-w-0 overflow-auto bg-page p-4 sm:p-6 xl:h-full"
                aria-label={t('cms.previewRegion')}
                role="region"
                tabIndex={0}
              >
                {deferredContent.trim() ? <ArticleContent markdown={deferredContent} /> : <p className="text-sm text-muted italic">{t('editor.previewEmpty')}</p>}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-b-card border-t border-line px-4 py-2 text-xs text-muted">
            <span className="tabular-nums">{t('cms.wordsShort', { words: stats.words, minutes: stats.minutes })}</span>
            {visibleErrors.content ? (
              <span className="font-medium text-red-700 dark:text-red-400">{visibleErrors.content}</span>
            ) : (
              <span>{mode === 'preview' ? t('editor.previewNote') : t('cms.shortcutsHint')}</span>
            )}
          </div>
        </Card>

        {/* Cột phải */}
        <aside className="space-y-3 xl:max-h-[calc(100dvh-10.5rem)] xl:overflow-y-auto xl:pr-1">
          <Panel id="publish" title={t('cms.panelPublish')} open={panels.publish} onToggle={togglePanel} badge={!isNew ? <ArticleStatusBadge status={status} /> : undefined}>
            <div className="space-y-1 text-sm">
              {scheduledAt && (
                <p className="flex items-center gap-1.5 font-medium text-primary-ink">
                  <CalendarClock className="size-4 shrink-0" aria-hidden />
                  {t('cms.scheduledFor', { date: formatDateTime(scheduledAt, locale) })}
                </p>
              )}
              {live && article?.publishedAt && <p className="text-muted">{t('cms.publishedSince', { date: formatDateTime(article.publishedAt, locale) })}</p>}
              <p className="text-muted">{article ? t('cms.lastSaved', { time: formatDateTime(article.updatedAt, locale) }) : t('cms.neverSaved')}</p>
              {article?.editedInCms && <p className="text-xs text-muted">{t('cms.editedInCms')}</p>}
            </div>

            <div className="flex flex-wrap gap-2">
              {!live && (
                <Button size="sm" variant="primary" icon={Send} onClick={() => publish.mutate(undefined)} loading={publish.isPending && !scheduleOpen} disabled={busy}>
                  {t('cms.publishNow')}
                </Button>
              )}
              {!live && !scheduleOpen && (
                <Button size="sm" icon={CalendarClock} onClick={openSchedule} disabled={busy}>
                  {status === 'scheduled' ? t('cms.reschedule') : t('cms.scheduleToggle')}
                </Button>
              )}
              {live && (
                <Button size="sm" icon={Undo2} onClick={() => setConfirm('unpublish')} disabled={busy}>
                  {t('editor.unpublish')}
                </Button>
              )}
              {status === 'scheduled' && (
                <Button size="sm" variant="ghost" icon={X} onClick={() => unpublish.mutate()} loading={unpublish.isPending} disabled={busy}>
                  {t('cms.cancelSchedule')}
                </Button>
              )}
              {article && live && (
                <a href={siteArticleUrl(article.slug)} target="_blank" rel="noreferrer" className="inline-flex h-8 items-center gap-1 px-1 text-sm font-medium text-primary-ink hover:underline">
                  <ExternalLink className="size-3.5" aria-hidden />
                  {t('cms.viewOnSite')}
                </a>
              )}
            </div>

            {scheduleOpen && !live && (
              <div className="space-y-2 rounded-ui border border-line bg-surface-2/60 p-3">
                <Field label={t('cms.scheduleAt')} error={scheduleError} hint={t('cms.scheduleHint', { tz })}>
                  {(a) => (
                    <input
                      {...a}
                      type="datetime-local"
                      className={inputClass}
                      value={scheduleValue}
                      min={toLocalInput(new Date())}
                      onChange={(e) => {
                        setScheduleValue(e.target.value);
                        setScheduleError(undefined);
                      }}
                    />
                  )}
                </Field>
                <div className="flex gap-2">
                  <Button size="sm" variant="primary" icon={CalendarClock} onClick={submitSchedule} loading={publish.isPending && scheduleOpen} disabled={busy}>
                    {t('cms.scheduleBtn')}
                  </Button>
                  <Button size="sm" variant="ghost" onClick={() => setScheduleOpen(false)}>
                    {t('common.cancel')}
                  </Button>
                </div>
              </div>
            )}

            {!isNew && status !== 'archived' && (
              <div className="border-t border-line pt-3">
                <Button size="sm" variant="ghost" icon={Archive} onClick={() => setConfirm('archive')} disabled={busy}>
                  {t('editor.archive')}
                </Button>
              </div>
            )}
          </Panel>

          <Panel id="info" title={t('cms.panelInfo')} open={panels.info} onToggle={togglePanel}>
            <Field label={t('editor.fieldTitle')} error={visibleErrors.title} required>
              {(a) => <input {...a} className={inputClass} value={form.title} maxLength={200} onChange={(e) => update('title', e.target.value)} />}
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
            <Field label={t('editor.fieldSummary')} error={visibleErrors.summary} aside={<Counter value={form.summary.trim().length} max={SUMMARY_MAX} />}>
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
            {article?.learningPathId && <p className="rounded-ui bg-surface-2 p-3 text-xs text-muted">{t('editor.learningPathNote')}</p>}
          </Panel>

          <Panel id="cover" title={t('cms.panelCover')} open={panels.cover} onToggle={togglePanel}>
            <ImageSlot
              url={form.coverImageUrl.trim()}
              alt={t('editor.coverPreview')}
              onPick={() => setPicker('cover')}
              onRemove={() => update('coverImageUrl', '')}
              pickLabel={t('cms.coverPick')}
              replaceLabel={t('cms.coverReplace')}
              removeLabel={t('cms.coverRemove')}
              emptyText={t('cms.coverEmpty')}
            />
            <Field label={t('cms.coverUrl')} error={visibleErrors.coverImageUrl}>
              {(a) => (
                <input
                  {...a}
                  inputMode="url"
                  className={inputClass}
                  value={form.coverImageUrl}
                  placeholder="https:// · /uploads/…"
                  onChange={(e) => update('coverImageUrl', e.target.value)}
                />
              )}
            </Field>
          </Panel>

          <Panel id="seo" title={t('cms.panelSeo')} open={panels.seo} onToggle={togglePanel}>
            <Field label={t('cms.seoTitle')} error={visibleErrors.seoTitle} hint={t('cms.seoTitleHint')} aside={<Counter value={form.seoTitle.trim().length} max={SEO_TITLE_MAX} />}>
              {(a) => <input {...a} className={inputClass} value={form.seoTitle} placeholder={form.title} onChange={(e) => update('seoTitle', e.target.value)} />}
            </Field>
            <Field
              label={t('cms.seoDesc')}
              error={visibleErrors.seoDescription}
              hint={t('cms.seoDescHint')}
              aside={<Counter value={form.seoDescription.trim().length} max={SEO_DESC_MAX} />}
            >
              {(a) => (
                <textarea {...a} rows={3} className={inputClass} value={form.seoDescription} placeholder={form.summary} onChange={(e) => update('seoDescription', e.target.value)} />
              )}
            </Field>
            <div className="space-y-1.5">
              <p className="text-sm font-medium text-ink">{t('cms.seoOg')}</p>
              <ImageSlot
                url={form.seoOgImageUrl.trim()}
                alt={t('cms.seoOg')}
                onPick={() => setPicker('og')}
                onRemove={() => update('seoOgImageUrl', '')}
                pickLabel={t('cms.seoOgPick')}
                replaceLabel={t('cms.coverReplace')}
                removeLabel={t('cms.seoOgRemove')}
                emptyText={t('cms.seoOgHint')}
              />
              {visibleErrors.seoOgImageUrl && <p className="text-xs font-medium text-red-700 dark:text-red-400">{visibleErrors.seoOgImageUrl}</p>}
            </div>
            <label className="flex items-start gap-2.5 text-sm">
              <input type="checkbox" className="mt-0.5 size-4 accent-[var(--color-primary)]" checked={form.seoNoindex} onChange={(e) => update('seoNoindex', e.target.checked)} />
              <span>
                <span className="font-medium text-ink">{t('cms.seoNoindex')}</span>
                <span className="mt-0.5 block text-xs text-muted">{t('cms.seoNoindexHint')}</span>
              </span>
            </label>
            <div>
              <p className="mb-1.5 text-xs font-semibold tracking-wide text-muted uppercase">{t('cms.seoPreview')}</p>
              <div className="rounded-ui border border-line bg-white p-3 font-[Arial,sans-serif] dark:bg-slate-900">
                <p className="truncate text-xs text-slate-700 dark:text-slate-300">antblog.xyz › articles › {form.slug.trim() || slugify(form.title) || '…'}</p>
                <p className="mt-0.5 line-clamp-1 text-lg leading-snug text-[#1a0dab] dark:text-[#8ab4f8]">{snippetTitle}</p>
                <p className="mt-0.5 line-clamp-2 text-[13px] leading-snug text-slate-600 dark:text-slate-400">
                  {snippetDesc.length > 160 ? `${snippetDesc.slice(0, 157)}…` : snippetDesc || '—'}
                </p>
                {form.seoNoindex && <p className="mt-1 text-[11px] font-semibold text-amber-700 dark:text-amber-400">noindex</p>}
              </div>
            </div>
          </Panel>

          <Panel id="outline" title={t('cms.panelOutline')} open={panels.outline} onToggle={togglePanel}>
            {outline.length === 0 ? (
              <p className="text-sm text-muted">{t('cms.outlineEmpty')}</p>
            ) : (
              <ol className="space-y-0.5 text-sm">
                {outline.map((h, i) => (
                  <li key={`${h.line}-${i}`}>
                    <button
                      type="button"
                      onClick={() => jumpTo(h)}
                      title={t('cms.outlineJump', { text: h.text })}
                      className={cx('block w-full truncate rounded-[6px] px-2 py-1 text-left hover:bg-surface-2', h.depth === 3 ? 'pl-6 text-muted' : 'text-ink')}
                    >
                      {h.text}
                    </button>
                  </li>
                ))}
              </ol>
            )}
          </Panel>
        </aside>
      </div>

      <MediaPickerModal
        open={picker !== null}
        onClose={() => setPicker(null)}
        onPick={onPicked}
        withAlt={picker === 'inline'}
        confirmLabel={picker === 'inline' ? t('media.insert') : t('media.pick')}
      />
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
