import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const flutter = [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']];
const dart = [['pub', 'get'], ['analyze'], ['test']];
// [thư mục lab, lệnh, các bước]
const labs = [
  ['50-flavors-build', 'flutter', flutter],
  ['51-android-signing', 'dart', dart],
  ['52-ios-signing', 'dart', dart],
  ['53-fastlane', 'dart', dart],
  ['54-ci-cd-build', 'dart', dart],
  ['55-app-distribution', 'dart', dart],
  ['56-testflight', 'dart', dart],
  ['57-google-play', 'dart', dart],
  ['58-app-store-release', 'dart', dart],
  ['59-post-release-ops', 'flutter', flutter],
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
console.log('\nPASS: Flutter articles 50–59 labs analyze and test.');
