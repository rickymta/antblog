// Đặt biến môi trường TRƯỚC khi mã nguồn đọc cấu hình. Test luôn dùng DB riêng và tự dọn.
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterAll } from 'vitest';

process.env.NODE_ENV = 'test';
process.env.MONGODB_URI = process.env.TEST_MONGODB_URI ?? 'mongodb://127.0.0.1:27017/tech_knowledge_hub_test';
process.env.JWT_SECRET = 'test-secret-only-for-automated-tests-0123456789abcdef0123456789';

// Ảnh tải lên trong test ghi vào thư mục tạm riêng của từng file test, xoá sạch khi file chạy xong.
const uploadDir = mkdtempSync(join(tmpdir(), 'tkh-uploads-test-'));
process.env.UPLOAD_DIR = uploadDir;
afterAll(() => rmSync(uploadDir, { recursive: true, force: true }));
