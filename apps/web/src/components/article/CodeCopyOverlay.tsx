'use client';

import { useCallback, useEffect, useLayoutEffect, useState, type RefObject } from 'react';
import { useTranslation } from 'react-i18next';
import { Check, Copy } from 'lucide-react';
import { copyText } from '../../lib/dom';

interface Block {
  el: HTMLPreElement;
  top: number;
  right: number;
  lang: string;
}

function languageOf(pre: HTMLPreElement): string {
  const code = pre.querySelector('code');
  const cls = [...(code?.classList ?? [])].find((c) => c.startsWith('language-'));
  return cls ? cls.slice('language-'.length) : '';
}

/**
 * Nút "Sao chép" cho mọi khối mã trong bài — KHÔNG sửa DOM do React/ArticleContent quản lý:
 * nút nằm ở một lớp phủ riêng (con của `containerRef`, định vị tuyệt đối theo vị trí từng `<pre>`),
 * đo lại khi kích thước vùng nội dung đổi (ảnh tải xong, đổi cỡ màn hình, đổi font).
 * `containerRef` phải `position: relative`; `contentKey` đổi khi nội dung bài đổi.
 */
export function CodeCopyOverlay({ containerRef, contentKey }: { containerRef: RefObject<HTMLElement | null>; contentKey: string }) {
  const { t } = useTranslation();
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [copied, setCopied] = useState<number | null>(null);

  const measure = useCallback(() => {
    const root = containerRef.current;
    if (!root) return;
    const rootRect = root.getBoundingClientRect();
    const next = [...root.querySelectorAll<HTMLPreElement>('.tkh-prose pre')].map((el) => {
      const r = el.getBoundingClientRect();
      return { el, top: r.top - rootRect.top, right: rootRect.right - r.right, lang: languageOf(el) };
    });
    setBlocks((prev) =>
      prev.length === next.length && prev.every((b, i) => b.el === next[i]!.el && b.top === next[i]!.top && b.right === next[i]!.right)
        ? prev
        : next,
    );
  }, [containerRef]);

  useLayoutEffect(() => {
    measure();
  }, [measure, contentKey]);

  useEffect(() => {
    const root = containerRef.current;
    if (!root || typeof ResizeObserver === 'undefined') return;
    const ro = new ResizeObserver(() => measure());
    ro.observe(root);
    return () => ro.disconnect();
  }, [containerRef, measure, contentKey]);

  useEffect(() => {
    if (copied === null) return;
    const id = window.setTimeout(() => setCopied(null), 1800);
    return () => window.clearTimeout(id);
  }, [copied]);

  if (blocks.length === 0) return null;

  return (
    <div className="pointer-events-none absolute inset-0">
      {blocks.map((b, i) => {
        const done = copied === i;
        return (
          <div key={i} className="absolute flex items-center gap-2" style={{ top: b.top + 8, right: b.right + 8 }}>
            {b.lang && <span className="font-mono text-[11px] tracking-wide text-slate-400 uppercase select-none">{b.lang}</span>}
            <button
              type="button"
              aria-label={t('article.copyCodeLabel', { lang: b.lang || '' }).replace(/\s+/g, ' ').trim()}
              onClick={async () => {
                const ok = await copyText(b.el.querySelector('code')?.textContent ?? b.el.textContent ?? '');
                if (ok) setCopied(i);
              }}
              className="pointer-events-auto inline-flex items-center gap-1 rounded-md border border-slate-600 bg-slate-800/90 px-2 py-1 text-xs font-medium text-slate-100 hover:bg-slate-700 focus-visible:outline-sky-300"
            >
              {done ? <Check aria-hidden="true" className="h-3.5 w-3.5 text-emerald-400" /> : <Copy aria-hidden="true" className="h-3.5 w-3.5" />}
              <span aria-live="polite">{done ? t('article.codeCopied') : t('article.copyCode')}</span>
            </button>
          </div>
        );
      })}
    </div>
  );
}
