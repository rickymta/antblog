import { useEffect, useId, useRef, useState, type FormEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { Avatar } from '@tkh/ui';
import type { User } from '@tkh/shared';
import { apiMessage } from '../../lib/errors';
import { btnPrimary, btnSecondary } from '../../lib/ui';

export const MAX_COMMENT = 2000;

/**
 * Ô soạn bình luận (A5): avatar + textarea + nút primary mờ khi trống. Ctrl/Cmd + Enter để gửi.
 * Dùng cho bình luận gốc, phản hồi và sửa bình luận. Chữ gửi đi là văn bản thuần.
 */
export function CommentComposer({
  user,
  onSubmit,
  placeholder,
  submitLabel,
  initialValue = '',
  autoFocus = false,
  onCancel,
  showAvatar = true,
  label,
}: {
  user: User;
  onSubmit: (body: string) => Promise<void>;
  placeholder: string;
  submitLabel: string;
  initialValue?: string;
  autoFocus?: boolean;
  onCancel?: () => void;
  showAvatar?: boolean;
  label?: string;
}) {
  const { t } = useTranslation();
  const [value, setValue] = useState(initialValue);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<HTMLTextAreaElement>(null);
  const id = useId();
  const errId = useId();

  useEffect(() => {
    if (!autoFocus || !ref.current) return;
    const el = ref.current;
    el.focus({ preventScroll: true });
    el.scrollIntoView({ block: 'nearest' });
    el.setSelectionRange(el.value.length, el.value.length);
  }, [autoFocus]);

  // Textarea tự giãn theo nội dung (tối đa ~12 dòng, sau đó cuộn).
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = 'auto';
    el.style.height = `${Math.min(el.scrollHeight, 320)}px`;
  }, [value]);

  const trimmed = value.trim();
  const tooLong = trimmed.length > MAX_COMMENT;
  const canSend = trimmed.length > 0 && !tooLong && !sending && trimmed !== initialValue.trim();

  const submit = async (e?: FormEvent) => {
    e?.preventDefault();
    if (!canSend) return;
    setSending(true);
    setError(null);
    try {
      await onSubmit(trimmed);
      setValue('');
    } catch (err) {
      setError(apiMessage(err) ?? t('comments.error'));
    } finally {
      setSending(false);
    }
  };

  return (
    <form onSubmit={(e) => void submit(e)} className="flex items-start gap-3">
      {showAvatar && <Avatar name={user.displayName} src={user.avatarUrl} size={36} className="mt-0.5" />}
      <div className="min-w-0 flex-1">
        <label htmlFor={id} className="sr-only">
          {label ?? t('comments.inputLabel')}
        </label>
        <textarea
          ref={ref}
          id={id}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) void submit();
            if (e.key === 'Escape' && onCancel) onCancel();
          }}
          rows={3}
          maxLength={MAX_COMMENT + 200}
          placeholder={placeholder}
          aria-invalid={tooLong || Boolean(error)}
          aria-describedby={error || tooLong ? errId : undefined}
          className="block w-full resize-none rounded-card border border-line bg-surface px-3 py-2.5 text-sm text-ink placeholder:text-muted focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/30"
        />
        <div className="mt-2 flex flex-wrap items-center justify-between gap-2">
          <span className={`text-xs ${tooLong ? 'font-semibold text-red-700 dark:text-red-400' : 'text-muted'}`}>
            {trimmed.length > MAX_COMMENT * 0.8 ? t('comments.charCount', { n: trimmed.length, max: MAX_COMMENT }) : ''}
          </span>
          <div className="flex items-center gap-2">
            {onCancel && (
              <button type="button" onClick={onCancel} className={`${btnSecondary} py-1.5`}>
                {t('common.cancel')}
              </button>
            )}
            <button type="submit" disabled={!canSend} className={`${btnPrimary} py-1.5`}>
              {sending ? t('comments.sending') : submitLabel}
            </button>
          </div>
        </div>
        {(error || tooLong) && (
          <p id={errId} role="alert" className="mt-1 text-sm text-red-700 dark:text-red-400">
            {tooLong ? t('comments.tooLong', { max: MAX_COMMENT }) : error}
          </p>
        )}
      </div>
    </form>
  );
}
