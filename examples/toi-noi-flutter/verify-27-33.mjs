import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const flutter = [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']];
const dart = [['pub', 'get'], ['analyze'], ['test']];
// [thư mục lab, lệnh, các bước]
const labs = [
  ['27-unit-test-mocktail', 'dart', dart],
  ['28-widget-test', 'flutter', flutter],
  ['29-golden-semantics', 'flutter', flutter],
  ['30-test-riverpod-async', 'flutter', flutter],
  ['31-integration-test-patrol', 'flutter', flutter],
  ['32-lint-analyzer', 'flutter', flutter],
  ['32-lint-analyzer/lint_plugin', 'dart', dart],
  ['33-ci-github-actions', 'dart', dart],
];

for (const [dir, tool, steps] of labs) {
  for (const args of steps) {
    console.log(`\n> [${dir}] ${tool} ${args.join(' ')}`);
    const result = spawnSync(tool, args, { cwd: path.join(base, dir), stdio: 'inherit', timeout: 600_000 });
    if (result.error) {
      console.error(result.error.message);
      process.exit(1);
    }
    if (result.status !== 0) process.exit(result.status ?? 1);
  }
}
console.log('\nPASS: Flutter articles 27–33 labs analyze and test.');
