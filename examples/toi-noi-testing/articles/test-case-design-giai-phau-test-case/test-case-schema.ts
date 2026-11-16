/**
 * TCD-02 — cấu trúc một tài liệu test case, viết bằng zod.
 * Schema chỉ bảo đảm test case ĐÚNG DẠNG (đủ trường, đúng kiểu, đúng quy ước mã).
 * Test case VIẾT TỐT hay không là việc của bộ lint trong `lint-test-case.ts`.
 */
import { z } from 'zod';

export const PRIORITIES = ['cao', 'trung-binh', 'thap'] as const;

const requirementCode = z.string().regex(/^R-[A-Z]+-\d{2}$/, 'mã yêu cầu dạng R-NHOM-nn');
const conditionCode = z.string().regex(/^TCOND-[A-Z]+-\d{2}$/, 'mã test condition dạng TCOND-NHOM-nn');

export const testDataRowSchema = z.object({
  id: z.string().regex(/^TD-\d{2}$/, 'mã dòng dữ liệu dạng TD-nn'),
  description: z.string().min(1),
  values: z.record(z.string(), z.union([z.string(), z.number(), z.boolean(), z.null()])),
});

export const stepSchema = z.object({
  /** Một hành động người chạy (hoặc script) thực hiện được — không chứa phán đoán đúng/sai. */
  action: z.string().min(5),
  /** Dòng test data mà bước dùng, nếu có. */
  data: z.string().regex(/^TD-\d{2}$/).optional(),
});

export const testCaseSchema = z.object({
  id: z.string().regex(/^TC-[A-Z]+-\d{3}$/, 'mã test case dạng TC-NHOM-nnn'),
  title: z.string().min(10).max(120),
  /** Đúng MỘT mục tiêu: test case này chứng minh điều gì. */
  objective: z.string().min(10),
  priority: z.enum(PRIORITIES),
  traceability: z.object({
    requirements: z.array(requirementCode).min(1),
    conditions: z.array(conditionCode),
  }),
  /** Phiên bản test basis mà kết quả mong đợi được suy ra. */
  basisVersion: z.string().min(1),
  environment: z.string().min(1),
  preconditions: z.array(z.string().min(1)),
  testData: z.array(testDataRowSchema),
  steps: z.array(stepSchema).min(1),
  expectedResults: z.array(z.string().min(1)).min(1),
  postconditions: z.array(z.string().min(1)),
});

export type TestCaseDoc = z.infer<typeof testCaseSchema>;
