import { Types } from 'mongoose';
import { z } from 'zod';
import { validation } from './errors.js';

// Thông điệp lỗi của zod bằng tiếng Việt.
z.config(z.locales.vi());

/** Chuyển lỗi zod thành `details.fields: { [field]: message }` (mỗi trường giữ thông điệp đầu tiên). */
export function zodFields(error: z.ZodError): Record<string, string> {
  const fields: Record<string, string> = {};
  for (const issue of error.issues) {
    const key = issue.path.length ? issue.path.map(String).join('.') : '_';
    if (!(key in fields)) fields[key] = issue.message;
  }
  return fields;
}

export function parse<S extends z.ZodType>(schema: S, data: unknown): z.output<S> {
  const result = schema.safeParse(data);
  if (!result.success) throw validation(zodFields(result.error));
  return result.data;
}

export function isObjectId(value: unknown): value is string {
  return typeof value === 'string' && /^[a-f0-9]{24}$/i.test(value) && Types.ObjectId.isValid(value);
}

export const objectIdSchema = z.string().refine((v) => isObjectId(v), { message: 'Id không hợp lệ' });

/** Số nguyên lấy từ query string (chuỗi) với khoảng [min, max] và giá trị mặc định. */
export const queryInt = (min: number, max: number, def: number) =>
  z.preprocess(
    (v) => (v === undefined || v === '' ? undefined : typeof v === 'string' ? Number(v) : v),
    z.number().int().min(min).max(max).default(def),
  );

export const pagingSchema = (defaultPageSize = 12) => ({
  page: queryInt(1, 100000, 1),
  pageSize: queryInt(1, 50, defaultPageSize),
});

/** Chuỗi query tuỳ chọn: rỗng ⇒ vắng. */
export const optionalQueryString = (max = 200) =>
  z.preprocess((v) => (typeof v === 'string' && v.trim() === '' ? undefined : v), z.string().trim().max(max).optional());
