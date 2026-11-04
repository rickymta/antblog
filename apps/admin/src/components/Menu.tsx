// Menu thả xuống nhỏ (thao tác của một dòng): Esc / bấm ra ngoài để đóng, mũi tên lên/xuống để di chuyển.
// Định vị `fixed` theo nút mở để không bị khung cuộn của bảng cắt mất.
import { useEffect, useRef, useState, type CSSProperties, type KeyboardEvent } from 'react';
import { Ellipsis, type LucideIcon } from 'lucide-react';
import { cx } from './ui';

export interface MenuItem {
  key: string;
  label: string;
  icon?: LucideIcon;
  onSelect?: () => void;
  /** Liên kết mở tab mới (vd "Xem trên trang"). */
  href?: string;
  danger?: boolean;
  disabled?: boolean;
}

export function Menu({ label, items, icon: Icon = Ellipsis }: { label: string; items: MenuItem[]; icon?: LucideIcon }) {
  const [open, setOpen] = useState(false);
  const [pos, setPos] = useState<CSSProperties>({});
  const wrap = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const list = useRef<HTMLUListElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDown = (e: MouseEvent) => {
      if (!wrap.current?.contains(e.target as Node)) setOpen(false);
    };
    const close = () => setOpen(false);
    document.addEventListener('mousedown', onDown);
    window.addEventListener('resize', close);
    window.addEventListener('scroll', close, true);
    list.current?.querySelector<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])')?.focus();
    return () => {
      document.removeEventListener('mousedown', onDown);
      window.removeEventListener('resize', close);
      window.removeEventListener('scroll', close, true);
    };
  }, [open]);

  const toggle = () => {
    if (open) return setOpen(false);
    const r = trigger.current?.getBoundingClientRect();
    if (r) {
      const estimated = items.length * 34 + 12;
      const up = r.bottom + estimated > window.innerHeight - 8 && r.top > estimated;
      setPos({
        position: 'fixed',
        right: Math.max(8, window.innerWidth - r.right),
        ...(up ? { bottom: window.innerHeight - r.top + 4 } : { top: r.bottom + 4 }),
      });
    }
    setOpen(true);
  };

  const onKey = (e: KeyboardEvent) => {
    const els = Array.from(list.current?.querySelectorAll<HTMLElement>('[role="menuitem"]:not([aria-disabled="true"])') ?? []);
    const idx = els.indexOf(document.activeElement as HTMLElement);
    if (e.key === 'Escape') {
      e.preventDefault();
      setOpen(false);
      trigger.current?.focus();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      els[(idx + 1) % els.length]?.focus();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      els[(idx - 1 + els.length) % els.length]?.focus();
    } else if (e.key === 'Tab') {
      setOpen(false);
    }
  };

  const itemClass = (it: MenuItem) =>
    cx(
      'flex w-full items-center gap-2 rounded-[6px] px-2.5 py-1.5 text-left text-sm hover:bg-surface-2 focus:bg-surface-2 focus:outline-none',
      it.danger ? 'text-red-700 dark:text-red-400' : 'text-ink',
      it.disabled && 'pointer-events-none opacity-50',
    );

  return (
    <div ref={wrap} className="relative inline-block">
      <button
        ref={trigger}
        type="button"
        aria-label={label}
        title={label}
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={toggle}
        className="inline-flex size-8 items-center justify-center rounded-ui text-muted hover:bg-surface-2 hover:text-ink"
      >
        <Icon className="size-4" aria-hidden />
      </button>
      {open && (
        <ul ref={list} role="menu" aria-label={label} onKeyDown={onKey} style={pos} className="z-50 min-w-[13rem] rounded-ui border border-line bg-surface p-1 shadow-lg">
          {items.map((it) => (
            <li key={it.key} role="none">
              {it.href ? (
                <a role="menuitem" href={it.href} target="_blank" rel="noreferrer" className={itemClass(it)} onClick={() => setOpen(false)}>
                  {it.icon && <it.icon className="size-4 shrink-0" aria-hidden />}
                  {it.label}
                </a>
              ) : (
                <button
                  type="button"
                  role="menuitem"
                  aria-disabled={it.disabled || undefined}
                  className={itemClass(it)}
                  onClick={() => {
                    if (it.disabled) return;
                    setOpen(false);
                    it.onSelect?.();
                  }}
                >
                  {it.icon && <it.icon className="size-4 shrink-0" aria-hidden />}
                  {it.label}
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
