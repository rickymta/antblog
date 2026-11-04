'use client';

import { useId, useState, type InputHTMLAttributes, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Eye, EyeOff } from 'lucide-react';
import { inputClass } from '../../lib/ui';

type InputProps = Omit<InputHTMLAttributes<HTMLInputElement>, 'id' | 'className'>;

/** Ô nhập có nhãn, gợi ý và lỗi hiện ngay dưới ô (gắn aria-describedby / aria-invalid). */
export function TextField({ label, hint, error, trailing, ...input }: InputProps & { label: string; hint?: string; error?: string; trailing?: ReactNode }) {
  const id = useId();
  const hintId = `${id}-hint`;
  const errId = `${id}-err`;
  const describedBy = [hint ? hintId : null, error ? errId : null].filter(Boolean).join(' ') || undefined;
  return (
    <div>
      <label htmlFor={id} className="mb-1.5 block text-sm font-semibold text-ink">
        {label}
      </label>
      <div className="relative">
        <input id={id} aria-invalid={Boolean(error)} aria-describedby={describedBy} className={`${inputClass} py-2.5 ${trailing ? 'pr-11' : ''}`} {...input} />
        {trailing && <div className="absolute inset-y-0 right-1 flex items-center">{trailing}</div>}
      </div>
      {hint && !error && (
        <p id={hintId} className="mt-1 text-xs text-muted">
          {hint}
        </p>
      )}
      {error && (
        <p id={errId} className="mt-1 text-sm font-medium text-red-700 dark:text-red-400">
          {error}
        </p>
      )}
    </div>
  );
}

export function PasswordField(props: InputProps & { label: string; hint?: string; error?: string }) {
  const { t } = useTranslation();
  const [visible, setVisible] = useState(false);
  return (
    <TextField
      {...props}
      type={visible ? 'text' : 'password'}
      trailing={
        <button
          type="button"
          onClick={() => setVisible((v) => !v)}
          aria-label={visible ? t('auth.hidePassword') : t('auth.showPassword')}
          aria-pressed={visible}
          className="inline-flex h-8 w-8 items-center justify-center rounded-ui text-muted hover:bg-surface-2 hover:text-ink"
        >
          {visible ? <EyeOff aria-hidden="true" className="h-4 w-4" /> : <Eye aria-hidden="true" className="h-4 w-4" />}
        </button>
      }
    />
  );
}
