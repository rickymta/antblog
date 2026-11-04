'use client';

import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Check, Link as LinkIcon, Share2 } from 'lucide-react';
import { copyText } from '../../lib/dom';

/** Sao chép liên kết bài (không có tham số/hash) và báo "Đã sao chép" trong 2 giây. */
export function ShareButton({ variant = 'pill', label }: { variant?: 'pill' | 'block'; label?: string }) {
  const { t } = useTranslation();
  const [state, setState] = useState<'idle' | 'copied' | 'failed'>('idle');

  useEffect(() => {
    if (state === 'idle') return;
    const id = window.setTimeout(() => setState('idle'), 2000);
    return () => window.clearTimeout(id);
  }, [state]);

  const onClick = async () => {
    const url = `${window.location.origin}${window.location.pathname}`;
    setState((await copyText(url)) ? 'copied' : 'failed');
  };

  const text = state === 'copied' ? t('common.linkCopied') : state === 'failed' ? t('common.copyFailed') : (label ?? t('common.share'));
  const Icon = state === 'copied' ? Check : variant === 'block' ? LinkIcon : Share2;

  return (
    <button
      type="button"
      onClick={() => void onClick()}
      className={
        variant === 'block'
          ? 'inline-flex w-full items-center justify-center gap-2 rounded-ui border border-line bg-surface px-3 py-2 text-sm font-semibold text-ink hover:bg-surface-2'
          : 'inline-flex items-center gap-1.5 rounded-ui border border-line px-3 py-1.5 text-sm font-medium text-muted hover:bg-surface-2 hover:text-ink'
      }
    >
      <Icon aria-hidden="true" className={`h-4 w-4 ${state === 'copied' ? 'text-emerald-600 dark:text-emerald-400' : ''}`} />
      <span aria-live="polite">{text}</span>
    </button>
  );
}
