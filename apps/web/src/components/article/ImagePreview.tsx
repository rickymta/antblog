'use client';

import { useCallback, useEffect, useRef, useState, type RefObject } from 'react';
import { useTranslation } from 'react-i18next';
import { ExternalLink, X } from 'lucide-react';

interface Preview {
  src: string;
  alt: string;
  caption: string;
}

const SELECTOR = '.tkh-prose img';

/** Ảnh nằm trong liên kết thì giữ hành vi của liên kết, không mở hộp xem ảnh. */
function previewable(img: HTMLImageElement): boolean {
  return !img.closest('a');
}

/**
 * Bấm vào ảnh trong bài để xem lớn trong một hộp thoại (`<dialog>` gốc của trình duyệt: khoá focus, Esc để đóng).
 * Giống CodeCopyOverlay/GlossaryPopover: lắng nghe sự kiện trên `containerRef`, không render lại nội dung bài.
 * Ảnh được gắn thêm `tabindex`/`role` để mở được bằng bàn phím (Enter hoặc Space).
 * Trong hộp thoại, bấm vào ảnh để chuyển giữa "vừa khung" và "kích thước gốc" (cuộn được).
 */
export function ImagePreview({ containerRef, contentKey }: { containerRef: RefObject<HTMLElement | null>; contentKey: string }) {
  const { t } = useTranslation();
  const dialogRef = useRef<HTMLDialogElement>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [actualSize, setActualSize] = useState(false);

  const opener = useRef<HTMLImageElement | null>(null);

  const open = useCallback((img: HTMLImageElement) => {
    const caption = img.closest('figure')?.querySelector('figcaption')?.textContent?.trim() ?? '';
    opener.current = img;
    setActualSize(false);
    setPreview({ src: img.currentSrc || img.src, alt: img.alt, caption });
  }, []);

  useEffect(() => {
    const dlg = dialogRef.current;
    if (!preview || !dlg || dlg.open) return;
    dlg.showModal();
    dlg.querySelector<HTMLButtonElement>('button')?.focus();
  }, [preview]);

  useEffect(() => {
    const root = containerRef.current;
    if (!root) return;
    const images = [...root.querySelectorAll<HTMLImageElement>(SELECTOR)].filter(previewable);
    for (const img of images) {
      img.dataset.previewable = '';
      img.tabIndex = 0;
      img.setAttribute('role', 'button');
      img.setAttribute('aria-haspopup', 'dialog');
      img.setAttribute('aria-label', t('article.imagePreviewOpen', { alt: img.alt || '' }).trim());
    }
    const target = (e: Event) => {
      const el = e.target instanceof Element ? e.target.closest<HTMLImageElement>(SELECTOR) : null;
      return el && root.contains(el) && previewable(el) ? el : null;
    };
    const onClick = (e: MouseEvent) => {
      const img = target(e);
      if (img) open(img);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      const img = target(e);
      if (!img) return;
      e.preventDefault();
      open(img);
    };
    root.addEventListener('click', onClick);
    root.addEventListener('keydown', onKey);
    return () => {
      root.removeEventListener('click', onClick);
      root.removeEventListener('keydown', onKey);
      for (const img of images) {
        delete img.dataset.previewable;
        img.removeAttribute('tabindex');
        img.removeAttribute('role');
        img.removeAttribute('aria-haspopup');
        img.removeAttribute('aria-label');
      }
    };
  }, [containerRef, contentKey, open, t]);

  const close = () => dialogRef.current?.close();

  return (
    <dialog
      ref={dialogRef}
      className="tkh-image-preview"
      aria-label={preview?.alt || t('article.imagePreviewTitle')}
      onClose={() => {
        setPreview(null);
        // Trả focus về đúng ảnh đã mở, để người dùng bàn phím đọc tiếp từ chỗ cũ.
        opener.current?.focus({ preventScroll: true });
      }}
      // Bấm vào nền tối (chính phần tử dialog, ngoài khung nội dung) thì đóng.
      onClick={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      {preview && (
        <div className="tkh-image-preview-frame">
          <div className="tkh-image-preview-bar">
            <button type="button" onClick={close} className="tkh-image-preview-btn" aria-label={t('article.imagePreviewClose')}>
              <X className="h-4 w-4" aria-hidden />
            </button>
            <a href={preview.src} target="_blank" rel="noopener" className="tkh-image-preview-btn">
              <ExternalLink className="h-4 w-4" aria-hidden />
              <span>{t('article.imagePreviewOriginal')}</span>
            </a>
          </div>
          <div className={`tkh-image-preview-stage${actualSize ? ' is-actual' : ''}`}>
            {/* eslint-disable-next-line @next/next/no-img-element -- ảnh của bài, đã tối ưu sẵn (WebP) */}
            <img
              src={preview.src}
              alt={preview.alt}
              onClick={() => setActualSize((v) => !v)}
              title={actualSize ? t('article.imagePreviewFit') : t('article.imagePreviewActual')}
            />
          </div>
          {preview.caption && <p className="tkh-image-preview-caption">{preview.caption}</p>}
        </div>
      )}
    </dialog>
  );
}
