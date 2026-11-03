import { useTranslation } from 'react-i18next';
import { ChevronDown, ListOrdered } from 'lucide-react';
import type { Heading } from '@tkh/ui';
import { scrollToId } from '../../lib/dom';

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
              className={`-ml-px block border-l-2 py-1.5 pr-2 text-sm leading-snug transition-colors ${h.depth === 3 ? 'pl-6' : 'pl-3'} ${
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
export function TocPanel({ headings, activeId, title }: { headings: Heading[]; activeId?: string; title?: string }) {
  const { t } = useTranslation();
  if (headings.length === 0) return null;
  return (
    <nav aria-labelledby="toc-title">
      <h2 id="toc-title" className="mb-3 text-xs font-bold tracking-wide text-ink uppercase">
        {title ?? t('article.toc')}
      </h2>
      <TocList headings={headings} activeId={activeId} />
    </nav>
  );
}

/** Mục lục thu gọn ở đầu bài trên màn hẹp (< 1024px). */
export function TocCollapsible({ headings, activeId }: { headings: Heading[]; activeId?: string }) {
  const { t } = useTranslation();
  if (headings.length === 0) return null;
  return (
    <details className="group rounded-card border border-line bg-surface lg:hidden">
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
