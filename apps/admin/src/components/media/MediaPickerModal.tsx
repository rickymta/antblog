// Hộp chọn ảnh từ thư viện (ảnh bìa, ảnh chia sẻ, ảnh chèn vào bài). Tải ảnh mới ngay trong hộp ⇒ tự chọn ảnh vừa tải.
import { useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { api, type MediaItem } from '@tkh/shared';
import { Button, Modal, cx, inputClass } from '../ui';
import { defaultAlt } from '../../lib/media';
import { MediaBrowser } from './MediaBrowser';

export function MediaPickerModal({
  open,
  onClose,
  onPick,
  confirmLabel,
  withAlt,
}: {
  open: boolean;
  onClose: () => void;
  /** `alt` chỉ có khi `withAlt` (chèn ảnh vào nội dung). */
  onPick: (item: MediaItem, alt?: string) => void;
  confirmLabel?: string;
  withAlt?: boolean;
}) {
  const { t } = useTranslation();
  const client = useQueryClient();
  const [selected, setSelected] = useState<MediaItem | null>(null);
  const [alt, setAlt] = useState('');

  useEffect(() => {
    if (!open) {
      setSelected(null);
      setAlt('');
    }
  }, [open]);

  const select = (item: MediaItem) => {
    setSelected(item);
    setAlt(defaultAlt(item));
  };

  const confirm = (item: MediaItem | null = selected) => {
    if (!item) return;
    const finalAlt = withAlt ? (item.id === selected?.id ? alt : defaultAlt(item)).trim() : undefined;
    // Người dùng gõ alt mới cho ảnh chưa có alt ⇒ lưu luôn vào thư viện (không chặn việc chèn nếu lỗi).
    if (withAlt && finalAlt && !item.alt && finalAlt !== defaultAlt(item)) {
      void api.admin.media
        .update(item.id, { alt: finalAlt.slice(0, 200) })
        .then(() => client.invalidateQueries({ queryKey: ['admin', 'media'] }))
        .catch(() => undefined);
    }
    onPick(item, finalAlt);
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={t('media.pickerTitle')}
      size="xl"
      footer={
        <div className="flex w-full flex-wrap items-center gap-3">
          <div className="min-w-0 flex-1">
            {selected ? (
              <div className="flex flex-wrap items-center gap-3">
                <span className="max-w-[16rem] truncate text-sm font-medium text-ink">{t('media.selectedLabel', { name: selected.fileName })}</span>
                {withAlt && (
                  <label className="flex min-w-[14rem] flex-1 items-center gap-2 text-sm">
                    <span className="shrink-0 text-muted">{t('media.pickerAlt')}</span>
                    <input className={cx(inputClass, 'h-9 py-1')} value={alt} maxLength={200} onChange={(e) => setAlt(e.target.value)} />
                  </label>
                )}
              </div>
            ) : (
              <span className="text-sm text-muted">{t('media.noneSelected')}</span>
            )}
          </div>
          <Button onClick={onClose}>{t('common.cancel')}</Button>
          <Button variant="primary" disabled={!selected} onClick={() => confirm()}>
            {confirmLabel ?? t('media.pick')}
          </Button>
        </div>
      }
    >
      <div className="-mx-5 -my-4">
        {open && (
          <MediaBrowser
            compact
            selectedId={selected?.id}
            onSelect={select}
            onActivate={(item) => confirm(item)}
            onUploaded={(items) => items[0] && select(items[0])}
          />
        )}
      </div>
    </Modal>
  );
}
