// Gọi API Tới Nơi và định dạng hiển thị — dùng chung cho trang khách và trang tài xế.

/** Gọi API JSON; lỗi ném Error có `code` lấy từ `{ error: { code, message } }`. */
export async function callApi(method, path, { user, body, idempotencyKey } = {}) {
  const headers = { 'content-type': 'application/json' };
  if (user) headers['x-user'] = user;
  if (idempotencyKey) headers['idempotency-key'] = idempotencyKey;
  const response = await fetch(path, { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  const data = await response.json().catch(() => ({}));
  if (!response.ok) {
    const error = new Error(data.error?.message ?? `Lỗi ${response.status}`);
    error.code = data.error?.code ?? 'UNKNOWN';
    throw error;
  }
  return data;
}

const money = new Intl.NumberFormat('vi-VN', { maximumFractionDigits: 0 });

/** 52000 ⇒ "52.000 đ" (SPEC R-UI-04). */
export function formatDong(value) {
  return `${money.format(value)} đ`;
}

export const STATUS_LABELS = {
  requested: 'Đã gửi yêu cầu',
  searching: 'Đang tìm tài xế',
  accepted: 'Tài xế đã nhận',
  arriving: 'Tài xế đang tới',
  picked_up: 'Đã đón / lấy hàng',
  in_transit: 'Đang chở / giao',
  completed: 'Hoàn thành',
  cancelled: 'Đã huỷ',
};

export function newIdempotencyKey() {
  return crypto.randomUUID();
}
