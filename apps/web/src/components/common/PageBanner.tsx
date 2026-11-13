import { BookOpen, Compass, Route } from 'lucide-react';

const icons = { articles: BookOpen, paths: Route, about: Compass };

/** A theme-aware heading keeps the content and filters close to the top. */
export function PageBanner({ variant, title, subtitle, compact = false }: {
  variant: keyof typeof icons;
  title: string;
  subtitle: string;
  compact?: boolean;
}) {
  const Icon = icons[variant];
  return (
    <header className={`flex items-start gap-4 border-b border-line ${compact ? 'mb-6 py-6' : 'mb-8 py-8'}`}>
      <span className="hidden h-11 w-11 shrink-0 items-center justify-center rounded-ui bg-primary-soft text-primary-ink sm:inline-flex"><Icon aria-hidden="true" className="h-5 w-5" /></span>
      <div>
        <h1 className="text-3xl font-bold tracking-tight text-ink">{title}</h1>
        <p className="mt-2 max-w-3xl text-base leading-relaxed text-muted">{subtitle}</p>
      </div>
    </header>
  );
}
