// Ghép `--flavor X` và `--dart-define-from-file=config/X.json` từ MỘT tên flavor,
// để hai cờ không bao giờ lệch nhau.
//
//   node tool/flavor-command.mjs <run|apk|appbundle|ipa> <dev|staging|prod> [--dry-run] [-- cờ thêm]
//
// Ví dụ: node tool/flavor-command.mjs apk staging --dry-run
//   in: flutter build apk --flavor staging --dart-define-from-file=config/staging.json
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const lab = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const flavors = ['dev', 'staging', 'prod'];
const targets = { run: ['run'], apk: ['build', 'apk'], appbundle: ['build', 'appbundle'], ipa: ['build', 'ipa'] };

const argv = process.argv.slice(2);
const dash = argv.indexOf('--');
const own = dash >= 0 ? argv.slice(0, dash) : argv;
const extra = dash >= 0 ? argv.slice(dash + 1) : [];
const dryRun = own.includes('--dry-run');
const [target, flavor] = own.filter((a) => a !== '--dry-run');

if (!(target in targets) || !flavors.includes(flavor)) {
  console.error(`Dùng: flavor-command.mjs <${Object.keys(targets).join('|')}> <${flavors.join('|')}> [--dry-run] [-- cờ thêm]`);
  process.exit(2);
}
const config = `config/${flavor}.json`;
if (!fs.existsSync(path.join(lab, config))) {
  console.error(`Thiếu ${config}`);
  process.exit(2);
}

const args = [...targets[target], '--flavor', flavor, `--dart-define-from-file=${config}`, ...extra];
console.log(`flutter ${args.join(' ')}`);
if (dryRun) process.exit(0);
const result = spawnSync('flutter', args, { cwd: lab, stdio: 'inherit' });
process.exit(result.status ?? 1);
