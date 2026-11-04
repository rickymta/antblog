// Lưới ảnh thư viện media dùng chung cho trang /media và hộp chọn ảnh trong trình soạn bài:
// tìm kiếm, phân trang, tải nhiều ảnh (nút + kéo thả), chọn ảnh (bấm), dùng ngay (bấm đúp / Enter).
import { useEffect, useRef, useState, type DragEvent, type KeyboardEvent } from 'react';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { CircleAlert, CircleCheck, ImageOff, LoaderCircle, Search, Upload, X } from 'lucide-react';
import { api, type MediaItem } from '@tkh/shared';
import { Button, EmptyState, ErrorState, IconButton, Pagination, Skeleton, cx, inputClass } from '../ui';
import { errorMessage } from '../../lib/errors';
import { useDebounced } from '../../lib/hooks';
import { MEDIA_ACCEPT, formatBytes } from '../../lib/media';
import { useLocale } from '../../i18n';
import { useMediaUpload } from './useMediaUpload';

export const MEDIA_PAGE_SIZE = 24;

export function MediaBrowser({
  selectedId,
  onSelect,
  onActivate,
  onUploaded,
  compact,
}: {
  selectedId?: string;
  onSelect: (item: MediaItem) => void;
  onActivate?: (item: MediaItem) => void;
  /** Gọi sau mỗi lượt tải lên với các ảnh tải thành công. */
  onUploaded?: (items: MediaItem[]) => void;
  /** Hộp chọn ảnh: lưới dày hơn, chiều cao giới hạn. */
  compact?: boolean;
}) {
  const { t, i18n } = useTranslation();
  const locale = useLocale();
  const [text, setText] = useState('');
  const q = useDebounced(text.trim(), 300);
  const [page, setPage] = useState(1);
  const [dragging, setDragging] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const { uploads, uploadFiles, dismiss } = useMediaUpload();

  useEffect(() => setPage(1), [q]);

  const query = { q: q || undefined, page, pageSize: MEDIA_PAGE_SIZE };
  const list = useQuery({
    queryKey: ['admin', 'media', 'list', query],
    queryFn: () => api.admin.media.list(query),
    placeholderData: keepPreviousData,
  });

  // Trang hiện tại trống sau khi xoá ảnh cuối ⇒ lùi một trang.
  useEffect(() => {
    if (list.data && list.data.items.length === 0 && page > 1) setPage(page - 1);
  }, [list.data, page]);

  const handleFiles = async (files: FileList | File[] | null) => {
    const arr = files ? Array.from(files) : [];
    if (!arr.length) return;
    if (page !== 1) setPage(1);
    const done = await uploadFiles(arr);
    if (done.length) onUploaded?.(done);
  };

  const hasFiles = (e: DragEvent) => Array.from(e.dataTransfer.types).includes('Files');
  const onDragOver = (e: DragEvent) => {
    if (!hasFiles(e)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    setDragging(true);
  };
  const onDragLeave = (e: DragEvent) => {
    if (e.currentTarget.contains(e.relatedTarget as Node | null)) return;
    setDragging(false);
  };
  const onDrop = (e: DragEvent) => {
    if (!hasFiles(e)) return;
    e.preventDefault();
    setDragging(false);
    void handleFiles(e.dataTransfer.files);
  };

  const onTileKey = (e: KeyboardEvent<HTMLButtonElement>, item: MediaItem) => {
    if (e.key === 'Enter' && onActivate) {
      e.preventDefault();
      onActivate(item);
    }
  };

  return (
    <div className="relative flex min-h-0 flex-col" onDragOver={onDragOver} onDragLeave={onDragLeave} onDrop={onDrop}>
      {/* Thanh công cụ */}
      <div className="flex flex-wrap items-center gap-3 border-b border-line p-4">
        <label className="relative min-w-[12rem] flex-1">
          <span className="sr-only">{t('common.search')}</span>
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted" aria-hidden />
          <input
            type="search"
            className={cx(inputClass, 'h-10 pl-9')}
            placeholder={t('media.searchPlaceholder')}
            value={text}
            onChange={(e) => setText(e.target.value)}
          />
        </label>
        <Button icon={Upload} variant="primary" onClick={() => fileInput.current?.click()}>
          {t('media.upload')}
        </Button>
        <input
          ref={fileInput}
          type="file"
          accept={MEDIA_ACCEPT}
          multiple
          className="sr-only"
          tabIndex={-1}
          aria-hidden
          onChange={(e) => {
            void handleFiles(e.target.files);
            e.target.value = '';
          }}
        />
        <p className="w-full text-xs text-muted">
          {t('media.dropHere')}{' '}
          <button type="button" className="font-medium text-primary-ink hover:underline" onClick={() => fileInput.current?.click()}>
            {t('media.chooseFiles')}
          </button>{' '}
          · {t('media.limits')}
        </p>
      </div>

      {/* Tiến trình tải */}
      {uploads.length > 0 && (
        <ul aria-label={t('media.uploadList')} aria-live="polite" className="max-h-40 space-y-1 overflow-y-auto border-b border-line px-4 py-2">
          {uploads.map((u) => (
            <li key={u.key} className="flex items-center gap-2 text-sm">
              {u.status === 'uploading' ? (
                <LoaderCircle className="size-4 shrink-0 animate-spin text-primary" aria-hidden />
              ) : u.status === 'done' ? (
                <CircleCheck className="size-4 shrink-0 text-emerald-600 dark:text-emerald-400" aria-hidden />
              ) : (
                <CircleAlert className="size-4 shrink-0 text-red-600 dark:text-red-400" aria-hidden />
              )}
              <span className="min-w-0 flex-1 truncate text-ink">
                {u.name}
                <span className={cx('ml-2 text-xs', u.status === 'error' ? 'font-medium text-red-700 dark:text-red-400' : 'text-muted')}>
                  {u.status === 'uploading' ? t('media.uploading') : u.status === 'done' ? t('media.uploaded') : u.error}
                </span>
              </span>
              {u.status !== 'uploading' && (
                <IconButton icon={X} label={t('media.dismiss')} className="size-7" onClick={() => dismiss(u.key)} />
              )}
            </li>
          ))}
        </ul>
      )}

      {/* Lưới */}
      <div className={cx('min-h-0 flex-1 overflow-y-auto p-4', compact && 'max-h-[min(60dvh,36rem)]')}>
        {list.isPending ? (
          <div className={gridClass(compact)} role="status" aria-label={t('common.loading')}>
            {Array.from({ length: 12 }, (_, i) => (
              <Skeleton key={i} className="aspect-square" />
            ))}
          </div>
        ) : list.isError ? (
          <ErrorState message={errorMessage(list.error, t, i18n.language)} onRetry={() => void list.refetch()} />
        ) : list.data.items.length === 0 ? (
          q ? (
            <EmptyState title={t('states.emptyFiltered')} action={<Button onClick={() => setText('')}>{t('common.clearFilters')}</Button>} />
          ) : (
            <EmptyState
              title={t('media.empty')}
              body={t('media.emptyHint')}
              action={
                <Button icon={Upload} variant="primary" onClick={() => fileInput.current?.click()}>
                  {t('media.upload')}
                </Button>
              }
            />
          )
        ) : (
          <>
            <p className="mb-3 text-xs text-muted">{t('media.count', { count: list.data.total })}</p>
            <ul aria-label={t('media.grid')} className={cx(gridClass(compact), list.isPlaceholderData && 'opacity-60')}>
              {list.data.items.map((m) => (
                <li key={m.id}>
                  <MediaTile
                    item={m}
                    selected={m.id === selectedId}
                    onClick={() => onSelect(m)}
                    onDoubleClick={onActivate ? () => onActivate(m) : undefined}
                    onKeyDown={(e) => onTileKey(e, m)}
                    sizeLabel={formatBytes(m.size, locale)}
                  />
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
      {list.data && list.data.total > list.data.pageSize && (
        <Pagination page={list.data.page} pageSize={list.data.pageSize} total={list.data.total} onPage={setPage} />
      )}

      {dragging && (
        <div className="pointer-events-none absolute inset-2 z-10 flex items-center justify-center rounded-card border-2 border-dashed border-primary bg-primary-soft/90 text-base font-semibold text-primary-ink">
          <Upload className="mr-2 size-5" aria-hidden />
          {t('media.dropActive')}
        </div>
      )}
    </div>
  );
}

function gridClass(compact?: boolean): string {
  return cx(
    'grid gap-3',
    compact ? 'grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6' : 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-6',
  );
}

function MediaTile({
  item,
  selected,
  onClick,
  onDoubleClick,
  onKeyDown,
  sizeLabel,
}: {
  item: MediaItem;
  selected: boolean;
  onClick: () => void;
  onDoubleClick?: () => void;
  onKeyDown: (e: KeyboardEvent<HTMLButtonElement>) => void;
  sizeLabel: string;
}) {
  const { t } = useTranslation();
  const [broken, setBroken] = useState(false);
  return (
    <button
      type="button"
      aria-pressed={selected}
      aria-label={t('media.selectItem', { name: item.alt || item.fileName })}
      title={item.fileName}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      onKeyDown={onKeyDown}
      className={cx(
        'group block w-full overflow-hidden rounded-ui border bg-surface-2 text-left transition-shadow',
        selected ? 'border-primary ring-2 ring-primary' : 'border-line hover:border-primary/60',
      )}
    >
      <span className="relative block aspect-square">
        {broken ? (
          <span className="flex size-full items-center justify-center text-muted">
            <ImageOff className="size-6" aria-hidden />
          </span>
        ) : (
          <img
            src={item.url}
            alt=""
            loading="lazy"
            decoding="async"
            className="size-full object-cover"
            onError={() => setBroken(true)}
          />
        )}
        {selected && (
          <span className="absolute top-1.5 right-1.5 rounded-full bg-primary p-0.5 text-white shadow">
            <CircleCheck className="size-4" aria-hidden />
          </span>
        )}
      </span>
      <span className="block truncate px-2 pt-1.5 text-xs font-medium text-ink">{item.fileName}</span>
      <span className="block truncate px-2 pb-1.5 text-[11px] text-muted">
        {item.width && item.height ? `${item.width}×${item.height} · ` : ''}
        {sizeLabel}
      </span>
    </button>
  );
}
