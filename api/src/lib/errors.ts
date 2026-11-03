export class AppError extends Error {
  readonly status: number;
  readonly code: string;
  readonly details?: Record<string, unknown>;

  constructor(status: number, code: string, message: string, details?: Record<string, unknown>) {
    super(message);
    this.name = 'AppError';
    this.status = status;
    this.code = code;
    this.details = details;
  }
}

export const notFound = (message = 'Không tìm thấy dữ liệu yêu cầu.') => new AppError(404, 'NOT_FOUND', message);
export const unauthenticated = (message = 'Bạn cần đăng nhập để thực hiện thao tác này.') =>
  new AppError(401, 'UNAUTHENTICATED', message);
export const forbidden = (message = 'Bạn không có quyền thực hiện thao tác này.') => new AppError(403, 'FORBIDDEN', message);
export const conflict = (code: string, message: string) => new AppError(409, code, message);
export const validation = (fields: Record<string, string>, message = 'Dữ liệu gửi lên không hợp lệ.') =>
  new AppError(400, 'VALIDATION', message, { fields });

/** Lỗi trùng khoá unique của MongoDB. */
export function isDuplicateKeyError(err: unknown): boolean {
  return typeof err === 'object' && err !== null && (err as { code?: unknown }).code === 11000;
}
