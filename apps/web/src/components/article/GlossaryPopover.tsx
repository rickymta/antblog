'use client';

import { useCallback, useEffect, useId, useLayoutEffect, useRef, useState, type RefObject } from 'react';
import { createPortal } from 'react-dom';
import { useTranslation } from 'react-i18next';
import { X } from 'lucide-react';
import Link from 'next/link';
import { glossaryPath } from '../../lib/glossary';

/** Phải khớp TERM_CLASS của rehypeGlossary (@tkh/ui). */
const TERM_SELECTOR = 'dfn.tkh-term[data-term]';
const OPEN_DELAY = 250;
const CLOSE_DELAY = 150;
const GAP = 8;
const EDGE = 16;
const MAX_WIDTH = 360;

interface Active {
  el: HTMLElement;
  slug: string;
  title: string;
  english?: string;
  definition: string;
  /** Mở bằng chạm/nhấp/phím: giữ mở tới khi người dùng đóng (không đóng khi rời chuột). */
  pinned: boolean;
  /** Toạ độ y của con trỏ lúc mở — chọn đúng dòng khi keyword bị xuống dòng. */
  pointerY?: number;
  /** Mở bằng Enter/Space: chuyển focus vào hộp thoại để Tab tới được nút đóng và liên kết "Xem thêm". */
  fromKeyboard?: boolean;
}

interface Position {
  top: number;
  left: number;
  width: number;
  placement: 'top' | 'bottom';
}

function readTerm(el: HTMLElement, pinned: boolean, pointerY?: number): Active | null {
  const definition = el.dataset.termDefinition;
  if (!definition) return null;
  return { el, slug: el.dataset.term ?? '', title: el.dataset.termTitle || el.textContent || '', english: el.dataset.termEnglish, definition, pinned, pointerY };
}

/** Hộp chữ của keyword: nếu keyword xuống dòng, lấy dòng gần con trỏ nhất (hoặc dòng đầu). */
function anchorRect(el: HTMLElement, pointerY?: number): DOMRect {
  const rects = [...el.getClientRects()];
  if (!rects.length) return el.getBoundingClientRect();
  if (pointerY === undefined) return rects[0]!;
  return rects.reduce((best, r) => (Math.abs((r.top + r.bottom) / 2 - pointerY) < Math.abs((best.top + best.bottom) / 2 - pointerY) ? r : best));
}

function computePosition(anchor: DOMRect, height: number): Position {
  const vw = document.documentElement.clientWidth;
  const vh = window.innerHeight;
  const width = Math.min(MAX_WIDTH, vw - EDGE * 2);
  const spaceBelow = vh - anchor.bottom - GAP - EDGE;
  const spaceAbove = anchor.top - GAP - EDGE;
  const placement = spaceBelow >= height || spaceBelow >= spaceAbove ? 'bottom' : 'top';
  const top = placement === 'bottom' ? anchor.bottom + GAP : anchor.top - GAP - height;
  const centered = anchor.left + anchor.width / 2 - width / 2;
  const left = Math.min(Math.max(centered, EDGE), vw - width - EDGE);
  return { top, left, width, placement };
}

/**
 * Hộp thoại giải thích keyword của từ điển (giai đoạn 3 — docs/content-dictionary/02-glossary-feature.md).
 * Đọc dữ liệu từ thuộc tính `data-term-*` mà rehypeGlossary đặt trên `<dfn class="tkh-term">`, nghe sự kiện chung trên
 * `containerRef` và chỉ đặt/gỡ `aria-describedby`, `aria-expanded` trên keyword — không đổi nội dung DOM do React quản lý.
 * Máy tính: rê chuột (có trễ) hoặc focus bằng bàn phím. Cảm ứng: chạm để mở, chạm ra ngoài hoặc Esc để đóng.
 */
export function GlossaryPopover({ containerRef }: { containerRef: RefObject<HTMLElement | null> }) {
  const { t } = useTranslation();
  const id = useId();
  const [active, setActive] = useState<Active | null>(null);
  const [pos, setPos] = useState<Position | null>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const activeRef = useRef<Active | null>(null);
  const openTimer = useRef<number | undefined>(undefined);
  const closeTimer = useRef<number | undefined>(undefined);

  const clearTimers = () => {
    window.clearTimeout(openTimer.current);
    window.clearTimeout(closeTimer.current);
  };

  const open = useCallback((next: Active) => {
    clearTimers();
    setActive((prev) => (prev && prev.el === next.el && prev.pinned && !next.pinned ? prev : next));
  }, []);

  const close = useCallback((restoreFocus = false) => {
    clearTimers();
    const current = activeRef.current;
    setActive(null);
    setPos(null);
    if (restoreFocus) current?.el.focus();
  }, []);

  const scheduleClose = useCallback(() => {
    window.clearTimeout(openTimer.current);
    window.clearTimeout(closeTimer.current);
    if (activeRef.current?.pinned) return;
    closeTimer.current = window.setTimeout(() => close(), CLOSE_DELAY);
  }, [close]);

  // Gắn/gỡ thuộc tính ARIA trên keyword đang mở.
  useEffect(() => {
    const el = active?.el;
    if (!el) return;
    el.setAttribute('aria-describedby', id);
    el.setAttribute('aria-expanded', 'true');
    return () => {
      el.removeAttribute('aria-describedby');
      el.removeAttribute('aria-expanded');
    };
  }, [active, id]);

  const reposition = useCallback(() => {
    const current = activeRef.current;
    const pop = popRef.current;
    if (!current || !pop || !current.el.isConnected) return;
    setPos(computePosition(anchorRect(current.el, current.pointerY), pop.offsetHeight));
  }, []);

  // Cập nhật activeRef TRƯỚC khi đo: layout effect chạy trước mọi useEffect, nên không thể gán activeRef ở useEffect.
  useLayoutEffect(() => {
    activeRef.current = active;
    if (active) reposition();
  }, [active, reposition]);

  // Mở bằng bàn phím ⇒ đưa focus vào phần tử tương tác đầu tiên của hộp thoại (portal nằm cuối <body>, Tab không tới).
  useEffect(() => {
    if (active?.pinned && active.fromKeyboard) popRef.current?.querySelector<HTMLElement>('button, a[href]')?.focus();
  }, [active]);

  // Sự kiện trên thân bài.
  useEffect(() => {
    const root = containerRef.current;
    if (!root) return;
    const termOf = (target: EventTarget | null) => (target instanceof Element ? target.closest<HTMLElement>(TERM_SELECTOR) : null);

    const onPointerOver = (e: PointerEvent) => {
      if (e.pointerType === 'touch') return;
      const el = termOf(e.target);
      if (!el) return;
      window.clearTimeout(closeTimer.current);
      if (activeRef.current?.el === el) return;
      window.clearTimeout(openTimer.current);
      openTimer.current = window.setTimeout(() => {
        const next = readTerm(el, false, e.clientY);
        if (next) open(next);
      }, OPEN_DELAY);
    };
    const onPointerOut = (e: PointerEvent) => {
      if (e.pointerType === 'touch') return;
      const el = termOf(e.target);
      if (!el || el.contains(e.relatedTarget as Node | null)) return;
      if (popRef.current?.contains(e.relatedTarget as Node | null)) return;
      scheduleClose();
    };
    const onClick = (e: MouseEvent) => {
      const el = termOf(e.target);
      if (!el) return;
      e.preventDefault();
      if (activeRef.current?.el === el && activeRef.current.pinned) return close();
      const next = readTerm(el, true, e.clientY || undefined);
      if (next) open(next);
    };
    const onFocusIn = (e: FocusEvent) => {
      const el = termOf(e.target);
      if (!el || activeRef.current?.el === el) return;
      const next = readTerm(el, false);
      if (next) open(next);
    };
    const onFocusOut = (e: FocusEvent) => {
      const el = termOf(e.target);
      if (!el || popRef.current?.contains(e.relatedTarget as Node | null)) return;
      scheduleClose();
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ' ') return;
      const el = termOf(e.target);
      if (!el) return;
      e.preventDefault();
      if (activeRef.current?.el === el && activeRef.current.pinned) return close();
      const next = readTerm(el, true);
      if (next) open({ ...next, fromKeyboard: true });
    };

    root.addEventListener('pointerover', onPointerOver);
    root.addEventListener('pointerout', onPointerOut);
    root.addEventListener('click', onClick);
    root.addEventListener('focusin', onFocusIn);
    root.addEventListener('focusout', onFocusOut);
    root.addEventListener('keydown', onKeyDown);
    return () => {
      clearTimers();
      root.removeEventListener('pointerover', onPointerOver);
      root.removeEventListener('pointerout', onPointerOut);
      root.removeEventListener('click', onClick);
      root.removeEventListener('focusin', onFocusIn);
      root.removeEventListener('focusout', onFocusOut);
      root.removeEventListener('keydown', onKeyDown);
    };
  }, [containerRef, open, close, scheduleClose]);

  // Khi đang mở: Esc, chạm ra ngoài, cuộn/đổi cỡ.
  useEffect(() => {
    if (!active) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close(true);
    };
    const onPointerDown = (e: PointerEvent) => {
      const target = e.target as Node;
      if (popRef.current?.contains(target) || activeRef.current?.el.contains(target)) return;
      close();
    };
    let frame = 0;
    const onMove = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(reposition);
    };
    document.addEventListener('keydown', onKey);
    document.addEventListener('pointerdown', onPointerDown);
    window.addEventListener('scroll', onMove, { passive: true, capture: true });
    window.addEventListener('resize', onMove);
    return () => {
      cancelAnimationFrame(frame);
      document.removeEventListener('keydown', onKey);
      document.removeEventListener('pointerdown', onPointerDown);
      window.removeEventListener('scroll', onMove, { capture: true });
      window.removeEventListener('resize', onMove);
    };
  }, [active, close, reposition]);

  if (!active || typeof document === 'undefined') return null;

  const showEnglish = active.english && active.english.toLowerCase() !== active.title.toLowerCase();
  return createPortal(
    <div
      ref={popRef}
      id={id}
      // Đã ghim (có nút đóng, liên kết) ⇒ dialog không chặn trang; chỉ rê chuột/focus ⇒ tooltip mô tả cho keyword.
      role={active.pinned ? 'dialog' : 'tooltip'}
      aria-label={t('article.glossaryLabel')}
      onPointerEnter={() => window.clearTimeout(closeTimer.current)}
      onPointerLeave={(e) => {
        if (e.pointerType !== 'touch' && !activeRef.current?.el.contains(e.relatedTarget as Node | null)) scheduleClose();
      }}
      style={pos ? { top: pos.top, left: pos.left, width: pos.width } : { top: 0, left: 0, width: MAX_WIDTH, visibility: 'hidden' }}
      data-placement={pos?.placement}
      className="fixed z-50 rounded-xl border border-line bg-surface p-4 text-left shadow-lg motion-safe:animate-[tkh-pop_120ms_ease-out]"
    >
      <div className="flex items-start gap-3">
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold text-ink">{active.title}</p>
          {showEnglish && (
            <p className="mt-0.5 text-xs text-muted">
              {t('article.glossaryEnglish')}: <span lang="en">{active.english}</span>
            </p>
          )}
        </div>
        {active.pinned && (
          <button
            type="button"
            onClick={() => close(true)}
            aria-label={t('article.glossaryClose')}
            className="-m-1 rounded-md p-1 text-muted hover:bg-surface-2 hover:text-ink focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
          >
            <X aria-hidden="true" className="h-4 w-4" />
          </button>
        )}
      </div>
      <p className="mt-2 text-sm leading-relaxed text-ink">{active.definition}</p>
      <Link
        href={glossaryPath(active.slug)}
        // Next chuyển trang phía client: đóng hộp thoại để nó không treo lại trên trang mới.
        onClick={() => close()}
        className="mt-3 inline-flex items-center gap-1 text-sm font-semibold text-primary-ink hover:underline focus-visible:outline focus-visible:outline-2 focus-visible:outline-primary"
      >
        {t('article.glossaryMore')} <span aria-hidden="true">→</span>
      </Link>
    </div>,
    document.body,
  );
}
