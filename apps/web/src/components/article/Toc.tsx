'use client';

import { useId, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronDown, ListOrdered } from 'lucide-react';
import type { Heading } from '@tkh/ui';
import { useScrollSpy } from '../../hooks/useScrollSpy';
import { scrollToId } from '../../lib/dom';

/** Tiêu đề đang đọc (scroll-spy) — mỗi khối mục lục tự theo dõi, danh sách tiêu đề do máy chủ tính sẵn. */
function useActiveHeading(headings: Heading[]): string | undefined {
  const ids = useMemo(() => headings.map((h) => h.id), [headings]);
  return useScrollSpy(ids);
}

/** Danh sách mục lục H2/H3; mục đang đọc tô primary (scroll-spy, A4). */
export function TocList({ headings, activeId, onNavigate }: { headings: Heading[]; activeId?: string; onNavigate?: () => void }) {
  return (
    <ul className="space-y-0.5 border-l border-line">
      {headings.map((h) => {
        const active = h.id === activeId;
        return (
          <li key={h.id}>
            <a
              href={`#${h.id}`}
              aria-current={active ? 'location' : undefined}
              onClick={(e) => {
                e.preventDefault();
                scrollToId(h.id);
                onNavigate?.();
              }}
              className={`-ml-px block [overflow-wrap:anywhere] border-l-2 py-1.5 pr-2 text-sm leading-snug transition-colors ${h.depth === 3 ? 'pl-6' : 'pl-3'} ${
                active ? 'border-primary font-semibold text-primary-ink' : 'border-transparent text-muted hover:border-line hover:text-ink'
              }`}
            >
              {h.text}
            </a>
          </li>
        );
      })}
    </ul>
  );
}

/** "Trên trang này" ở cột bên (màn rộng). */
export function TocPanel({ headings, title }: { headings: Heading[]; title?: string }) {
  const { t } = useTranslation();
  const activeId = useActiveHeading(headings);
  const titleId = useId();
  if (headings.length === 0) return null;
  return (
    <nav aria-labelledby={titleId}>
      <h2 id={titleId} className="mb-3 text-xs font-bold tracking-wide text-ink uppercase">
        {title ?? t('article.toc')}
      </h2>
      <TocList headings={headings} activeId={activeId} />
    </nav>
  );
}

/** Mục lục thu gọn ở đầu bài trên màn hẹp (< 1280px). */
export function TocCollapsible({ headings }: { headings: Heading[] }) {
  const { t } = useTranslation();
  const activeId = useActiveHeading(headings);
  if (headings.length === 0) return null;
  return (
    <details className="group rounded-card border border-line bg-surface min-[1280px]:hidden">
      <summary className="flex cursor-pointer list-none items-center gap-2 px-4 py-3 text-sm font-semibold text-ink [&::-webkit-details-marker]:hidden">
        <ListOrdered aria-hidden="true" className="h-4 w-4 text-muted" />
        {t('article.tocMobile')}
        <ChevronDown aria-hidden="true" className="ml-auto h-4 w-4 text-muted transition-transform group-open:rotate-180" />
      </summary>
      <nav aria-label={t('article.tocMobile')} className="px-4 pb-4">
        <TocList headings={headings} activeId={activeId} />
      </nav>
    </details>
  );
}
