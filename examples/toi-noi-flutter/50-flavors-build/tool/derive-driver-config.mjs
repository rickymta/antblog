// Sinh native-config/driver từ native-config/customer: app tài xế dùng đúng khuôn của app khách,
// chỉ khác ID gốc, tên hiển thị và id app Firebase giả. Ghi đè file có sẵn, không đụng icon PNG
// (icon do make-placeholder-icons.mjs sinh cho cả hai app).
//
//   node examples/toi-noi-flutter/50-flavors-build/tool/derive-driver-config.mjs
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const lab = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const from = path.join(lab, 'native-config/customer');
const to = path.join(lab, 'native-config/driver');
const contract = JSON.parse(fs.readFileSync(path.join(lab, 'flavors.json'), 'utf8'));
const names = contract.apps.driver.displayNames;

function walk(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
  return out;
}

function derive(relative, text) {
  const flavor = /(?:src\/|Flavors\/)(dev|staging|prod)[/.]/.exec(relative)?.[1];
  let out = text
    .replaceAll('vn.toinoi.customer', 'vn.toinoi.driver')
    .replaceAll('app khách', 'app tài xế')
    .replace(/FAKE-(DEV|STAGING|PROD)-REPLACE-ME/g, 'FAKE-DRIVER-$1-REPLACE-ME');
  if (relative.endsWith('Info.plist')) out = out.replace('<string>customer</string>', '<string>driver</string>');
  if (flavor && relative.endsWith('strings.xml')) {
    out = out.replace(/(<string name="app_name">)[^<]*(<\/string>)/, `$1${names[flavor]}$2`);
  }
  if (flavor && relative.includes('Flavors/')) {
    out = out.replace(/^APP_DISPLAY_NAME = .*$/m, `APP_DISPLAY_NAME = ${names[flavor]}`);
  }
  return out;
}

for (const file of walk(from)) {
  if (file.endsWith('.png')) continue;
  const relative = path.relative(from, file);
  const target = path.join(to, relative);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, derive(relative, fs.readFileSync(file, 'utf8')));
  fs.chmodSync(target, fs.statSync(file).mode);
  console.log(path.relative(lab, target));
}
