import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
// [thư mục lab, lệnh, các bước]
const labs = [
  ['01-flutter-va-dart', 'flutter', [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']]],
  ['02-dart-bien-kieu-null-safety', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
  ['03-ham-class-sealed-trang-thai', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
  ['04-collections-generics', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
  ['05-async-stream', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
  ['packages/toi_noi_core', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
  ['06-dung-toi-noi-core', 'dart', [['pub', 'get'], ['analyze'], ['test']]],
];

for (const [dir, tool, steps] of labs) {
  for (const args of steps) {
    console.log(`\n> [${dir}] ${tool} ${args.join(' ')}`);
    const result = spawnSync(tool, args, { cwd: path.join(base, dir), stdio: 'inherit', timeout: 300_000 });
    if (result.error) {
      console.error(result.error.message);
      process.exit(1);
    }
    if (result.status !== 0) process.exit(result.status ?? 1);
  }
}
console.log('\nPASS: Flutter articles 01–06 labs analyze and test.');
