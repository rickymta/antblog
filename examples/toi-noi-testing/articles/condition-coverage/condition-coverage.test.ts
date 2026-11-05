/**
 * TEST-045 «Condition Coverage» — kiểm các khẳng định của bài trên code thật của lab.
 *
 * Chạy: npx vitest run articles/condition-coverage
 */
import { describe, expect, it } from 'vitest';
import { assertVoucherEligible } from '../../src/domain/voucher';
import { BO_A, BO_B, BO_C, HN11 } from './bo-test';
import { chayThat, giaTriTheoSpec, quyetDinh, type DuLieuTest } from './dieu-kien-voucher';
import { doPhu, type DongChay } from './do-phu';
import { DOT_BIEN } from './dot-bien';

const theoBangChanTri = (bo: DuLieuTest[]): DongChay[] =>
  bo.map((t) => {
    const giaTri = giaTriTheoSpec(HN11, t);
    return { giaTri, ketQua: quyetDinh(giaTri) };
  });
const theoCodeThat = (bo: DuLieuTest[]): DongChay[] =>
  bo.map((t) => {
    const r = chayThat(HN11, t);
    return { giaTri: giaTriTheoSpec(HN11, t), daDanhGia: r.daDanhGia, ketQua: r.apDuoc };
  });

describe('TEST-045 · oracle SPEC khớp code thật (R-VOUCHER-04…08)', () => {
  it.each([...BO_A, ...BO_B, ...BO_C])('$id · quyết định theo SPEC = kết quả code thật', (t) => {
    expect(chayThat(HN11, t).apDuoc).toBe(quyetDinh(giaTriTheoSpec(HN11, t)));
  });

  it('mã lỗi theo thứ tự R-VOUCHER-08', () => {
    expect([...BO_A, ...BO_B, ...BO_C].map((t) => `${t.id}:${chayThat(HN11, t).ma ?? 'OK'}`)).toEqual([
      'A1:VOUCHER_EXPIRED',
      'A2:VOUCHER_NOT_APPLICABLE',
      'B1:OK',
      'B2:VOUCHER_EXPIRED',
      'C0:OK',
      'C1:VOUCHER_EXPIRED',
      'C2:VOUCHER_NOT_APPLICABLE',
      'C3:VOUCHER_NOT_APPLICABLE',
      'C4:VOUCHER_MIN_AMOUNT',
    ]);
    expect(chayThat(HN11, BO_C[0]!).tienGiamDong).toBe(5_000); // min(10 % × 50.000; 15.000)
  });
});

describe('TEST-045 · Bộ A: condition coverage đạt, decision coverage hụt', () => {
  it('bảng chân trị: 8/8 item điều kiện, 1/2 item quyết định (không test nào áp được voucher)', () => {
    const d = doPhu(theoBangChanTri(BO_A));
    expect(d.dieuKien).toMatchObject({ phu: 8, tong: 8 });
    expect(d.quyetDinh).toMatchObject({ phu: 1, tong: 2, thieu: ['quyếtĐịnh=T'] });
  });

  it('code thật: điều kiện sau lỗi đầu tiên không được tính ⇒ chỉ 4/8 item điều kiện', () => {
    const d = doPhu(theoCodeThat(BO_A));
    expect(d.dieuKien.phu).toBe(4);
    expect(d.dieuKien.thieu).toEqual(['dungDichVu=T', 'dungThanhPho=T', 'duTien=T', 'duTien=F']);
  });

  it('quyết định if (!kindOk || !cityOk) của src: hai test lật chéo đều ném lỗi, nhánh «áp được» chưa chạy', () => {
    const at = BO_C[0]!.at;
    expect(() => assertVoucherEligible(HN11, { kind: 'parcel', cityCode: 'HN', at })).toThrow(/không áp dụng/);
    expect(() => assertVoucherEligible(HN11, { kind: 'ride', cityCode: 'SG', at })).toThrow(/không áp dụng/);
  });
});

describe('TEST-045 · Bộ B và Bộ C', () => {
  it('Bộ B: 2 test đã đủ condition + decision coverage (bảng chân trị)', () => {
    const d = doPhu(theoBangChanTri(BO_B));
    expect([d.dieuKien.phu, d.quyetDinh.phu]).toEqual([8, 2]);
  });

  it('Bộ C: 8/8 và 2/2 kể cả khi đếm theo code thật', () => {
    const d = doPhu(theoCodeThat(BO_C));
    expect([d.dieuKien.phu, d.quyetDinh.phu]).toEqual([8, 2]);
  });
});

describe('TEST-045 · bộ nào phát hiện được bản cài đặt sai?', () => {
  const phatHien = (bo: DuLieuTest[], ma: string) =>
    bo.filter((t) => {
      const g = giaTriTheoSpec(HN11, t);
      return DOT_BIEN[ma]!.quyetDinh(g) !== quyetDinh(g);
    }).map((t) => t.id);

  it('Bộ A và Bộ B không phát hiện M1, M2, M3; Bộ C phát hiện cả ba', () => {
    const bang = Object.keys(DOT_BIEN).map((ma) => [ma, phatHien(BO_A, ma), phatHien(BO_B, ma), phatHien(BO_C, ma)]);
    expect(bang).toEqual([
      ['M1', [], [], ['C3']],
      ['M2', [], [], ['C2', 'C3']],
      ['M3', [], [], ['C4']],
    ]);
  });
});
