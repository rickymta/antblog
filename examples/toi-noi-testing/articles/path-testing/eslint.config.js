// TEST-048 · ESLint 9 CHỈ để đo cyclomatic complexity (luật `complexity`) — không cần thông tin kiểu.
// Chạy từ gốc lab: npx eslint -c articles/path-testing/eslint.config.js --rule 'complexity: ["warn", 1]' <tệp>
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

export default defineConfig({
  files: ['**/*.ts'],
  languageOptions: { parser: tseslint.parser },
  rules: { complexity: ['warn', 10] },
});
