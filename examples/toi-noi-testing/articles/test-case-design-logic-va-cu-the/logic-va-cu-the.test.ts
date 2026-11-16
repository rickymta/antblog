/**
 * TCD-03 · Test case logic và test case cụ thể — R-FOOD qua POST /api/quotes, cộng procedure/suite/schedule.
 */
import request from 'supertest';
import { describe, expect, it } from 'vitest';
import { TAT_CA_CU_THE, moRong, phiGiaoTheoSpec, type TestCaseCuThe } from './mo-rong';
import { BANG_DU_LIEU, TEST_CASE_LOGIC, type DongDuLieu } from './test-case-logic';
import {
  TP_FOOD_01, TP_FOOD_02, TP_FOOD_03, TP_FOOD_03A, TP_FOOD_03B,
  chayThuTuc, chayTheoLich, lapLich, taoNguCanh, type MucLich, type TestSuite,
} from './thu-tuc';

const SPEC_CODES = /^R-(FOOD|FARE|VALID|VOUCHER|GEN|API)-\d{2}$/;

describe('TCD-03 · 1. kiểm chính bộ test case trước khi chạy', () => {
  it('12 test case logic, 19 dòng test data, mỗi logic có ≥ 1 dòng, mỗi dòng thuộc đúng một logic', () => {
    expect(TEST_CASE_LOGIC).toHaveLength(12);
    expect(BANG_DU_LIEU).toHaveLength(19);
    for (const tc of TEST_CASE_LOGIC) expect(BANG_DU_LIEU.some((d) => d.tc === tc.id)).toBe(true);
    for (const d of BANG_DU_LIEU) expect(TEST_CASE_LOGIC.filter((tc) => tc.id === d.tc)).toHaveLength(1);
    for (const tc of TEST_CASE_LOGIC) for (const r of tc.truyVet) expect(r).toMatch(SPEC_CODES);
  });

  it('mở rộng ra 19 test case cụ thể, id duy nhất dạng TC-FOOD-nnn/TD-nn', () => {
    expect(TAT_CA_CU_THE).toHaveLength(19);
    expect(new Set(TAT_CA_CU_THE.map((c) => c.id)).size).toBe(19);
    expect(TAT_CA_CU_THE.map((c) => c.id)).toContain('TC-FOOD-005/TD-08');
  });

  it('kết quả mong đợi tính bằng công thức SPEC khớp cột tính tay của từng dòng (ghi hai lần, kiểm chéo)', () => {
    for (const c of TAT_CA_CU_THE) {
      const d = BANG_DU_LIEU.find((x) => c.id.endsWith(x.id))!;
      if (typeof d.tinhTay === 'number') expect(c.mongDoi, c.id).toMatchObject({ http: 200, total: d.tinhTay });
      else expect(c.mongDoi, c.id).toEqual(d.tinhTay);
    }
  });

  it('ví dụ tính tay của bài: 3,3 km cao điểm + mưa ⇒ 26.850 × 1,25 + 8.000 = 41.562,5 ⇒ 42.000 đ', () => {
    expect(phiGiaoTheoSpec(3.3, false, false)).toBe(27_000);
    expect(phiGiaoTheoSpec(3.3, true, false)).toBe(34_000);
    expect(phiGiaoTheoSpec(3.3, true, true)).toBe(42_000);
    expect(phiGiaoTheoSpec(0.6, false, false)).toBe(15_000);
  });

  it('dòng dữ liệu trượt khỏi lớp của test case logic bị chặn khi mở rộng', () => {
    const tc002 = TEST_CASE_LOGIC.find((t) => t.id === 'TC-FOOD-002')!;
    const sai: DongDuLieu = { ...BANG_DU_LIEU[1]!, id: 'TD-99', mon: [{ name: 'Bánh mì', unitPrice: 16_000, quantity: 2 }] }; // 32.000 ≠ 30.000
    expect(() => moRong(tc002, [sai])).toThrow('TD-99 không thuộc lớp mà TC-FOOD-002 mô tả');
  });
});

describe('TCD-03 · 2. chạy 19 test case cụ thể qua POST /api/quotes', () => {
  it.each(TAT_CA_CU_THE.map((c) => [c.id, c] as const))('%s', async (_id, c: TestCaseCuThe) => {
    // tiền điều kiện cụ thể: hệ thống mới, đồng hồ, thời tiết
    const ctx = taoNguCanh();
    ctx.clock.set(new Date(c.tienDieuKien.dongHo));
    ctx.system.weather.setRaining(c.tienDieuKien.thanhPhoMua as never);

    const res = await request(ctx.app).post('/api/quotes').set('x-user', 'customer:C001').send(c.body);

    expect(res.status).toBe(c.mongDoi.http);
    if (c.mongDoi.http === 200) {
      const { http: _http, ...giaTri } = c.mongDoi;
      expect(res.body.quote).toMatchObject(giaTri);
      expect(res.body.quote.fare.discount).toBe(0); // voucher không trừ vào phí giao (R-FOOD-04)
    } else {
      expect(res.body.error.code).toBe(c.mongDoi.code);
    }
  });
});

describe('TCD-03 · 3. test procedure: setup → bước → teardown', () => {
  it('TP-FOOD-01 đạt, nhật ký đúng thứ tự 2 setup + 5 bước + 2 teardown', async () => {
    const kq = await chayThuTuc(TP_FOOD_01);
    expect(kq.ketQua).toBe('pass');
    expect(kq.nhatKy).toEqual([
      'setup: đồng hồ 04/11/2026 09:00 giờ VN',
      'setup: SG không mưa',
      'bước: báo giá đơn đồ ăn ⇒ 200, tổng 102000',
      'bước: đặt chuyến từ báo giá ⇒ 201, searching',
      'bước: tài xế D101 nhận, ETA 5 phút ⇒ accepted',
      'bước: đồng hồ +1 phút',
      'bước: khách huỷ ⇒ 200, phí 0, grace_period',
      'teardown: kiểm hậu điều kiện: D101 về online',
      'teardown: dọn: reset kho, thời tiết, ví giả',
    ]);
  });

  it('TP-FOOD-02 đạt: đơn đồ ăn huỷ sau 5 phút mất 10.000 đ (R-CANCEL-03)', async () => {
    expect((await chayThuTuc(TP_FOOD_02)).ketQua).toBe('pass');
  });

  it('bước thất bại giữa chừng: các bước sau không chạy nhưng teardown vẫn chạy', async () => {
    const hong = { ...TP_FOOD_01, id: 'TP-FOOD-01-hong', buoc: [TP_FOOD_01.buoc[0]!, TP_FOOD_01.buoc[2]!, ...TP_FOOD_01.buoc.slice(3)] }; // bỏ bước đặt chuyến
    const kq = await chayThuTuc(hong);
    expect(kq.ketQua).toBe('fail');
    expect(kq.nhatKy.filter((l) => l.startsWith('bước:'))).toHaveLength(1);
    expect(kq.nhatKy.slice(-2)).toEqual(['teardown: kiểm hậu điều kiện: D101 về online', 'teardown: dọn: reset kho, thời tiết, ví giả']);
  });
});

describe('TCD-03 · 4. phụ thuộc giữa test case', () => {
  it('bản phụ thuộc: 03a rồi 03b trên CÙNG môi trường ⇒ cả hai đạt', async () => {
    const ctx = taoNguCanh();
    expect((await chayThuTuc(TP_FOOD_03A, ctx)).ketQua).toBe('pass');
    expect((await chayThuTuc(TP_FOOD_03B, ctx)).ketQua).toBe('pass');
  });

  it('bản phụ thuộc: chạy riêng 03b (hoặc đảo thứ tự) ⇒ 03b trượt dù hệ thống không có lỗi', async () => {
    const rieng = await chayThuTuc(TP_FOOD_03B);
    expect(rieng.ketQua).toBe('fail');
    expect(rieng.loi).toContain('expected 200 to be 422');
  });

  it('bản độc lập TP-FOOD-03: chạy một mình, chạy hai lần liền, chạy sau procedure khác — luôn đạt', async () => {
    expect((await chayThuTuc(TP_FOOD_03)).ketQua).toBe('pass');
    const ctx = taoNguCanh();
    expect((await chayThuTuc(TP_FOOD_03, ctx)).ketQua).toBe('pass');
    expect((await chayThuTuc(TP_FOOD_03, ctx)).ketQua).toBe('pass'); // teardown đã dọn
    expect((await chayThuTuc(TP_FOOD_02, ctx)).ketQua).toBe('pass');
  });
});

describe('TCD-03 · 5. test suite và test execution schedule', () => {
  const idCuThe = new Set(TAT_CA_CU_THE.map((c) => c.id));
  const idThuTuc = new Set([TP_FOOD_01, TP_FOOD_02, TP_FOOD_03].map((t) => t.id));
  const SUITES: TestSuite[] = [
    { id: 'TS-FOOD-SMOKE', mucDich: 'chạy mỗi lần build', thanhPhan: ['TC-FOOD-001/TD-01', 'TP-FOOD-01'] },
    { id: 'TS-FOOD-REGRESSION', mucDich: 'chạy trước mỗi bản phát hành', thanhPhan: [...idCuThe, ...idThuTuc] },
  ];

  it('mọi thành phần của suite đều tồn tại, không trùng; smoke là tập con của regression', () => {
    for (const s of SUITES) {
      expect(new Set(s.thanhPhan).size).toBe(s.thanhPhan.length);
      for (const id of s.thanhPhan) expect(idCuThe.has(id) || idThuTuc.has(id), id).toBe(true);
    }
    expect(SUITES[1]!.thanhPhan).toHaveLength(22); // 19 test case cụ thể + 3 procedure
    for (const id of SUITES[0]!.thanhPhan) expect(SUITES[1]!.thanhPhan).toContain(id);
  });

  const LICH: MucLich[] = [
    { id: 'TS-FOOD-SMOKE', uuTien: 1, phuThuoc: [] },
    { id: 'TC-FOOD-003..012 (ca âm)', uuTien: 2, phuThuoc: ['TS-FOOD-SMOKE'] },
    { id: 'TC-FOOD-001..011 (ca dương)', uuTien: 1, phuThuoc: ['TS-FOOD-SMOKE'] },
    { id: 'TP-FOOD-02', uuTien: 2, phuThuoc: ['TS-FOOD-SMOKE'] },
    { id: 'TP-FOOD-03a', uuTien: 3, phuThuoc: ['TS-FOOD-SMOKE'] },
    { id: 'TP-FOOD-03b', uuTien: 1, phuThuoc: ['TP-FOOD-03a'] },
  ];

  it('lịch tôn trọng phụ thuộc trước, ưu tiên sau: 03b (ưu tiên 1) vẫn phải chờ 03a (ưu tiên 3)', () => {
    expect(lapLich(LICH)).toEqual([
      'TS-FOOD-SMOKE', 'TC-FOOD-001..011 (ca dương)', 'TC-FOOD-003..012 (ca âm)', 'TP-FOOD-02', 'TP-FOOD-03a', 'TP-FOOD-03b',
    ]);
  });

  it('vòng phụ thuộc thì không lập được lịch', () => {
    expect(() => lapLich([{ id: 'A', uuTien: 1, phuThuoc: ['B'] }, { id: 'B', uuTien: 1, phuThuoc: ['A'] }])).toThrow('Vòng phụ thuộc');
  });

  it('smoke trượt ⇒ 5 mục còn lại bị blocked; chỉ 03a trượt ⇒ chỉ 03b bị blocked', () => {
    const tatCaDat = Object.fromEntries(LICH.map((m) => [m.id, 'pass' as const]));
    const smokeTruot = chayTheoLich(LICH, { ...tatCaDat, 'TS-FOOD-SMOKE': 'fail' });
    expect(Object.values(smokeTruot).filter((v) => v === 'blocked')).toHaveLength(5);
    const truot03a = chayTheoLich(LICH, { ...tatCaDat, 'TP-FOOD-03a': 'fail' });
    expect(Object.entries(truot03a).filter(([, v]) => v === 'blocked').map(([k]) => k)).toEqual(['TP-FOOD-03b']);
  });
});
