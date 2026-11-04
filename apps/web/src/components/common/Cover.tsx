'use client';

import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { CategoryGlyph } from '@tkh/ui';
import { pickLocalized, type CategoryRef } from '@tkh/shared';
import { useLocale } from '../../i18n';
import { safeColor } from '../../lib/format';

/**
 * Ảnh bìa bài viết; không có ảnh hoặc ảnh lỗi ⇒ khối gradient màu chủ đề + biểu tượng chủ đề (docs/ui-guide.md §3).
 * Khung giữ tỉ lệ cố định để trang không nhảy khi ảnh tải xong.
 */
export function Cover({
  src,
  category,
  className = '',
  iconSize = 'h-10 w-10',
  eager = false,
}: {
  src?: string;
  category: Pick<CategoryRef, 'name' | 'color' | 'icon'>;
  className?: string;
  iconSize?: string;
  eager?: boolean;
}) {
  const { t } = useTranslation();
  const locale = useLocale();
  const [failed, setFailed] = useState<string | null>(null);
  const color = safeColor(category.color);

  if (src && failed !== src) {
    return (
      <img
        src={src}
        alt=""
        width={640}
        height={360}
        loading={eager ? 'eager' : 'lazy'}
        fetchPriority={eager ? 'high' : undefined}
        decoding="async"
        referrerPolicy="no-referrer"
        onError={() => setFailed(src)}
        className={`block h-full w-full bg-surface-2 object-cover ${className}`}
      />
    );
  }
  return (
    <div
      role="img"
      aria-label={t('common.noCover', { topic: pickLocalized(category.name, locale) })}
      className={`relative flex h-full w-full items-center justify-center overflow-hidden ${className}`}
      style={{ background: `linear-gradient(135deg, ${color} 0%, #312E81 100%)` }}
    >
      <span aria-hidden="true" className="absolute -top-6 -right-6 h-24 w-24 rounded-full bg-white/10" />
      <span aria-hidden="true" className="absolute -bottom-8 -left-4 h-28 w-28 rounded-full bg-black/10" />
      <CategoryGlyph icon={category.icon} className={`relative text-white drop-shadow ${iconSize}`} strokeWidth={1.5} />
    </div>
  );
}
