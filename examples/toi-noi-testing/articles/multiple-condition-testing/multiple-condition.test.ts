/**
 * TEST-046 «Multiple Condition Testing» — chạy mọi tổ hợp khả thi của quyết định
 * «tài xế nhận được chuyến» (R-ACCEPT-01/02) trên code thật của lab.
 *
 * Chạy: npx vitest run articles/multiple-condition-testing
 */
import { describe, expect, it } from 'vitest';
import { bangMucCode, mongDoi, sinhToHop, type DongBang } from './bang-chan-tri';
import { chayDong } from './dung-tinh-huong';
import { acceptTripSaiThuTu } from './nhan-chuyen-sai-thu-tu';

const bang = bangMucCode();
const khaThi = bang.filter((d) => d.khaThi);
const nhan = (d: DongBang) => Object.values(d.to).map((b) => (b ? 'T' : 'F')).join('');
/** Bộ «lật từng điều kiện» (kiểu N + 1): dòng toàn T và các dòng chỉ có MỘT điều kiện F. */
const boLatTung = khaThi.filter((d) => Object.values(d.to).filter((b) => !b).length <= 1);

describe('TEST-046 · bảng chân trị mức code (N = 5)', () => {
  it('32 tổ hợp, 8 bất khả thi (offline ∧ busy), 24 khả thi', () => {
    expect([bang.length, bang.length - khaThi.length, khaThi.length]).toEqual([32, 8, 24]);
  });

  it('mức đặc tả (N = 4): 16 tổ hợp, không tổ hợp nào bất khả thi', () => {
    expect(sinhToHop(['online', 'cungThanhPho', 'dungLoaiXe', 'dangSearching'] as const)).toHaveLength(16);
  });

  it('24 dòng khả thi chỉ cho 6 kết quả khác nhau (if…throw ngắt mạch)', () => {
    expect(new Set(khaThi.map((d) => d.mongDoi)).size).toBe(6);
  });
});

describe('TEST-046 · multiple condition testing trên src/app/trips.ts', () => {
  it.each(khaThi.map((d) => [d.so, nhan(d), d.mongDoi] as const))('#%i %s ⇒ %s', async (so, _nhan, kq) => {
    const dong = bang[so - 1]!;
    expect(await chayDong(dong.to)).toBe(kq);
  });
});

describe('TEST-046 · bản sai thứ tự (loại xe trước thành phố)', () => {
  const phatHien = async (dong: DongBang[]) => {
    const lo: string[] = [];
    for (const d of dong) if ((await chayDong(d.to, acceptTripSaiThuTu)) !== mongDoi(d.to)) lo.push(`#${d.so} ${nhan(d)}`);
    return lo;
  };

  it('bộ lật từng điều kiện (6 test) không phát hiện', async () => {
    expect(boLatTung).toHaveLength(6);
    expect(await phatHien(boLatTung)).toEqual([]);
  });

  it('đủ 24 tổ hợp khả thi phát hiện ở đúng 2 dòng', async () => {
    expect(await phatHien(khaThi)).toEqual(['#7 TTFFT', '#8 TTFFF']);
  });
});
