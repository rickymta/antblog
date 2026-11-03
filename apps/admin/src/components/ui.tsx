// Thành phần giao diện riêng của trang quản trị (nút, ô nhập, thẻ, trạng thái tải/rỗng/lỗi, phân trang, hộp thoại).
import {
  useEffect,
  useId,
  useRef,
  type ButtonHTMLAttributes,
  type ReactNode,
} from 'react';
import { ChevronLeft, ChevronRight, CircleAlert, Inbox, LoaderCircle, RefreshCw, type LucideIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

type IconType = LucideIcon;

export function cx(...parts: (string | false | null | undefined)[]): string {
  return parts.filter(Boolean).join(' ');
}

// ---------- Nút ----------

type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
type ButtonSize = 'sm' | 'md';

const VARIANT: Record<ButtonVariant, string> = {
  primary: 'bg-primary text-white hover:bg-primary-hover disabled:hover:bg-primary',
  secondary: 'border border-line bg-surface text-ink hover:bg-surface-2',
  ghost: 'text-ink hover:bg-surface-2',
  danger: 'bg-red-600 text-white hover:bg-red-700 disabled:hover:bg-red-600',
  success: 'bg-emerald-700 text-white hover:bg-emerald-800 disabled:hover:bg-emerald-700',
};

/** Lớp kiểu nút — dùng cho <Link> trông như nút (không lồng <button> trong <a>). */
export function buttonClass(variant: ButtonVariant = 'secondary', size: ButtonSize = 'md', className?: string): string {
  return cx(
    'inline-flex items-center justify-center gap-2 rounded-ui font-medium whitespace-nowrap transition-colors disabled:cursor-not-allowed disabled:opacity-55',
    size === 'sm' ? 'h-8 px-3 text-sm' : 'h-10 px-4 text-sm',
    VARIANT[variant],
    className,
  );
}

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  icon?: IconType;
  loading?: boolean;
}

export function Button({
  variant = 'secondary',
  size = 'md',
  icon: Icon,
  loading,
  disabled,
  className,
  children,
  type = 'button',
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      className={buttonClass(variant, size, className)}
      {...rest}
    >
      {loading ? (
        <LoaderCircle className="size-4 animate-spin" aria-hidden />
      ) : Icon ? (
        <Icon className="size-4" aria-hidden />
      ) : null}
      {children}
    </button>
  );
}

export function IconButton({
  icon: Icon,
  label,
  className,
  type = 'button',
  ...rest
}: Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> & { icon: IconType; label: string }) {
  return (
    <button
      type={type}
      aria-label={label}
      title={label}
      className={cx(
        'inline-flex size-9 items-center justify-center rounded-ui text-muted transition-colors hover:bg-surface-2 hover:text-ink disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-transparent',
        className,
      )}
      {...rest}
    >
      <Icon className="size-4.5" aria-hidden />
    </button>
  );
}

// ---------- Ô nhập ----------

export const inputClass =
  'block w-full rounded-ui border border-line bg-surface px-3 py-2 text-sm text-ink placeholder:text-muted focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/30 disabled:opacity-60 aria-[invalid=true]:border-red-500';

export const selectClass = cx(inputClass, 'h-10 pr-8');

/** Nhãn + ô nhập + gợi ý + lỗi. `children` nhận id và các thuộc tính aria để gắn vào ô. */
export function Field({
  label,
  error,
  hint,
  required,
  aside,
  children,
}: {
  label: string;
  error?: string;
  hint?: ReactNode;
  required?: boolean;
  aside?: ReactNode;
  children: (a: { id: string; 'aria-invalid': boolean; 'aria-describedby'?: string }) => ReactNode;
}) {
  const id = useId();
  const hintId = `${id}-hint`;
  const errId = `${id}-err`;
  const describedBy = [error ? errId : null, hint ? hintId : null].filter(Boolean).join(' ') || undefined;
  return (
    <div className="space-y-1.5">
      <div className="flex items-baseline justify-between gap-2">
        <label htmlFor={id} className="text-sm font-medium text-ink">
          {label}
          {required && (
            <span className="ml-0.5 text-red-600 dark:text-red-400" aria-hidden>
              *
            </span>
          )}
        </label>
        {aside}
      </div>
      {children({ id, 'aria-invalid': Boolean(error), 'aria-describedby': describedBy })}
      {error && (
        <p id={errId} className="flex items-center gap-1 text-xs font-medium text-red-700 dark:text-red-400">
          <CircleAlert className="size-3.5 shrink-0" aria-hidden />
          {error}
        </p>
      )}
      {hint && (
        <p id={hintId} className="text-xs text-muted">
          {hint}
        </p>
      )}
    </div>
  );
}

// ---------- Khung trang ----------

export function PageHeader({ title, subtitle, actions }: { title: string; subtitle?: string; actions?: ReactNode }) {
  return (
    <div className="mb-6 flex flex-wrap items-end justify-between gap-4">
      <div className="min-w-0">
        <h1 className="text-2xl font-bold tracking-tight text-ink">{title}</h1>
        {subtitle && <p className="mt-1 text-sm text-muted">{subtitle}</p>}
      </div>
      {actions && <div className="flex flex-wrap items-center gap-2">{actions}</div>}
    </div>
  );
}

export function Card({ className, children }: { className?: string; children: ReactNode }) {
  return <section className={cx('rounded-card border border-line bg-surface', className)}>{children}</section>;
}

export function FilterBar({ children }: { children: ReactNode }) {
  return <div className="flex flex-wrap items-end gap-3 border-b border-line p-4">{children}</div>;
}

// ---------- Nhãn ----------

export type Tone = 'neutral' | 'primary' | 'success' | 'warning' | 'danger' | 'indigo' | 'violet';

const TONE: Record<Tone, string> = {
  neutral: 'bg-slate-100 text-slate-700 dark:bg-slate-700/60 dark:text-slate-200',
  primary: 'bg-blue-100 text-blue-800 dark:bg-blue-500/20 dark:text-blue-200',
  success: 'bg-emerald-100 text-emerald-800 dark:bg-emerald-500/20 dark:text-emerald-200',
  warning: 'bg-amber-100 text-amber-900 dark:bg-amber-500/20 dark:text-amber-200',
  danger: 'bg-red-100 text-red-800 dark:bg-red-500/20 dark:text-red-200',
  indigo: 'bg-indigo-100 text-indigo-800 dark:bg-indigo-500/20 dark:text-indigo-200',
  violet: 'bg-violet-100 text-violet-800 dark:bg-violet-500/20 dark:text-violet-200',
};

export function Badge({ tone = 'neutral', children }: { tone?: Tone; children: ReactNode }) {
  return (
    <span className={cx('inline-flex items-center rounded-ui px-2 py-0.5 text-xs font-semibold whitespace-nowrap', TONE[tone])}>
      {children}
    </span>
  );
}

// ---------- Trạng thái tải / rỗng / lỗi ----------

export function Skeleton({ className }: { className?: string }) {
  return <div className={cx('animate-pulse rounded-ui bg-surface-2', className)} aria-hidden />;
}

export function TableSkeleton({ rows = 6, cols = 5 }: { rows?: number; cols?: number }) {
  const { t } = useTranslation();
  return (
    <div className="space-y-3 p-4" role="status" aria-label={t('common.loading')}>
      {Array.from({ length: rows }, (_, r) => (
        <div key={r} className="flex gap-4">
          {Array.from({ length: cols }, (_, c) => (
            <Skeleton key={c} className={cx('h-5', c === 0 ? 'flex-[3]' : 'flex-1')} />
          ))}
        </div>
      ))}
    </div>
  );
}

export function EmptyState({ title, body, action }: { title: string; body?: string; action?: ReactNode }) {
  return (
    <div className="flex flex-col items-center gap-2 px-4 py-12 text-center">
      <Inbox className="size-10 text-muted" aria-hidden />
      <p className="font-semibold text-ink">{title}</p>
      {body && <p className="max-w-md text-sm text-muted">{body}</p>}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}

export function ErrorState({ message, onRetry, title }: { message: string; onRetry?: () => void; title?: string }) {
  const { t } = useTranslation();
  return (
    <div role="alert" className="flex flex-col items-center gap-2 px-4 py-12 text-center">
      <CircleAlert className="size-10 text-red-600 dark:text-red-400" aria-hidden />
      <p className="font-semibold text-ink">{title ?? t('states.errorTitle')}</p>
      <p className="max-w-md text-sm text-muted">{message}</p>
      {onRetry && (
        <Button icon={RefreshCw} onClick={onRetry} className="mt-2">
          {t('common.retry')}
        </Button>
      )}
    </div>
  );
}

// ---------- Phân trang ----------

export function Pagination({
  page,
  pageSize,
  total,
  onPage,
}: {
  page: number;
  pageSize: number;
  total: number;
  onPage: (p: number) => void;
}) {
  const { t } = useTranslation();
  const pages = Math.max(1, Math.ceil(total / Math.max(1, pageSize)));
  return (
    <nav aria-label={t('common.pagination')} className="flex flex-wrap items-center justify-between gap-3 border-t border-line px-4 py-3">
      <p className="text-sm text-muted">{t('common.pageInfo', { page, pages, total })}</p>
      <div className="flex gap-1">
        <IconButton icon={ChevronLeft} label={t('common.prev')} disabled={page <= 1} onClick={() => onPage(page - 1)} />
        <IconButton icon={ChevronRight} label={t('common.next')} disabled={page >= pages} onClick={() => onPage(page + 1)} />
      </div>
    </nav>
  );
}

// ---------- Hộp thoại ----------

/** Hộp thoại dùng <dialog> gốc: khoá focus bên trong, Esc để đóng. */
export function Modal({
  open,
  onClose,
  title,
  children,
  footer,
  size = 'md',
}: {
  open: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  size?: 'sm' | 'md' | 'lg';
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  useEffect(() => {
    const d = ref.current;
    if (!d) return;
    if (open && !d.open) d.showModal();
    if (!open && d.open) d.close();
  }, [open]);
  return (
    <dialog
      ref={ref}
      aria-labelledby={titleId}
      onCancel={(e) => {
        e.preventDefault();
        onClose();
      }}
      className={cx(
        'm-auto w-[calc(100vw-2rem)] rounded-card border border-line bg-surface p-0 text-ink shadow-2xl backdrop:bg-slate-950/55',
        size === 'sm' ? 'max-w-md' : size === 'lg' ? 'max-w-3xl' : 'max-w-xl',
      )}
    >
      {open && (
        <div className="flex max-h-[calc(100dvh-2rem)] flex-col">
          <h2 id={titleId} className="border-b border-line px-5 py-4 text-lg font-semibold">
            {title}
          </h2>
          <div className="overflow-y-auto px-5 py-4">{children}</div>
          {footer && <div className="flex flex-wrap justify-end gap-2 border-t border-line px-5 py-3">{footer}</div>}
        </div>
      )}
    </dialog>
  );
}

export function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel,
  tone = 'danger',
  loading,
  error,
  onConfirm,
  onCancel,
  cancelLabel,
}: {
  open: boolean;
  title: string;
  body: ReactNode;
  confirmLabel: string;
  tone?: 'danger' | 'primary';
  loading?: boolean;
  error?: string;
  onConfirm: () => void;
  onCancel: () => void;
  cancelLabel?: string;
}) {
  const { t } = useTranslation();
  return (
    <Modal
      open={open}
      onClose={onCancel}
      title={title}
      size="sm"
      footer={
        <>
          <Button onClick={onCancel} autoFocus>
            {cancelLabel ?? t('common.cancel')}
          </Button>
          <Button variant={tone} loading={loading} onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <div className="space-y-3 text-sm text-muted">
        <div>{body}</div>
        {error && (
          <p role="alert" className="rounded-ui bg-red-50 p-3 font-medium text-red-800 dark:bg-red-950/50 dark:text-red-200">
            {error}
          </p>
        )}
      </div>
    </Modal>
  );
}

// ---------- Bảng ----------

export function TableWrap({ children, label }: { children: ReactNode; label: string }) {
  return (
    <div className="overflow-x-auto" role="region" aria-label={label} tabIndex={0}>
      <table className="w-full min-w-[640px] border-collapse text-left text-sm">{children}</table>
    </div>
  );
}

export function Th({ children, className }: { children?: ReactNode; className?: string }) {
  return (
    <th scope="col" className={cx('border-b border-line bg-surface-2 px-4 py-2.5 text-xs font-semibold tracking-wide text-slate-600 uppercase dark:text-slate-300', className)}>
      {children}
    </th>
  );
}

export function Td({ children, className }: { children?: ReactNode; className?: string }) {
  return <td className={cx('border-b border-line px-4 py-3 align-middle text-ink', className)}>{children}</td>;
}

export function Spinner({ label }: { label: string }) {
  return (
    <div role="status" className="flex items-center justify-center gap-2 py-16 text-sm text-muted">
      <LoaderCircle className="size-5 animate-spin" aria-hidden />
      {label}
    </div>
  );
}
