import type { TFunction } from 'i18next';
import { ApiError } from '@tkh/shared';

export function isApiError(err: unknown): err is ApiError {
  return err instanceof ApiError;
}

export function hasStatus(err: unknown, status: number): boolean {
  return isApiError(err) && err.status === status;
}

export function hasCode(err: unknown, code: string): boolean {
  return isApiError(err) && err.code === code;
}

/** Lỗi trường từ API (`tags.0`, `name.vi`…) gộp theo khoá gốc — giữ thông điệp đầu tiên. */
export function topLevelFields(fields: Record<string, string>): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [k, v] of Object.entries(fields)) {
    const top = k.split('.')[0] ?? k;
    if (!(top in out)) out[top] = v;
  }
  return out;
}

/** Phiên hết hạn/không có ⇒ quay về màn đăng nhập. Sai mật khẩu lúc đăng nhập KHÔNG tính. */
export function isSessionLost(err: unknown): boolean {
  return isApiError(err) && err.status === 401 && err.code !== 'INVALID_CREDENTIALS';
}

/**
 * Câu lỗi cho người dùng. Lỗi mạng (API chưa chạy, proxy trả HTML) và 5xx dùng câu của giao diện;
 * lỗi nghiệp vụ 4xx dùng câu API (tiếng Việt) khi đang ở tiếng Việt, câu chung theo mã khi ở tiếng Anh.
 */
export function errorMessage(err: unknown, t: TFunction, lang: string): string {
  if (!isApiError(err)) return t('errors.network');
  // 5xx không có thân lỗi của API (vd proxy Vite báo API chưa chạy) ⇒ coi như lỗi kết nối.
  if (err.status >= 500) return err.message.startsWith('Lỗi ') ? t('errors.network') : t('errors.server');
  if (lang === 'vi' && err.message && !err.message.startsWith('Lỗi ')) return err.message;
  switch (err.status) {
    case 400:
      return t('errors.validation');
    case 403:
      return t('errors.forbidden');
    case 404:
      return t('errors.notFound');
    case 429:
      return t('errors.rateLimited');
    default:
      return err.message || t('errors.unknown');
  }
}
