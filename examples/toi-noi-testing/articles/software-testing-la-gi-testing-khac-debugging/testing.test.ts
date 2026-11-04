/**
 * TESTING (việc của tester): chạy test case trên bản có lỗi để GÂY RA failure và ghi nhận.
 *
 * Mặc định các ca mà bản có lỗi trả sai được đánh dấu `it.fails` (failure đã biết, đã báo)
 * để `npm test` của lab vẫn xanh. Muốn thấy màu đỏ thật như tester thấy lần đầu:
 *   SHOW_RED=1 npx vitest run articles/software-testing-la-gi-testing-khac-debugging
 */
import { describe, expect, it } from 'vitest';
import { calculateFareBuggy } from './buggy-fare';
import { FARE_TEST_CASES } from './test-cases';

const showRed = process.env.SHOW_RED === '1';
// Ca nào bản có lỗi trả sai — đúng như tester ghi lại sau lần chạy đầu tiên.
const KNOWN_FAILURES = new Set(['TC-FARE-01', 'TC-FARE-02']);

describe('Testing · tính cước trên bản build có lỗi', () => {
  for (const tc of FARE_TEST_CASES) {
    const run = KNOWN_FAILURES.has(tc.id) && !showRed ? it.fails : it;
    run(`${tc.id} (${tc.basis}) ${tc.title} ⇒ ${tc.expectedTotal} đ`, () => {
      expect(calculateFareBuggy(tc.input).total).toBe(tc.expectedTotal);
    });
  }
});
