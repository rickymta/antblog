import { useEffect, useId, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { MediaItem } from '@tkh/shared';
import { Card, PageHeader } from '../components/ui';
import { MediaBrowser } from '../components/media/MediaBrowser';
import { MediaDetail } from '../components/media/MediaDetail';
import { useIsAdmin } from '../lib/session';

/** Trang Thư viện ảnh (`/media`, admin + author — tác giả chỉ thấy ảnh của mình, API lọc). */
export function MediaPage() {
  const { t } = useTranslation();
  const isAdmin = useIsAdmin();
  const [selected, setSelected] = useState<MediaItem | null>(null);
  const headingId = useId();

  useEffect(() => {
    if (!selected) return;
    const onKey = (e: KeyboardEvent) => {
      // Hộp xác nhận (dialog) tự xử lý Esc; chỉ đóng ngăn chi tiết khi không có dialog mở.
      if (e.key === 'Escape' && !document.querySelector('dialog[open]')) setSelected(null);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selected]);

  return (
    <>
      <PageHeader title={t('media.title')} subtitle={isAdmin ? t('media.subtitle') : t('media.authorSubtitle')} />
      <Card>
        <MediaBrowser selectedId={selected?.id} onSelect={setSelected} onActivate={setSelected} />
      </Card>

      {selected && (
        <div className="fixed inset-0 z-40">
          <button type="button" className="absolute inset-0 bg-slate-950/40" aria-label={t('media.closeDetail')} onClick={() => setSelected(null)} />
          <aside
            role="dialog"
            aria-modal="true"
            aria-labelledby={headingId}
            className="absolute inset-y-0 right-0 w-full max-w-md overflow-y-auto border-l border-line bg-surface p-5 shadow-2xl"
          >
            <MediaDetail
              key={selected.id}
              item={selected}
              headingId={headingId}
              onClose={() => setSelected(null)}
              onChanged={setSelected}
              onDeleted={() => setSelected(null)}
            />
          </aside>
        </div>
      )}
    </>
  );
}
