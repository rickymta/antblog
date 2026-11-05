/**
 * TEST-046 «Multiple Condition Testing» — bảng chân trị của quyết định «tài xế nhận được chuyến»
 * (SPEC R-ACCEPT-01) và kết quả mong đợi theo thứ tự kiểm R-ACCEPT-02.
 *
 * Hai mức tách điều kiện:
 * - Mức đặc tả (N = 4): online, cungThanhPho, dungLoaiXe, dangSearching ⇒ 16 tổ hợp.
 * - Mức code (N = 5): «online» của R-ACCEPT-01 là «không offline VÀ không busy» và code kiểm
 *   hai điều kiện này riêng (assertDriverAvailable) ⇒ 32 tổ hợp, trong đó offline ∧ busy bất khả thi.
 */
export const DIEU_KIEN_CODE = ['khongOffline', 'khongBusy', 'cungThanhPho', 'dungLoaiXe', 'dangSearching'] as const;
export type DieuKienCode = (typeof DIEU_KIEN_CODE)[number];
export type To = Record<DieuKienCode, boolean>;

export interface DongBang {
  so: number; // số thứ tự 1…32 (bit cao = khongOffline)
  to: To;
  khaThi: boolean;
  lyDoBatKhaThi: string | null;
  /** Kết quả mong đợi: 'OK' hoặc mã lỗi đầu tiên theo R-ACCEPT-02. */
  mongDoi: string;
}

/** Sinh 2^N tổ hợp theo thứ tự bảng chân trị (TTTTT trước, FFFFF sau). */
export function sinhToHop<K extends string>(ten: readonly K[]): Record<K, boolean>[] {
  const n = ten.length;
  return Array.from({ length: 2 ** n }, (_, i) =>
    Object.fromEntries(ten.map((k, j) => [k, ((i >> (n - 1 - j)) & 1) === 0])) as Record<K, boolean>,
  );
}

/** Oracle theo R-ACCEPT-02. Chuyến không searching ở bảng = đã có tài xế khác nhận. */
export function mongDoi(t: To): string {
  if (!t.khongOffline) return 'DRIVER_OFFLINE';
  if (!t.khongBusy) return 'DRIVER_BUSY';
  if (!t.cungThanhPho) return 'CITY_MISMATCH';
  if (!t.dungLoaiXe) return 'VEHICLE_MISMATCH';
  if (!t.dangSearching) return 'TRIP_ALREADY_TAKEN';
  return 'OK';
}

export function bangMucCode(): DongBang[] {
  return sinhToHop(DIEU_KIEN_CODE).map((to, i) => {
    // Tài xế chỉ có MỘT trạng thái (offline | online | busy — SPEC R-ACCEPT-05, schema CHECK).
    const batKhaThi = !to.khongOffline && !to.khongBusy;
    return {
      so: i + 1,
      to,
      khaThi: !batKhaThi,
      lyDoBatKhaThi: batKhaThi ? 'tài xế vừa offline vừa busy' : null,
      mongDoi: batKhaThi ? '—' : mongDoi(to),
    };
  });
}
