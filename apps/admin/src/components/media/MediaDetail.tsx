// Chi tiết một ảnh: xem trước, thông tin, sửa alt, sao chép URL/Markdown, xoá (xác nhận trong trang).
import { useEffect, useState, type ReactNode } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { ClipboardCopy, ExternalLink, Link as LinkIcon, Trash, X } from 'lucide-react';
import { api, type MediaItem } from '@tkh/shared';
import { Button, ConfirmDialog, Field, IconButton, inputClass } from '../ui';
import { errorMessage } from '../../lib/errors';
import { formatDateTime } from '../../lib/articles';
import { copyText, defaultAlt, formatBytes, imageMarkdown } from '../../lib/media';
import { useToast } from '../../lib/toast';
import { useLocale } from '../../i18n';

const ALT_MAX = 200;

export function MediaDetail({
  item,
  onClose,
  onChanged,
  onDeleted,
  extraActions,
  headingId,
}: {
  item: MediaItem;
  onClose?: () => void;
  onChanged?: (item: MediaItem) => void;
  onDeleted?: (id: string) => void;
  extraActions?: ReactNode;
  headingId?: string;
}) {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const toast = useToast();
  const client = useQueryClient();
  const [alt, setAlt] = useState(item.alt ?? '');
  const [confirmDelete, setConfirmDelete] = useState(false);
  useEffect(() => setAlt(item.alt ?? ''), [item.id, item.alt]);

  const saveAlt = useMutation({
    mutationFn: () => api.admin.media.update(item.id, { alt: alt.trim() }),
    onSuccess: (updated) => {
      toast.success(t('media.altSaved'));
      void client.invalidateQueries({ queryKey: ['admin', 'media'] });
      onChanged?.(updated);
    },
    onError: (err) => toast.error(errorMessage(err, t, i18n.language)),
  });

  const remove = useMutation({
    mutationFn: () => api.admin.media.remove(item.id),
    onSuccess: () => {
      setConfirmDelete(false);
      toast.success(t('media.deleted'));
      void client.invalidateQueries({ queryKey: ['admin', 'media'] });
      onDeleted?.(item.id);
    },
  });

  const copy = async (text: string) => {
    if (await copyText(text)) toast.success(t('media.copied'));
    else toast.error(t('media.copyFailed'));
  };

  const altDirty = alt.trim() !== (item.alt ?? '');
  const tooLong = alt.trim().length > ALT_MAX;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-start justify-between gap-2">
        <h2 id={headingId} className="text-base font-semibold text-ink">
          {t('media.detailTitle')}
        </h2>
        {onClose && <IconButton icon={X} label={t('media.closeDetail')} onClick={onClose} className="-mt-1 -mr-2" />}
      </div>
      <a href={item.url} target="_blank" rel="noreferrer" className="block overflow-hidden rounded-ui border border-line bg-surface-2" title={t('media.openOriginal')}>
        <img src={item.url} alt={item.alt ?? ''} className="mx-auto max-h-64 w-auto object-contain" />
      </a>
      <dl className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1.5 text-sm">
        <dt className="text-muted">{t('media.fileName')}</dt>
        <dd className="truncate text-ink" title={item.fileName}>
          {item.fileName}
        </dd>
        <dt className="text-muted">{t('media.type')}</dt>
        <dd className="text-ink">{item.mimeType}</dd>
        <dt className="text-muted">{t('media.size')}</dt>
        <dd className="text-ink">{formatBytes(item.size, locale)}</dd>
        {item.width && item.height ? (
          <>
            <dt className="text-muted">{t('media.dimensions')}</dt>
            <dd className="text-ink tabular-nums">
              {item.width} × {item.height}
            </dd>
          </>
        ) : null}
        <dt className="text-muted">{t('media.uploadedBy')}</dt>
        <dd className="truncate text-ink">{item.uploadedBy.displayName}</dd>
        <dt className="text-muted">{t('media.uploadedAt')}</dt>
        <dd className="text-ink">{formatDateTime(item.createdAt, locale)}</dd>
        <dt className="text-muted">{t('media.url')}</dt>
        <dd className="truncate font-mono text-xs text-ink" title={item.url}>
          {item.url}
        </dd>
      </dl>

      <Field
        label={t('media.alt')}
        hint={t('media.altHint')}
        aside={<span className="text-xs text-muted tabular-nums">{alt.trim().length}/{ALT_MAX}</span>}
      >
        {(a) => (
          <textarea
            {...a}
            rows={2}
            maxLength={ALT_MAX}
            className={inputClass}
            value={alt}
            onChange={(e) => setAlt(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                if (altDirty && !tooLong) saveAlt.mutate();
              }
            }}
          />
        )}
      </Field>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" variant="primary" onClick={() => saveAlt.mutate()} loading={saveAlt.isPending} disabled={!altDirty || tooLong}>
          {t('media.saveAlt')}
        </Button>
        <Button size="sm" icon={LinkIcon} onClick={() => void copy(item.url)}>
          {t('media.copyUrl')}
        </Button>
        <Button size="sm" icon={ClipboardCopy} onClick={() => void copy(imageMarkdown(item.url, defaultAlt(item)))}>
          {t('media.copyMarkdown')}
        </Button>
        <a href={item.url} target="_blank" rel="noreferrer" className="inline-flex h-8 items-center gap-1.5 rounded-ui px-2 text-sm font-medium text-primary-ink hover:underline">
          <ExternalLink className="size-4" aria-hidden />
          {t('media.openOriginal')}
        </a>
      </div>
      {extraActions}
      <div className="border-t border-line pt-4">
        <Button size="sm" variant="ghost" icon={Trash} className="text-red-700 dark:text-red-400" onClick={() => setConfirmDelete(true)}>
          {t('media.delete')}
        </Button>
      </div>

      <ConfirmDialog
        open={confirmDelete}
        title={t('media.confirmDeleteTitle', { name: item.fileName })}
        body={t('media.confirmDeleteBody')}
        confirmLabel={t('media.delete')}
        loading={remove.isPending}
        error={remove.isError ? errorMessage(remove.error, t, i18n.language) : undefined}
        onConfirm={() => remove.mutate()}
        onCancel={() => {
          setConfirmDelete(false);
          remove.reset();
        }}
      />
    </div>
  );
}
