// Chép file cấu hình trong native-config/ vào runner do `flutter create` sinh.
//
//   flutter create --platforms=ios,android --project-name toi_noi_31 --org vn.toinoi .
//   node tool/apply-native-config.mjs
//
// `flutter create .` còn sinh test/widget_test.dart (đếm bộ đếm của MyApp, app này
// không có MyApp); script xoá nó nếu vẫn đúng là bản template.
//
// Android: chép đủ (build.gradle.kts, hai AndroidManifest, network_security_config,
// MainActivityTest.java), ghi đè file cùng tên của template.
// iOS: chỉ chép RunnerUITests.m khi target RunnerUITests đã được tạo bằng Xcode
// (project.pbxproj không chép tự động được); đoạn ATS trong Info.plist phải dán tay.
import { cpSync, existsSync, readdirSync, readFileSync, rmSync, statSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const lab = join(dirname(fileURLToPath(import.meta.url)), '..');
const source = join(lab, 'native-config');

function files(dir) {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : [path];
  });
}

function copyTree(subdir, { skip = () => false } = {}) {
  for (const from of files(join(source, subdir))) {
    const rel = relative(source, from);
    if (skip(rel)) continue;
    const to = join(lab, rel);
    cpSync(from, to, { recursive: true });
    console.log(`đã chép ${rel}`);
  }
}

if (!existsSync(join(lab, 'android'))) {
  console.error('Chưa có android/: chạy `flutter create` trước (xem README).');
  process.exit(1);
}

const template = join(lab, 'test', 'widget_test.dart');
if (
  existsSync(template) &&
  readFileSync(template, 'utf8').includes('Counter increments smoke test')
) {
  rmSync(template);
  console.log('đã xoá test/widget_test.dart do flutter create sinh');
}

copyTree('android');

if (existsSync(join(lab, 'ios', 'RunnerUITests'))) {
  copyTree('ios', { skip: (rel) => !rel.endsWith('RunnerUITests.m') });
} else {
  console.log('bỏ qua iOS: chưa có target RunnerUITests (tạo bằng Xcode, xem README).');
}
console.log(
  'Còn lại, làm tay: đoạn NSAppTransportSecurity trong ' +
    'native-config/ios/Runner/Info.plist.ats-fragment.xml dán vào ios/Runner/Info.plist.',
);
