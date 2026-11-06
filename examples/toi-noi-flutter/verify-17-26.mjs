import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const flutter = [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']];
const dart = [['pub', 'get'], ['analyze'], ['test']];
// [thư mục lab, lệnh, các bước]
const labs = [
  ['packages/toi_noi_api', 'dart', dart],
  ['tools/fake_api', 'dart', dart],
  ['17-state-changenotifier', 'flutter', flutter],
  ['18-riverpod', 'flutter', flutter],
  ['19-http-dio-json', 'flutter', flutter],
  ['20-error-retry-connectivity', 'flutter', flutter],
  ['21-clean-architecture', 'flutter', flutter],
  ['22-persistence-drift', 'flutter', flutter],
  ['23-go-router', 'flutter', flutter],
  ['24-auth-jwt-refresh', 'flutter', flutter],
  ['25-flavors-config', 'flutter', flutter],
  ['26-codegen', 'flutter', flutter],
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
console.log('\nPASS: Flutter articles 17–26 labs analyze and test.');
