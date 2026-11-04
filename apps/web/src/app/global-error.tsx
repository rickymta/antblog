'use client';

import './globals.css';

/**
 * Lưới an toàn cuối cùng khi chính layout gốc lỗi: không phụ thuộc i18n/Query (có thể chính chúng đang lỗi).
 * Tải lại trang là cách phục hồi. Chữ song ngữ cố định.
 */
export default function GlobalError({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <html lang="vi">
      <body>
        <div className="flex min-h-dvh flex-col items-center justify-center gap-4 bg-page px-6 text-center">
          <h1 className="text-2xl font-bold text-ink">Đã có lỗi xảy ra · Something went wrong</h1>
          <p className="max-w-md text-muted">Máy chủ chưa phản hồi hoặc mạng đang gián đoạn. Bạn thử lại sau ít phút nhé.</p>
          <div className="flex gap-3">
            <button type="button" onClick={() => reset()} className="rounded-ui bg-primary px-4 py-2 text-sm font-semibold text-white hover:bg-primary-hover">
              Thử lại · Retry
            </button>
            <a href="/" className="rounded-ui border border-line bg-surface px-4 py-2 text-sm font-semibold text-ink hover:bg-surface-2">
              Về trang chủ · Home
            </a>
          </div>
        </div>
      </body>
    </html>
  );
}
