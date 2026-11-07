import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const base = path.dirname(fileURLToPath(import.meta.url));
const flutter = [['pub', 'get'], ['analyze', '--no-pub'], ['test', '--no-pub']];
const dart = [['pub', 'get'], ['analyze'], ['test']];
// [thư mục lab, lệnh, các bước]
const labs = [
  ['34-map-flutter-map', 'flutter', flutter],
  ['35-location-permission', 'flutter', flutter],
  ['36-realtime-websocket', 'flutter', flutter],
  ['37-push-notification', 'flutter', flutter],
  ['38-background-work', 'flutter', flutter],
  ['39-payment-flow', 'flutter', flutter],
  ['40-platform-channel-pigeon', 'flutter', flutter],
  ['40-platform-channel-pigeon/native', 'dart', dart],
  ['41-performance', 'flutter', flutter],
  ['42-mobile-security', 'dart', dart],
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
console.log('\nPASS: Flutter articles 34–42 labs analyze and test.');
