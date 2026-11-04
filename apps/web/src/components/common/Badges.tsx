'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';
import { CategoryGlyph, LevelBadge } from '@tkh/ui';
import { pickLocalized, type CategoryRef, type Level } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { safeColor, withAlpha } from '../../lib/format';

export function LevelTag({ level, className }: { level: Level; className?: string }) {
  const { t } = useTranslation();
  return <LevelBadge level={level} label={t(`common.levels.${level}`)} className={className} />;
}

/**
 * Nhãn chủ đề: chấm/biểu tượng màu chủ đề + tên chữ màu mực thường (màu chủ đề do người quản trị chọn,
 * không đảm bảo tương phản nên không dùng làm màu chữ).
 */
export function CategoryChip({
  category,
  link = true,
  uppercase = false,
}: {
  category: Pick<CategoryRef, 'slug' | 'name' | 'color' | 'icon'>;
  link?: boolean;
  uppercase?: boolean;
}) {
  const locale = useLocale();
  const color = safeColor(category.color);
  const content = (
    <>
      <CategoryGlyph icon={category.icon} className="h-3.5 w-3.5" style={{ color }} />
      <span className={uppercase ? 'tracking-wide uppercase' : undefined}>{pickLocalized(category.name, locale)}</span>
    </>
  );
  const cls = 'inline-flex max-w-full items-center gap-1.5 truncate rounded-ui px-2 py-0.5 text-xs font-semibold text-ink';
  const style = { backgroundColor: withAlpha(color, 0.12) };
  return link ? (
    // min-h-6: vùng bấm tối thiểu 24 × 24 px (WCAG 2.5.8) — các thẻ chủ đề trên lưới bài nằm sát nhau.
    <Link href={`/topics/${encodeURIComponent(category.slug)}`} className={`${cls} min-h-6 hover:underline`} style={style}>
      {content}
    </Link>
  ) : (
    <span className={cls} style={style}>
      {content}
    </span>
  );
}

export function TagChip({ tag }: { tag: string }) {
  return (
    <Link
      href={`/articles?tag=${encodeURIComponent(tag)}`}
      className="inline-flex items-center rounded-ui border border-line bg-surface px-2 py-1 text-xs font-medium text-ink hover:border-primary hover:text-primary-ink"
    >
      #{tag}
    </Link>
  );
}
