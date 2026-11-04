'use client';

import Link from 'next/link';
import { useTranslation } from 'react-i18next';

/**
 * Ô logo "</>". Phông mono của HỆ THỐNG (không dùng JetBrains Mono): JetBrains Mono chỉ nạp ở trang bài (khối mã) —
 * logo mà dùng nó thì trang nào cũng phải tải thêm một tệp phông chỉ để vẽ 3 ký tự.
 */
export function LogoMark({ className = 'h-9 w-9 text-sm' }: { className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={`inline-flex shrink-0 items-center justify-center rounded-ui bg-gradient-to-br from-primary to-secondary font-bold text-white [font-family:ui-monospace,'Cascadia_Mono',Consolas,monospace] ${className}`}
    >
      {'</>'}
    </span>
  );
}

/**
 * Logo về trang chủ. Tên truy cập lấy từ chính chữ nhìn thấy (tên site + khẩu hiệu) — không dùng aria-label khác chữ
 * hiển thị (WCAG 2.5.3). Màn < 400px ẩn chữ bằng sr-only (vẫn còn tên cho trình đọc màn hình).
 */
export function Logo({ onNavigate }: { onNavigate?: () => void }) {
  const { t } = useTranslation();
  return (
    <Link href="/" onClick={onNavigate} className="flex min-w-0 items-center gap-2.5 rounded-ui">
      <LogoMark />
      <span className="min-w-0 leading-tight max-[399px]:sr-only">
        <span className="block truncate text-[15px] font-bold text-ink">{t('meta.siteName')}</span>
        <span className="block truncate text-[11px] font-medium text-muted">{t('meta.tagline')}</span>
      </span>
    </Link>
  );
}
