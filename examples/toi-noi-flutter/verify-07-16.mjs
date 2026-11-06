import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const flutter = [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']];
const dart = [['pub', 'get'], ['analyze'], ['test']];
// [thư mục lab, lệnh, các bước]
const labs = [
  ['packages/toi_noi_ui', 'flutter', flutter],
  ['07-widget-man-hinh-bao-gia', 'flutter', flutter],
  ['08-layout-constraints', 'flutter', flutter],
  ['09-listview-sliver', 'flutter', flutter],
  ['10-stateful-lifecycle', 'flutter', flutter],
  ['11-form-validation', 'flutter', flutter],
  ['12-navigator-route', 'flutter', flutter],
  ['13-theme-design-system', 'flutter', flutter],
  ['14-asset-intl-l10n', 'flutter', flutter],
  ['15-animation-co-ban', 'flutter', flutter],
  ['16-a11y-responsive-adaptive', 'flutter', flutter],
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
console.log('\nPASS: Flutter articles 07–16 labs analyze and test.');
