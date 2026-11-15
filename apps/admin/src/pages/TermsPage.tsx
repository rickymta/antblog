import { useMemo, useState, type FormEvent } from 'react';
import { Link } from 'react-router';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Pencil, Plus, ScanSearch, Search, Trash2 } from 'lucide-react';
import { api, formatNumber, slugify, type AdminTerm, type TermInput, type TermStatus, type TermUsage } from '@tkh/shared';
import {
  Badge,
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  Field,
  FilterBar,
  IconButton,
  Modal,
  PageHeader,
  TableSkeleton,
  TableWrap,
  Td,
  Th,
  cx,
  inputClass,
  selectClass,
} from '../components/ui';
import { errorMessage, isApiError } from '../lib/errors';
import { SLUG_RE } from '../lib/site';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const LIST_KEY = ['admin', 'terms'] as const;
/** Khớp SHORT_DEFINITION_MAX của API (api/src/routes/glossary.ts). */
const SHORT_DEFINITION_MAX = 300;

type Language = 'vi' | 'en';

/** Trang quản trị từ điển keyword (giai đoạn 4 — docs/content-dictionary/02-glossary-feature.md). Chỉ admin. */
export function TermsPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const client = useQueryClient();
  const toast = useToast();
  const list = useQuery({ queryKey: LIST_KEY, queryFn: () => api.admin.terms.list() });
  const [q, setQ] = useState('');
  const [language, setLanguage] = useState<'' | Language>('');
  const [status, setStatus] = useState<'' | TermStatus>('');
  const [editing, setEditing] = useState<AdminTerm | 'new' | null>(null);
  const [deleting, setDeleting] = useState<AdminTerm | null>(null);
  const [deleteError, setDeleteError] = useState<string | undefined>();

  const rows = useMemo(() => {
    const needle = q.trim().toLocaleLowerCase('vi');
    return (list.data ?? []).filter(
      (term) =>
        (!language || term.language === language) &&
        (!status || term.status === status) &&
        (!needle || [term.term, term.english ?? '', term.slug, ...term.aliases].some((s) => s.toLocaleLowerCase('vi').includes(needle))),
    );
  }, [list.data, q, language, status]);

  const remove = useMutation({
    mutationFn: (term: AdminTerm) => api.admin.terms.remove(term.id),
    onSuccess: () => {
      setDeleting(null);
      toast.success(t('terms.deleted'));
      void client.invalidateQueries({ queryKey: LIST_KEY });
      void client.invalidateQueries({ queryKey: ['glossary'] });
    },
    onError: (err) => setDeleteError(errorMessage(err, t, i18n.language)),
  });

  const filtered = Boolean(q.trim() || language || status);
  return (
    <>
      <PageHeader
        title={t('terms.title')}
        subtitle={t('terms.subtitle')}
        actions={
          <Button variant="primary" icon={Plus} onClick={() => setEditing('new')}>
            {t('terms.newTerm')}
          </Button>
        }
      />
      <Card>
        <FilterBar>
          <Field label={t('common.search')}>
            {(a) => (
              <div className="relative">
                <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
                <input {...a} type="search" className={cx(inputClass, 'w-64 pl-9')} value={q} placeholder={t('terms.searchPlaceholder')} onChange={(e) => setQ(e.target.value)} />
              </div>
            )}
          </Field>
          <Field label={t('terms.fieldLanguage')}>
            {(a) => (
              <select {...a} className={selectClass} value={language} onChange={(e) => setLanguage(e.target.value as '' | Language)}>
                <option value="">{t('common.all')}</option>
                <option value="vi">{t('language.vi')}</option>
                <option value="en">{t('language.en')}</option>
              </select>
            )}
          </Field>
          <Field label={t('terms.fieldStatus')}>
            {(a) => (
              <select {...a} className={selectClass} value={status} onChange={(e) => setStatus(e.target.value as '' | TermStatus)}>
                <option value="">{t('common.all')}</option>
                <option value="published">{t('terms.statusPublished')}</option>
                <option value="draft">{t('terms.statusDraft')}</option>
              </select>
            )}
          </Field>
          {filtered && (
            <Button
              onClick={() => {
                setQ('');
                setLanguage('');
                setStatus('');
              }}
            >
              {t('common.clearFilters')}
            </Button>
          )}
        </FilterBar>

        {list.isPending ? (
          <TableSkeleton cols={5} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : rows.length === 0 ? (
          <EmptyState
            title={filtered ? t('terms.noMatch') : t('terms.empty')}
            body={filtered ? undefined : t('terms.emptyBody')}
            action={
              filtered ? undefined : (
                <Button variant="primary" icon={Plus} onClick={() => setEditing('new')}>
                  {t('terms.newTerm')}
                </Button>
              )
            }
          />
        ) : (
          <TableWrap label={t('terms.title')}>
            <thead>
              <tr>
                <Th>{t('terms.colTerm')}</Th>
                <Th>{t('terms.colDefinition')}</Th>
                <Th>{t('terms.colSlug')}</Th>
                <Th>{t('terms.colStatus')}</Th>
                <Th className="text-right">{t('common.actions')}</Th>
              </tr>
            </thead>
            <tbody>
              {rows.map((term) => (
                <tr key={term.id} className="align-top hover:bg-surface-2/60">
                  <Td>
                    <p className="font-medium text-ink">{term.term}</p>
                    {term.english && term.english.toLowerCase() !== term.term.toLowerCase() && <p className="text-xs text-muted" lang="en">{term.english}</p>}
                    {term.aliases.length > 0 && (
                      <p className="mt-1 text-xs text-muted">
                        {t('terms.aliasCount', { count: term.aliases.length })}: {term.aliases.join(', ')}
                      </p>
                    )}
                  </Td>
                  <Td className="max-w-md">
                    <p className="line-clamp-2 text-sm text-muted">{term.shortDefinition}</p>
                  </Td>
                  <Td className="font-mono text-xs text-muted">
                    {term.slug}
                    <span className="ml-2 font-sans uppercase">{term.language}</span>
                  </Td>
                  <Td>
                    <div className="flex flex-wrap gap-1">
                      <Badge tone={term.status === 'published' ? 'success' : 'neutral'}>
                        {term.status === 'published' ? t('terms.statusPublished') : t('terms.statusDraft')}
                      </Badge>
                      {!term.autoLink && <Badge tone="warning">{t('terms.badgeManual')}</Badge>}
                      {term.caseSensitive && <Badge tone="indigo">{t('terms.badgeCaseSensitive')}</Badge>}
                      {term.editedInCms && <Badge tone="violet">{t('terms.badgeEdited')}</Badge>}
                    </div>
                  </Td>
                  <Td className="text-right whitespace-nowrap">
                    <IconButton icon={Pencil} label={`${t('common.edit')}: ${term.term}`} onClick={() => setEditing(term)} />
                    <IconButton
                      icon={Trash2}
                      label={`${t('common.delete')}: ${term.term}`}
                      className="hover:text-red-700 dark:hover:text-red-400"
                      onClick={() => {
                        setDeleteError(undefined);
                        setDeleting(term);
                      }}
                    />
                  </Td>
                </tr>
              ))}
            </tbody>
          </TableWrap>
        )}
        {list.data && list.data.length > 0 && (
          <p className="border-t border-line px-4 py-3 text-xs text-muted">
            {t('terms.count', { shown: formatNumber(rows.length, locale), total: formatNumber(list.data.length, locale) })}
          </p>
        )}
      </Card>

      {editing && <TermForm term={editing === 'new' ? undefined : editing} onClose={() => setEditing(null)} />}

      <ConfirmDialog
        open={deleting !== null}
        title={t('terms.confirmDeleteTitle', { name: deleting?.term ?? '' })}
        body={t('terms.confirmDeleteBody')}
        confirmLabel={t('common.delete')}
        loading={remove.isPending}
        error={deleteError}
        onConfirm={() => deleting && remove.mutate(deleting)}
        onCancel={() => setDeleting(null)}
      />
    </>
  );
}

interface FormState {
  term: string;
  slug: string;
  language: Language;
  english: string;
  /** Mỗi dòng một alias. */
  aliases: string;
  shortDefinition: string;
  body: string;
  caseSensitive: boolean;
  autoLink: boolean;
  status: TermStatus;
}
type Errors = Partial<Record<keyof FormState, string>>;

const FIELDS = new Set<keyof FormState>(['term', 'slug', 'language', 'english', 'aliases', 'shortDefinition', 'body', 'status']);

const parseAliases = (raw: string) => [...new Set(raw.split(/\r?\n/).map((s) => s.trim()).filter(Boolean))];

function TermForm({ term, onClose }: { term?: AdminTerm; onClose: () => void }) {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const client = useQueryClient();
  const toast = useToast();
  const [slugTouched, setSlugTouched] = useState(Boolean(term));
  const [form, setForm] = useState<FormState>(() => ({
    term: term?.term ?? '',
    slug: term?.slug ?? '',
    language: term?.language ?? 'vi',
    english: term?.english ?? '',
    aliases: term?.aliases.join('\n') ?? '',
    shortDefinition: term?.shortDefinition ?? '',
    body: term?.body ?? '',
    caseSensitive: term?.caseSensitive ?? false,
    autoLink: term?.autoLink ?? true,
    status: term?.status ?? 'published',
  }));
  const [errors, setErrors] = useState<Errors>({});
  const [usage, setUsage] = useState<TermUsage | null>(null);

  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => {
    setForm((f) => {
      const next = { ...f, [k]: v };
      if (k === 'term' && !slugTouched) next.slug = slugify(String(v));
      return next;
    });
    setErrors((e) => ({ ...e, [k]: undefined }));
    // Kết quả đếm cũ không còn đúng khi đổi cách dò.
    if (k === 'term' || k === 'aliases' || k === 'caseSensitive' || k === 'language') setUsage(null);
  };

  const validate = (): Errors => {
    const e: Errors = {};
    if (!form.term.trim()) e.term = t('terms.errTerm');
    if (!SLUG_RE.test(form.slug.trim())) e.slug = t('terms.errSlug');
    if (!form.shortDefinition.trim()) e.shortDefinition = t('terms.errDefinition');
    else if (form.shortDefinition.trim().length > SHORT_DEFINITION_MAX) e.shortDefinition = t('terms.errDefinitionLong', { max: SHORT_DEFINITION_MAX });
    return e;
  };

  const save = useMutation({
    mutationFn: (input: TermInput) => (term ? api.admin.terms.update(term.id, input) : api.admin.terms.create(input)),
    onSuccess: () => {
      toast.success(term ? t('terms.saved') : t('terms.created'));
      void client.invalidateQueries({ queryKey: LIST_KEY });
      void client.invalidateQueries({ queryKey: ['glossary'] });
      onClose();
    },
    onError: (err) => {
      if (isApiError(err) && err.fields) {
        const mapped: Errors = {};
        for (const [k, v] of Object.entries(err.fields)) {
          const key = k.split('.')[0] as keyof FormState;
          if (FIELDS.has(key)) mapped[key] = v;
        }
        setErrors(mapped);
      } else if (isApiError(err) && err.status === 409) {
        setErrors({ slug: errorMessage(err, t, i18n.language) });
      }
      toast.error(errorMessage(err, t, i18n.language));
    },
  });

  const check = useMutation({
    mutationFn: () => api.admin.terms.usage({ language: form.language, term: form.term.trim(), aliases: parseAliases(form.aliases), caseSensitive: form.caseSensitive }),
    onSuccess: setUsage,
    onError: (err) => toast.error(errorMessage(err, t, i18n.language)),
  });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const v = validate();
    setErrors(v);
    if (Object.keys(v).length > 0) return;
    save.mutate({
      term: form.term.trim(),
      slug: form.slug.trim(),
      language: form.language,
      english: form.english.trim() || undefined,
      aliases: parseAliases(form.aliases),
      shortDefinition: form.shortDefinition.trim(),
      body: form.body.trim() || undefined,
      caseSensitive: form.caseSensitive,
      autoLink: form.autoLink,
      status: form.status,
    });
  };

  const defLength = form.shortDefinition.trim().length;
  const formId = 'term-form';
  const showEnglish = form.english.trim() && form.english.trim().toLowerCase() !== form.term.trim().toLowerCase();
  return (
    <Modal
      open
      onClose={onClose}
      title={term ? t('terms.editTerm') : t('terms.newTerm')}
      size="lg"
      footer={
        <>
          <Button onClick={onClose}>{t('common.cancel')}</Button>
          <Button type="submit" form={formId} variant="primary" loading={save.isPending}>
            {t('common.save')}
          </Button>
        </>
      }
    >
      <form id={formId} onSubmit={submit} noValidate className="grid gap-4 sm:grid-cols-2">
        <Field label={t('terms.fieldTerm')} error={errors.term} hint={t('terms.hintTerm')} required>
          {(a) => <input {...a} className={inputClass} value={form.term} onChange={(e) => set('term', e.target.value)} autoFocus />}
        </Field>
        <Field label={t('terms.fieldEnglish')} error={errors.english} hint={t('terms.hintEnglish')}>
          {(a) => <input {...a} lang="en" className={inputClass} value={form.english} onChange={(e) => set('english', e.target.value)} />}
        </Field>
        <Field label={t('terms.fieldSlug')} error={errors.slug} required>
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
        <div className="grid grid-cols-2 gap-4">
          <Field label={t('terms.fieldLanguage')} error={errors.language}>
            {(a) => (
              <select {...a} className={selectClass} value={form.language} onChange={(e) => set('language', e.target.value as Language)}>
                <option value="vi">{t('language.vi')}</option>
                <option value="en">{t('language.en')}</option>
              </select>
            )}
          </Field>
          <Field label={t('terms.fieldStatus')} error={errors.status}>
            {(a) => (
              <select {...a} className={selectClass} value={form.status} onChange={(e) => set('status', e.target.value as TermStatus)}>
                <option value="published">{t('terms.statusPublished')}</option>
                <option value="draft">{t('terms.statusDraft')}</option>
              </select>
            )}
          </Field>
        </div>
        <div className="sm:col-span-2">
          <Field label={t('terms.fieldAliases')} error={errors.aliases} hint={t('terms.hintAliases')}>
            {(a) => <textarea {...a} rows={3} className={inputClass} value={form.aliases} onChange={(e) => set('aliases', e.target.value)} />}
          </Field>
        </div>
        <div className="sm:col-span-2">
          <Field
            label={t('terms.fieldDefinition')}
            error={errors.shortDefinition}
            hint={t('terms.hintDefinition')}
            required
            aside={
              <span className={cx('text-xs tabular-nums', defLength > SHORT_DEFINITION_MAX ? 'font-semibold text-red-700 dark:text-red-400' : 'text-muted')}>
                {defLength}/{SHORT_DEFINITION_MAX}
              </span>
            }
          >
            {(a) => <textarea {...a} rows={3} className={inputClass} value={form.shortDefinition} onChange={(e) => set('shortDefinition', e.target.value)} />}
          </Field>
        </div>
        <div className="sm:col-span-2">
          <Field label={t('terms.fieldBody')} error={errors.body} hint={t('terms.hintBody')}>
            {(a) => <textarea {...a} rows={4} className={cx(inputClass, 'font-mono text-xs')} value={form.body} onChange={(e) => set('body', e.target.value)} />}
          </Field>
        </div>
        <fieldset className="space-y-2 sm:col-span-2">
          <legend className="mb-1 text-sm font-medium text-ink">{t('terms.fieldMatching')}</legend>
          <label className="flex items-start gap-2 text-sm text-ink">
            <input type="checkbox" className="mt-0.5 size-4 accent-[var(--color-primary)]" checked={form.autoLink} onChange={(e) => set('autoLink', e.target.checked)} />
            <span>
              {t('terms.autoLink')}
              <span className="block text-xs text-muted">{t('terms.autoLinkHint')}</span>
            </span>
          </label>
          <label className="flex items-start gap-2 text-sm text-ink">
            <input type="checkbox" className="mt-0.5 size-4 accent-[var(--color-primary)]" checked={form.caseSensitive} onChange={(e) => set('caseSensitive', e.target.checked)} />
            <span>
              {t('terms.caseSensitive')}
              <span className="block text-xs text-muted">{t('terms.caseSensitiveHint')}</span>
            </span>
          </label>
        </fieldset>

        <div className="sm:col-span-2">
          <p className="mb-1.5 text-sm font-medium text-ink">{t('terms.preview')}</p>
          <div className="rounded-card border border-line bg-page p-4">
            <div className="max-w-sm rounded-xl border border-line bg-surface p-4 shadow-lg">
              <p className="text-sm font-semibold text-ink">{form.term.trim() || '—'}</p>
              {showEnglish && (
                <p className="mt-0.5 text-xs text-muted">
                  {t('terms.previewEnglish')}: <span lang="en">{form.english.trim()}</span>
                </p>
              )}
              <p className="mt-2 text-sm leading-relaxed text-ink">{form.shortDefinition.trim() || t('terms.previewEmpty')}</p>
            </div>
          </div>
        </div>

        <div className="space-y-3 sm:col-span-2">
          <div className="flex flex-wrap items-center gap-3">
            <Button icon={ScanSearch} loading={check.isPending} disabled={!form.term.trim()} onClick={() => check.mutate()}>
              {t('terms.checkUsage')}
            </Button>
            <p className="text-xs text-muted">{t('terms.checkUsageHint')}</p>
          </div>
          {usage && (
            <div role="status" className="rounded-card border border-line bg-page p-4">
              <p className="text-sm font-medium text-ink">
                {usage.articles === 0
                  ? t('terms.usageNone')
                  : t('terms.usageSummary', { articles: formatNumber(usage.articles, locale), occurrences: formatNumber(usage.occurrences, locale) })}
              </p>
              {usage.samples.length > 0 && (
                <ul className="mt-3 space-y-2">
                  {usage.samples.map((s) => (
                    <li key={s.id} className="text-sm">
                      <div className="flex flex-wrap items-center gap-2">
                        <Link to={`/articles/${s.id}`} target="_blank" className="font-medium text-primary hover:underline">
                          {s.title}
                        </Link>
                        {s.status !== 'published' && <Badge tone="neutral">{t(`articleStatus.${s.status}`)}</Badge>}
                        <span className="text-xs text-muted">{t('terms.usageTimes', { count: s.occurrences })}</span>
                      </div>
                      <p className="mt-0.5 text-xs text-muted">{s.excerpt}</p>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>
      </form>
    </Modal>
  );
}
