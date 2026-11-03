import { Link } from 'react-router';
import { useTranslation } from 'react-i18next';
import { ChevronLeft, ChevronRight } from 'lucide-react';

/** Dãy số trang có dấu "…": luôn hiện trang đầu, cuối và hai trang quanh trang hiện tại. */
function pageItems(page: number, pageCount: number): (number | 'gap')[] {
  const pages = new Set<number>([1, pageCount, page - 1, page, page + 1]);
  const sorted = [...pages].filter((p) => p >= 1 && p <= pageCount).sort((a, b) => a - b);
  const out: (number | 'gap')[] = [];
  for (const p of sorted) {
    const prev = out[out.length - 1];
    if (typeof prev === 'number' && p - prev > 1) out.push('gap');
    out.push(p);
  }
  return out;
}

export function Pagination({
  page,
  total,
  pageSize,
  hrefFor,
}: {
  page: number;
  total: number;
  pageSize: number;
  hrefFor: (page: number) => string;
}) {
  const { t } = useTranslation();
  const pageCount = Math.max(1, Math.ceil(total / pageSize));
  if (pageCount <= 1) return null;

  const cell = 'inline-flex h-9 min-w-9 items-center justify-center rounded-ui px-2 text-sm font-medium';
  const arrow = `${cell} gap-1 border border-line bg-surface text-ink hover:bg-surface-2`;
  const disabled = `${cell} gap-1 border border-line bg-surface text-muted opacity-50`;

  return (
    <nav aria-label={t('articles.pagination')} className="flex flex-wrap items-center justify-center gap-1.5">
      {page > 1 ? (
        <Link to={hrefFor(page - 1)} className={arrow} aria-label={t('articles.prevPage')}>
          <ChevronLeft aria-hidden="true" className="h-4 w-4" />
        </Link>
      ) : (
        <span className={disabled} aria-hidden="true">
          <ChevronLeft className="h-4 w-4" />
        </span>
      )}
      {pageItems(page, pageCount).map((item, i) =>
        item === 'gap' ? (
          <span key={`gap-${i}`} className="px-1 text-muted" aria-hidden="true">
            …
          </span>
        ) : (
          <Link
            key={item}
            to={hrefFor(item)}
            aria-current={item === page ? 'page' : undefined}
            aria-label={t('articles.pageN', { n: item })}
            className={`${cell} ${item === page ? 'bg-primary text-white' : 'border border-line bg-surface text-ink hover:bg-surface-2'}`}
          >
            {item}
          </Link>
        ),
      )}
      {page < pageCount ? (
        <Link to={hrefFor(page + 1)} className={arrow} aria-label={t('articles.nextPage')}>
          <ChevronRight aria-hidden="true" className="h-4 w-4" />
        </Link>
      ) : (
        <span className={disabled} aria-hidden="true">
          <ChevronRight className="h-4 w-4" />
        </span>
      )}
    </nav>
  );
}
