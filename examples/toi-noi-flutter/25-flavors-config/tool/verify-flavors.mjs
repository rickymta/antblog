// Chạy bộ test của lab 25 theo từng cách cấp cấu hình.
//
//   node examples/toi-noi-flutter/25-flavors-config/tool/verify-flavors.mjs
//
// Cần Flutter 3.47 trên PATH. Không cần simulator, emulator hay mạng.
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const lab = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const flavors = ['dev', 'staging', 'prod'];

function flutter(args, { capture = false } = {}) {
  console.log(`\n> flutter ${args.join(' ')}`);
  const result = spawnSync('flutter', args, {
    cwd: lab,
    encoding: 'utf8',
    stdio: capture ? 'pipe' : 'inherit',
    timeout: 600_000,
  });
  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }
  return result;
}

function mustPass(args) {
  const { status } = flutter(args);
  if (status !== 0) process.exit(status ?? 1);
}

// Chạy phải thất bại và đầu ra phải chứa `expected`: đây là phép thử "fail sớm".
function mustFail(args, expected) {
  const { status, stdout = '', stderr = '' } = flutter(args, { capture: true });
  const output = stdout + stderr;
  if (status === 0 || !output.includes(expected)) {
    console.error(output);
    console.error(`FAIL: lệnh trên phải thất bại với thông báo chứa "${expected}".`);
    process.exit(1);
  }
  console.log(`(đã thất bại như mong đợi: "${expected}")`);
}

mustPass(['pub', 'get']);
mustPass(['analyze', '--no-pub']);

// 1. Không define: nhóm "chạy không có define" chạy, nhóm kia bị bỏ qua.
mustPass(['test', '--no-pub']);

// 2. Mỗi flavor: file cấu hình + --flavor cùng tên.
for (const flavor of flavors) {
  mustPass(['test', '--no-pub', '--flavor', flavor, `--dart-define-from-file=config/${flavor}.json`]);
}

// 3. Chỉ có file, không có --flavor (cách chạy của lab khi chưa có flavor native).
mustPass(['test', '--no-pub', '--dart-define-from-file=config/staging.json']);

// 4. File dev nhưng --flavor prod: AppConfig phải từ chối ngay.
mustFail(
  ['test', '--no-pub', '--flavor', 'prod', '--dart-define-from-file=config/dev.json'],
  'hai nguồn này không khớp',
);

// 5. Công cụ không cho tự đặt khoá FLUTTER_APP_FLAVOR bằng define.
mustFail(
  ['test', '--no-pub', '--dart-define=FLUTTER_APP_FLAVOR=prod'],
  'FLUTTER_APP_FLAVOR is used by the framework',
);

console.log('\nPASS: lab 25 analyze sạch, test xanh ở mọi cách cấp cấu hình.');
