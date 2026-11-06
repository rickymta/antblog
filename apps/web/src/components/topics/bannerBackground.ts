import type { CSSProperties } from 'react';
import { safeColor } from '../../lib/format';

/**
 * Nền banner chủ đề: ảnh `public/images/categories/<slug>.webp` phủ lên gradient màu chủ đề → navy.
 * Hai lớp nền trong CÙNG một khai báo CSS ⇒ chủ đề tạo từ CMS chưa có ảnh (404) tự rơi về gradient, không cần JS.
 * Ảnh vẽ chừa trống nửa trái (chỗ đặt chữ) nên neo phải để phần minh hoạ luôn còn trong khung khi bị cắt.
 */
export function bannerBackground(slug: string, color: string | undefined): CSSProperties {
  return {
    backgroundImage: `url("/images/categories/${encodeURIComponent(slug)}.webp"), linear-gradient(135deg, ${safeColor(color)}, #0F172A)`,
    backgroundSize: 'cover',
    backgroundPosition: 'right center',
    backgroundRepeat: 'no-repeat',
  };
}
