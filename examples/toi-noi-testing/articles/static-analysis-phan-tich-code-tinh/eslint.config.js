// TEST-029 · cấu hình ESLint 9 (flat config) CHỈ cho bài này.
// Chạy từ gốc lab: npx eslint -c articles/static-analysis-phan-tich-code-tinh/eslint.config.js <tệp>
import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default defineConfig({
  files: ['**/*.ts', '**/*.mts'],
  extends: [js.configs.recommended, tseslint.configs.recommendedTypeChecked],
  languageOptions: {
    parserOptions: {
      project: './tsconfig.json', // tsconfig của bài (gồm cả tệp .mts)
      tsconfigRootDir: import.meta.dirname,
    },
  },
  rules: {
    // Control flow: độ phức tạp chu trình (McCabe, ESLint đếm cả && || ??), mã không tới được
    complexity: ['warn', 10],
    'no-unreachable': 'error',
    // Data flow: gán rồi gán đè không dùng (define–define), biến định nghĩa không dùng
    'no-useless-assignment': 'error',
    '@typescript-eslint/no-unused-vars': 'error',
    // Điều kiện luôn đúng/luôn sai theo kiểu (cần thông tin kiểu)
    '@typescript-eslint/no-unnecessary-condition': 'error',
    // Quy ước mã (coding standard) — ví dụ một luật "phong cách"
    'no-console': 'warn',
    // Luật cố tình để chặt (cấm số trong template string) — minh hoạ một cảnh báo giả trên mã đúng
    '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: false }],
  },
});
