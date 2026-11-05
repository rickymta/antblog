/**
 * TEST-031 · EP bắt được lỗi gì và bỏ lọt lỗi gì?
 * Chạy CÙNG bảng đại diện EP của quãng đường trên bản đúng và hai bản lỗi cố ý (quang-duong-loi.ts).
 */
import { describe, expect, it } from 'vitest';
import { normalizeDistanceKm } from '../../src/domain/fare';
import { kiemTruocLamTron, thieuCanTren, type Normalize } from './quang-duong-loi';

// Một đại diện "ở giữa" mỗi phân vùng — đúng tinh thần EP
const EP_TABLE = [
  { partition: 'D1', km: 0.2, valid: false },
  { partition: 'D2', km: 7.4, valid: true },
  { partition: 'D3', km: 150, valid: false },
  { partition: 'D4′', km: Number.NaN, valid: false },
];

/** Số ca EP cho kết quả khác mong đợi (chấp nhận/từ chối) trên một bản cài đặt. */
function failingCases(normalize: Normalize): string[] {
  return EP_TABLE.filter(({ km, valid }) => {
    let accepted: boolean;
    try {
      normalize(km);
      accepted = true;
    } catch {
      accepted = false;
    }
    return accepted !== valid;
  }).map((r) => `${r.partition} (${r.km})`);
}

describe('Cùng bảng EP trên ba bản cài đặt', () => {
  it('Bản đúng (src): 0 ca đỏ', () => {
    expect(failingCases(normalizeDistanceKm)).toEqual([]);
  });

  it('Lỗi A «quên cận trên»: EP BẮT được nhờ đại diện của D3', () => {
    expect(failingCases(thieuCanTren)).toEqual(['D3 (150)']);
  });

  it('Lỗi B «kiểm trước khi làm tròn»: EP BỎ LỌT — lỗi nằm sát biên', () => {
    expect(failingCases(kiemTruocLamTron)).toEqual([]);
    // Chỉ giá trị ở biên mới lộ lỗi: 0,45 phải hợp lệ (R-FARE-02) — việc của BVA (TEST-032)
    expect(() => normalizeDistanceKm(0.45)).not.toThrow();
    expect(() => kiemTruocLamTron(0.45)).toThrow('DISTANCE_OUT_OF_RANGE');
  });
});
