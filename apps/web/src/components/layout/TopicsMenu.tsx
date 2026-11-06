'use client';

import { useCallback, useId, useRef, useState } from 'react';
import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { ChevronDown } from 'lucide-react';
import { CategoryGlyph } from '@tkh/ui';
import { categoryTree, formatNumber, pickLocalized, type Category, type CategoryTreeNode } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { useDismiss } from '../../hooks/useDismiss';
import { safeColor, withAlpha } from '../../lib/format';

/** Danh sách chủ đề (dùng ở menu thả, ngăn kéo điện thoại). */
export function TopicLinks({ categories, onNavigate, dense = false }: { categories: Category[]; onNavigate?: () => void; dense?: boolean }) {
  const { t } = useTranslation();
  const locale = useLocale();
  function render(nodes: CategoryTreeNode[], depth = 1) {
    return <ul className={depth > 1 ? 'ml-5 border-l border-line pl-2' : dense ? 'space-y-0.5' : 'space-y-1'}>
      {nodes.map(c => {
        const color = safeColor(c.color);
        const link = <Link href={'/topics/' + encodeURIComponent(c.slug)} onClick={onNavigate} className="flex min-w-0 flex-1 items-center gap-2 rounded-ui px-2 py-2 text-sm text-ink hover:bg-surface-2"><span className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-ui" style={{backgroundColor:withAlpha(color,0.14)}}><CategoryGlyph icon={c.icon} className="h-4 w-4" style={{color}} /></span><span className="min-w-0 flex-1 [overflow-wrap:anywhere] font-medium">{pickLocalized(c.name,locale)}</span><span className="shrink-0 text-xs text-muted" title={t('common.articleCount',{count:c.articleCount,n:formatNumber(c.articleCount,locale)})}>{formatNumber(c.articleCount,locale)}</span></Link>;
        return <li key={c.id}>{c.children.length ? <details open={depth === 1} className="topic-branch"><summary className="flex cursor-pointer list-none items-center gap-1"><span className="min-w-0 flex-1">{link}</span><span className="topic-toggle inline-flex h-8 w-8 items-center justify-center rounded-ui text-muted hover:bg-surface-2"><ChevronDown size={16} aria-hidden="true" /><span className="sr-only">{t('nav.topics')}: {pickLocalized(c.name,locale)}</span></span></summary>{render(c.children,depth+1)}</details> : link}</li>;
      })}
    </ul>;
  }
  return render(categoryTree(categories));
}

/** Mục "Chủ đề ▾" trên thanh đầu (màn rộng). Danh sách chủ đề do máy chủ tải sẵn (layout gốc). */
export function TopicsMenu({ categories }: { categories: Category[] | null }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const wrapRef = useRef<HTMLDivElement>(null);
  const btnRef = useRef<HTMLButtonElement>(null);
  const panelId = useId();
  const close = useCallback(() => setOpen(false), []);
  useDismiss(wrapRef, open, close, btnRef);

  return (
    <div ref={wrapRef} className="relative">
      <button
        ref={btnRef}
        type="button"
        aria-expanded={open}
        aria-controls={panelId}
        onClick={() => setOpen((v) => !v)}
        className="inline-flex items-center gap-1 rounded-ui px-3 py-2 text-sm font-medium whitespace-nowrap text-muted hover:bg-surface-2 hover:text-ink"
      >
        {t('nav.topics')}
        <ChevronDown aria-hidden="true" className={`h-4 w-4 transition-transform ${open ? 'rotate-180' : ''}`} />
      </button>
      {open && (
        <div id={panelId} className="absolute left-0 top-full z-50 mt-2 max-h-[min(70vh,650px)] w-96 max-w-[90vw] overflow-y-auto rounded-card border border-line bg-surface p-2 shadow-xl shadow-slate-900/10">
          {categories ? <TopicLinks categories={categories} onNavigate={close} /> : <p className="p-3 text-sm text-muted">{t('nav.topicsLoadFailed')}</p>}
          <div className="mt-1 border-t border-line pt-1">
            <Link href="/articles" onClick={close} className="block rounded-ui px-2 py-2 text-sm font-semibold text-primary-ink hover:bg-surface-2">
              {t('nav.allTopics')} →
            </Link>
          </div>
        </div>
      )}
    </div>
  );
}
