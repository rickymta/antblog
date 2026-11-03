import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { ArrowRight } from 'lucide-react';
import { btnLink } from '../../lib/ui';

export function PageHeader({ title, subtitle, eyebrow, children }: { title: string; subtitle?: string; eyebrow?: ReactNode; children?: ReactNode }) {
  return (
    <header className="space-y-3 py-8 sm:py-10">
      {eyebrow}
      <h1 className="text-3xl font-extrabold tracking-tight text-ink sm:text-4xl">{title}</h1>
      {subtitle && <p className="max-w-3xl text-base text-muted sm:text-lg">{subtitle}</p>}
      {children}
    </header>
  );
}

/** Tiêu đề khối trên trang chủ: tiêu đề + phụ đề trái, liên kết "Xem tất cả →" phải. */
export function SectionHeader({
  id,
  title,
  subtitle,
  icon,
  link,
}: {
  id: string;
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  link?: { to: string; label: string };
}) {
  return (
    <div className="mb-6 flex flex-wrap items-end justify-between gap-3">
      <div>
        <h2 id={id} className="flex items-center gap-2 text-2xl font-bold tracking-tight text-ink">
          {icon}
          {title}
        </h2>
        {subtitle && <p className="mt-1 text-sm text-muted">{subtitle}</p>}
      </div>
      {link && (
        <Link to={link.to} className={btnLink}>
          {link.label}
          <ArrowRight aria-hidden="true" className="h-4 w-4" />
        </Link>
      )}
    </div>
  );
}
