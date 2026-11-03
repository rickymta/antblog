import { useState, type FormEvent } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Pencil, Plus, Trash2 } from 'lucide-react';
import { api, CATEGORY_ICONS, formatNumber, pickLocalized, slugify, type Category, type CategoryIcon as CategoryIconName, type CategoryInput } from '@tkh/shared';
import {
  Button,
  Card,
  ConfirmDialog,
  EmptyState,
  ErrorState,
  Field,
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
import { CategoryIcon } from '../components/domain';
import { errorMessage, hasCode, isApiError } from '../lib/errors';
import { SLUG_RE } from '../lib/site';
import { useToast } from '../lib/toast';
import { useLocale } from '../i18n';

const LIST_KEY = ['admin', 'categories'] as const;
const HEX_RE = /^#[0-9a-fA-F]{6}$/;

export function CategoriesPage() {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const client = useQueryClient();
  const toast = useToast();
  const list = useQuery({ queryKey: LIST_KEY, queryFn: () => api.admin.categories.list() });
  const [editing, setEditing] = useState<Category | 'new' | null>(null);
  const [deleting, setDeleting] = useState<Category | null>(null);
  const [deleteError, setDeleteError] = useState<string | undefined>();

  const remove = useMutation({
    mutationFn: (c: Category) => api.admin.categories.remove(c.id),
    onSuccess: () => {
      setDeleting(null);
      toast.success(t('categories.deleted'));
      void client.invalidateQueries({ queryKey: LIST_KEY });
      void client.invalidateQueries({ queryKey: ['categories'] });
    },
    onError: (err) => {
      if (hasCode(err, 'CATEGORY_IN_USE')) {
        // articleCount công khai chỉ đếm bài đã xuất bản; API chặn cả bản nháp/lưu trữ ⇒ chỉ nêu số khi API trả kèm.
        const count = isApiError(err) && typeof err.details?.articleCount === 'number' ? err.details.articleCount : 0;
        setDeleteError(count > 0 ? t('categories.inUse', { count }) : t('categories.inUseGeneric'));
      } else {
        setDeleteError(errorMessage(err, t, i18n.language));
      }
    },
  });

  const sorted = [...(list.data ?? [])].sort((a, b) => a.order - b.order);

  return (
    <>
      <PageHeader
        title={t('categories.title')}
        subtitle={t('categories.subtitle')}
        actions={
          <Button variant="primary" icon={Plus} onClick={() => setEditing('new')}>
            {t('categories.newCategory')}
          </Button>
        }
      />
      <Card>
        {list.isPending ? (
          <TableSkeleton cols={5} />
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : sorted.length === 0 ? (
          <EmptyState
            title={t('categories.empty')}
            action={
              <Button variant="primary" icon={Plus} onClick={() => setEditing('new')}>
                {t('categories.newCategory')}
              </Button>
            }
          />
        ) : (
          <TableWrap label={t('categories.title')}>
            <thead>
              <tr>
                <Th className="w-16">{t('categories.colIcon')}</Th>
                <Th>{t('categories.colName')}</Th>
                <Th>{t('categories.colSlug')}</Th>
                <Th className="text-right">{t('categories.colOrder')}</Th>
                <Th className="text-right">{t('categories.colArticles')}</Th>
                <Th className="text-right">{t('common.actions')}</Th>
              </tr>
            </thead>
            <tbody>
              {sorted.map((c) => {
                const name = pickLocalized(c.name, locale);
                return (
                  <tr key={c.id} className="hover:bg-surface-2/60">
                    <Td>
                      <CategoryIcon icon={c.icon} color={c.color} />
                    </Td>
                    <Td>
                      <p className="font-medium">{name}</p>
                      <p className="line-clamp-1 text-xs text-muted">{pickLocalized(c.description, locale)}</p>
                    </Td>
                    <Td className="font-mono text-xs text-muted">{c.slug}</Td>
                    <Td className="text-right tabular-nums">{c.order}</Td>
                    <Td className="text-right tabular-nums">{formatNumber(c.articleCount ?? 0, locale)}</Td>
                    <Td className="text-right whitespace-nowrap">
                      <IconButton icon={Pencil} label={`${t('common.edit')}: ${name}`} onClick={() => setEditing(c)} />
                      <IconButton
                        icon={Trash2}
                        label={`${t('common.delete')}: ${name}`}
                        className="hover:text-red-700 dark:hover:text-red-400"
                        onClick={() => {
                          setDeleteError(undefined);
                          setDeleting(c);
                        }}
                      />
                    </Td>
                  </tr>
                );
              })}
            </tbody>
          </TableWrap>
        )}
      </Card>

      {editing && (
        <CategoryForm
          category={editing === 'new' ? undefined : editing}
          nextOrder={sorted.length > 0 ? Math.max(...sorted.map((c) => c.order)) + 1 : 1}
          onClose={() => setEditing(null)}
        />
      )}

      <ConfirmDialog
        open={deleting !== null}
        title={t('categories.confirmDeleteTitle', { name: deleting ? pickLocalized(deleting.name, locale) : '' })}
        body={t('categories.confirmDeleteBody')}
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
  slug: string;
  nameVi: string;
  nameEn: string;
  descVi: string;
  descEn: string;
  icon: CategoryIconName;
  color: string;
  order: string;
}
type Errors = Partial<Record<keyof FormState, string>>;

/** Lỗi API theo đường dẫn trường (`name.vi`, `description.en`…) ⇒ ô tương ứng của form. */
const API_FIELD_MAP: Record<string, keyof FormState> = {
  slug: 'slug',
  'name.vi': 'nameVi',
  'name.en': 'nameEn',
  name: 'nameVi',
  'description.vi': 'descVi',
  'description.en': 'descEn',
  description: 'descVi',
  icon: 'icon',
  color: 'color',
  order: 'order',
};

function CategoryForm({ category, nextOrder, onClose }: { category?: Category; nextOrder: number; onClose: () => void }) {
  const { t, i18n } = useTranslation();
  const client = useQueryClient();
  const toast = useToast();
  const [slugTouched, setSlugTouched] = useState(Boolean(category));
  const [form, setForm] = useState<FormState>(() => ({
    slug: category?.slug ?? '',
    nameVi: category?.name.vi ?? '',
    nameEn: category?.name.en ?? '',
    descVi: category?.description.vi ?? '',
    descEn: category?.description.en ?? '',
    icon: category?.icon ?? 'code',
    color: category?.color ?? '#2563EB',
    order: String(category?.order ?? nextOrder),
  }));
  const [errors, setErrors] = useState<Errors>({});

  const set = <K extends keyof FormState>(k: K, v: FormState[K]) => {
    setForm((f) => {
      const next = { ...f, [k]: v };
      if (k === 'nameVi' && !slugTouched) next.slug = slugify(String(v));
      return next;
    });
    setErrors((e) => ({ ...e, [k]: undefined }));
  };

  const validate = (): Errors => {
    const e: Errors = {};
    if (!SLUG_RE.test(form.slug.trim())) e.slug = t('categories.errSlug');
    if (!form.nameVi.trim()) e.nameVi = t('categories.errName');
    if (!form.nameEn.trim()) e.nameEn = t('categories.errName');
    if (!HEX_RE.test(form.color)) e.color = t('categories.errColor');
    if (!/^-?\d+$/.test(form.order.trim())) e.order = t('categories.errOrder');
    return e;
  };

  const save = useMutation({
    mutationFn: (input: CategoryInput) =>
      category ? api.admin.categories.update(category.id, input) : api.admin.categories.create(input),
    onSuccess: () => {
      toast.success(category ? t('categories.saved') : t('categories.created'));
      void client.invalidateQueries({ queryKey: LIST_KEY });
      void client.invalidateQueries({ queryKey: ['categories'] });
      onClose();
    },
    onError: (err) => {
      if (isApiError(err) && err.fields) {
        const mapped: Errors = {};
        for (const [k, v] of Object.entries(err.fields)) {
          const target = API_FIELD_MAP[k];
          if (target) mapped[target] = v;
        }
        setErrors(mapped);
      } else if (hasCode(err, 'SLUG_TAKEN') || hasStatus409(err)) {
        setErrors({ slug: errorMessage(err, t, i18n.language) });
      }
      toast.error(errorMessage(err, t, i18n.language));
    },
  });

  const submit = (e: FormEvent) => {
    e.preventDefault();
    const v = validate();
    setErrors(v);
    if (Object.keys(v).length > 0) return;
    save.mutate({
      slug: form.slug.trim(),
      name: { vi: form.nameVi.trim(), en: form.nameEn.trim() },
      description: { vi: form.descVi.trim(), en: form.descEn.trim() },
      icon: form.icon,
      color: form.color.toUpperCase(),
      order: Number(form.order.trim()),
    });
  };

  const formId = 'category-form';
  return (
    <Modal
      open
      onClose={onClose}
      title={category ? t('categories.editCategory') : t('categories.newCategory')}
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
        <Field label={t('categories.fieldNameVi')} error={errors.nameVi} required>
          {(a) => <input {...a} className={inputClass} value={form.nameVi} onChange={(e) => set('nameVi', e.target.value)} autoFocus />}
        </Field>
        <Field label={t('categories.fieldNameEn')} error={errors.nameEn} required>
          {(a) => <input {...a} className={inputClass} value={form.nameEn} onChange={(e) => set('nameEn', e.target.value)} />}
        </Field>
        <Field label={t('categories.fieldSlug')} error={errors.slug} required>
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
        <Field label={t('categories.fieldOrder')} error={errors.order}>
          {(a) => <input {...a} type="number" step={1} className={inputClass} value={form.order} onChange={(e) => set('order', e.target.value)} />}
        </Field>
        <Field label={t('categories.fieldDescVi')} error={errors.descVi}>
          {(a) => <textarea {...a} rows={3} className={inputClass} value={form.descVi} onChange={(e) => set('descVi', e.target.value)} />}
        </Field>
        <Field label={t('categories.fieldDescEn')} error={errors.descEn}>
          {(a) => <textarea {...a} rows={3} className={inputClass} value={form.descEn} onChange={(e) => set('descEn', e.target.value)} />}
        </Field>
        <Field label={t('categories.fieldIcon')} error={errors.icon}>
          {(a) => (
            <select {...a} className={selectClass} value={form.icon} onChange={(e) => set('icon', e.target.value as CategoryIconName)}>
              {CATEGORY_ICONS.map((ic) => (
                <option key={ic} value={ic}>
                  {t(`categories.icons.${ic}`)} ({ic})
                </option>
              ))}
            </select>
          )}
        </Field>
        <Field label={t('categories.fieldColor')} error={errors.color}>
          {(a) => (
            <div className="flex items-center gap-2">
              <input
                type="color"
                aria-label={t('categories.fieldColor')}
                className="h-10 w-12 shrink-0 cursor-pointer rounded-ui border border-line bg-surface p-1"
                value={HEX_RE.test(form.color) ? form.color : '#2563EB'}
                onChange={(e) => set('color', e.target.value.toUpperCase())}
              />
              <input {...a} className={cx(inputClass, 'font-mono uppercase')} value={form.color} maxLength={7} onChange={(e) => set('color', e.target.value)} />
            </div>
          )}
        </Field>
        <div className="sm:col-span-2">
          <p className="mb-1.5 text-sm font-medium text-ink">{t('categories.preview')}</p>
          <div className="flex items-center gap-3 rounded-card border border-line bg-page p-3">
            <CategoryIcon icon={form.icon} color={HEX_RE.test(form.color) ? form.color : '#64748B'} size="lg" />
            <div className="min-w-0">
              <p className="font-semibold text-ink">{(i18n.language === 'en' ? form.nameEn : form.nameVi) || form.nameVi || '—'}</p>
              <p className="line-clamp-2 text-sm text-muted">{(i18n.language === 'en' ? form.descEn : form.descVi) || form.descVi}</p>
            </div>
          </div>
        </div>
      </form>
    </Modal>
  );
}

function hasStatus409(err: unknown): boolean {
  return isApiError(err) && err.status === 409;
}
