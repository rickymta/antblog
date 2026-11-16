/**
 * TCD-08 — kiểm chính RTM sinh từ mã nguồn: độ phủ, lỗ hổng, mồ côi, tag lệch, ảnh hưởng, truy vết ngược.
 * Các con số khẳng định ở đây là TRẠNG THÁI HIỆN TẠI có chủ đích của lab (có lỗ hổng để bài phân tích).
 * Trong CI thật, thường khẳng định «không mồ côi, không lệch tag, độ phủ yêu cầu ≥ ngưỡng».
 */
import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { TEST_CONDITIONS } from './dieu-kien';
import { SPEC_PATH, anhHuong, docNguon, docTieuDeTest, docYeuCau, dungRtm, phanTichTag, truyNguoc } from './rtm';

const nguon = docNguon();
const rtm = dungRtm(nguon);

describe('TCD-08 · RTM sinh từ mã nguồn', () => {
  it('đọc được 15 yêu cầu R-STATE/R-CANCEL từ SPEC.md, loại R-STATE-08 có lý do ⇒ phạm vi 14', () => {
    expect(docYeuCau(readFileSync(SPEC_PATH, 'utf8'))).toHaveLength(15);
    expect(rtm.phamVi).toHaveLength(14);
    expect(rtm.phamVi).not.toContain('R-STATE-08');
  });

  it('đọc được 17 test case từ tiêu đề test (16 có mã TC, 1 không mã)', () => {
    expect(nguon.testCase).toHaveLength(17);
    expect(nguon.testCase.filter((t) => t.id !== null)).toHaveLength(16);
  });

  it('bộ đọc tag: một tiêu đề cho mã TC, tag yêu cầu và tag test condition', () => {
    const [tieuDe] = docTieuDeTest("it('TC-X-001 @R-CANCEL-02 @TCOND-CANCEL-03 · biên', () => {})");
    expect(phanTichTag(tieuDe!, 'a.test.ts')).toMatchObject({ id: 'TC-X-001', spec: ['R-CANCEL-02'], tcond: ['TCOND-CANCEL-03'] });
  });

  it('mọi test condition đều truy về yêu cầu có trong SPEC (không có test condition «lơ lửng»)', () => {
    const coTrongSpec = new Set(nguon.yeuCau);
    expect(TEST_CONDITIONS.flatMap((d) => d.spec).filter((r) => !coTrongSpec.has(r))).toEqual([]);
    expect(rtm.yeuCauChuaCoDieuKien).toEqual([]);
  });

  it('độ phủ yêu cầu 13/14 = 92,9% nhưng độ phủ test condition chỉ 16/20 = 80%', () => {
    expect(rtm.phuYeuCau).toEqual({ daPhu: 13, tong: 14, phanTram: 92.9 });
    expect(rtm.phuDieuKien).toEqual({ daPhu: 16, tong: 20, phanTram: 80 });
  });

  it('lỗ hổng: R-STATE-05 chưa có test case; 4 test condition chưa phủ', () => {
    expect(rtm.yeuCauChuaPhu).toEqual(['R-STATE-05']);
    expect(rtm.dieuKienChuaPhu).toEqual(['TCOND-STATE-04', 'TCOND-STATE-07', 'TCOND-CANCEL-08', 'TCOND-CANCEL-09']);
  });

  it('«đã phủ» nhưng nông: R-CANCEL-04 có test case, còn biên của nó (TCOND-CANCEL-08) thì không', () => {
    const r04 = rtm.dong.find((d) => d.yeuCau === 'R-CANCEL-04')!;
    expect(r04).toMatchObject({ dieuKien: ['TCOND-CANCEL-07', 'TCOND-CANCEL-08'], testCase: ['TC-CANCEL-008'] });
    expect(rtm.dieuKienChuaPhu).toContain('TCOND-CANCEL-08');
  });

  it('2 test case mồ côi: một không có tag, một trỏ vào R-CANCEL-08 không tồn tại', () => {
    expect(rtm.moCoi.map((o) => o.lyDo)).toEqual(['không có tag @R-… hay @TCOND-…', 'R-CANCEL-08 không có trong test basis']);
  });

  it('1 tag lệch registry: TC-CANCEL-006 gắn R-CANCEL-02 nhưng TCOND-CANCEL-05 không truy về R-CANCEL-02', () => {
    expect(rtm.lechTag).toEqual([{ testCase: 'TC-CANCEL-006', yeuCau: 'R-CANCEL-02' }]);
  });

  it('nhiều-nhiều: R-CANCEL-01 có 5 test condition và 6 test case; TC-STATE-002 truy về 3 yêu cầu', () => {
    const r01 = rtm.dong.find((d) => d.yeuCau === 'R-CANCEL-01')!;
    expect(r01.dieuKien).toHaveLength(5);
    expect(r01.testCase).toHaveLength(6);
    expect(truyNguoc(nguon, 'TC-STATE-002')?.yeuCau).toEqual(['R-STATE-02', 'R-STATE-06', 'R-STATE-07']);
  });

  it('ảnh hưởng khi đổi mốc 2 phút → 3 phút (R-CANCEL-02): 2 test condition, 4 test case cần xem lại', () => {
    expect(anhHuong(nguon, ['R-CANCEL-02'])).toEqual({
      dieuKien: ['TCOND-CANCEL-02', 'TCOND-CANCEL-03'],
      testCase: ['TC-CANCEL-002', 'TC-CANCEL-003', 'TC-CANCEL-004', 'TC-CANCEL-006'],
    });
  });

  it('ảnh hưởng khi thêm một trạng thái mới (R-STATE-01/02/03, R-CANCEL-01): 9 test condition, 10 test case — test mồ côi không lọt vào danh sách', () => {
    const kq = anhHuong(nguon, ['R-STATE-01', 'R-STATE-02', 'R-STATE-03', 'R-CANCEL-01']);
    expect(kq.dieuKien).toHaveLength(9);
    expect(kq.testCase).toEqual([
      'TC-STATE-001', 'TC-STATE-002', 'TC-STATE-003', 'TC-STATE-004', 'TC-STATE-005', 'TC-STATE-006',
      'TC-CANCEL-001', 'TC-CANCEL-002', 'TC-CANCEL-005', 'TC-CANCEL-006',
    ]);
    expect(kq.testCase.some((t) => t.includes('lần thứ hai'))).toBe(false);
  });

  it('truy vết ngược từ defect (giả lập) DEF-SIM-001 ⇒ TC-CANCEL-006 ⇒ TCOND-CANCEL-05 ⇒ R-CANCEL-01 (+ R-CANCEL-02 do tag lệch)', () => {
    expect(truyNguoc(nguon, 'DEF-SIM-001')).toEqual({
      testCase: 'TC-CANCEL-006',
      dieuKien: ['TCOND-CANCEL-05'],
      yeuCau: ['R-CANCEL-01', 'R-CANCEL-02'],
    });
  });
});
