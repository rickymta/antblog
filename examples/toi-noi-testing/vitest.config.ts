import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/**/*.test.ts', 'articles/**/*.test.ts'],
    exclude: ['**/node_modules/**', 'articles/**/e2e/**'],
    environment: 'node',
    testTimeout: 20_000,
    coverage: {
      provider: 'v8',
      include: ['src/**/*.ts'],
      exclude: ['src/server.ts'],
      reporter: ['text', 'html', 'json-summary'],
      reportsDirectory: 'reports/coverage',
    },
  },
});
