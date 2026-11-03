/** Sao chép chữ vào clipboard; ngữ cảnh không hỗ trợ Clipboard API thì dùng textarea tạm. */
export async function copyText(text: string): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return true;
    }
  } catch {
    /* rơi xuống cách dự phòng */
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.setAttribute('readonly', '');
    ta.style.position = 'fixed';
    ta.style.opacity = '0';
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    document.body.removeChild(ta);
    return ok;
  } catch {
    return false;
  }
}

export function prefersReducedMotion(): boolean {
  try {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  } catch {
    return false;
  }
}

/** Cuộn tới phần tử theo id (mượt nếu người dùng không tắt chuyển động) và cập nhật hash mà không tạo lượt điều hướng mới. */
export function scrollToId(id: string): void {
  const el = document.getElementById(id);
  if (!el) return;
  el.scrollIntoView({ behavior: prefersReducedMotion() ? 'auto' : 'smooth', block: 'start' });
  try {
    // Giữ nguyên state của React Router để lịch sử điều hướng không lệch.
    window.history.replaceState(window.history.state, '', `#${encodeURIComponent(id)}`);
  } catch {
    /* bỏ qua */
  }
}
