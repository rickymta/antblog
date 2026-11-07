// Kiểm cấu hình flavor trong native-config/ mà không cần Gradle, Xcode hay Flutter.
//
//   node examples/toi-noi-flutter/50-flavors-build/tool/verify-flavors.mjs
//   node .../verify-flavors.mjs --root <thư mục có flavors.json, native-config/, config/>
//
// Đọc `flavors.json` (bảng ID của curriculum mục 3.2) làm hợp đồng, rồi đối chiếu từng file.
// Thoát với mã 1 và in mọi vấn đề tìm được; mã 0 khi sạch. Chỉ đọc file, không ghi gì.
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const args = process.argv.slice(2);
const rootFlag = args.indexOf('--root');
const root =
  rootFlag >= 0
    ? path.resolve(args[rootFlag + 1])
    : path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');

const problems = [];
const seenIds = new Map();
const fail = (message) => problems.push(message);
const rel = (file) => path.relative(root, file);

function read(file) {
  if (!fs.existsSync(file)) {
    fail(`thiếu file ${rel(file)}`);
    return null;
  }
  return fs.readFileSync(file, 'utf8');
}

// ---------- hợp đồng ----------
const contractText = read(path.join(root, 'flavors.json'));
if (contractText === null) report();
const contract = JSON.parse(contractText);
const flavors = contract.flavors;
const apps = Object.keys(contract.apps);

if (flavors.join() !== 'dev,staging,prod') {
  fail(`flavors.json phải liệt kê đúng dev, staging, prod; đang là ${flavors.join(', ')}`);
}

// Không hai (app, flavor) nào dùng chung một ID, không hai flavor của một app dùng chung tên.
for (const app of apps) {
  const { ids, displayNames } = contract.apps[app];
  const seenNames = new Map();
  for (const flavor of flavors) {
    const id = ids[flavor];
    if (seenIds.has(id)) fail(`ID ${id} trùng giữa ${seenIds.get(id)} và ${app}/${flavor}`);
    seenIds.set(id, `${app}/${flavor}`);
    const name = displayNames[flavor];
    if (seenNames.has(name)) fail(`${app}: tên app "${name}" trùng giữa ${seenNames.get(name)} và ${flavor}`);
    seenNames.set(name, flavor);
  }
}

const nativeRoot = path.join(root, 'native-config');

// ---------- Android: build.gradle.kts ----------
function parseGradle(text) {
  const result = { applicationId: null, dimensions: [], flavors: {} };
  result.applicationId = /applicationId\s*=\s*"([^"]+)"/.exec(text)?.[1] ?? null;
  const dims = /flavorDimensions\s*\+=\s*"([^"]+)"/.exec(text);
  if (dims) result.dimensions.push(dims[1]);
  const block = /productFlavors\s*\{([\s\S]*?)\n    \}/.exec(text)?.[1] ?? '';
  for (const m of block.matchAll(/create\("([^"]+)"\)\s*\{([^}]*)\}/g)) {
    result.flavors[m[1]] = {
      dimension: /dimension\s*=\s*"([^"]+)"/.exec(m[2])?.[1] ?? null,
      suffix: /applicationIdSuffix\s*=\s*"([^"]*)"/.exec(m[2])?.[1] ?? '',
    };
  }
  return result;
}

for (const app of apps) {
  const gradleFile = path.join(nativeRoot, app, 'android/app/build.gradle.kts');
  const text = read(gradleFile);
  if (text === null) continue;
  const gradle = parseGradle(text);
  const where = rel(gradleFile);
  if (/resValue\s*\(/.test(text)) {
    fail(`${where}: dùng resValue(...); AGP 9 tắt resValues mặc định, đặt app_name trong src/<flavor>/res/values/strings.xml`);
  }
  if (gradle.dimensions.length !== 1) fail(`${where}: cần đúng một flavorDimensions`);
  const names = Object.keys(gradle.flavors);
  const gradleIds = new Map();
  for (const flavor of flavors) {
    const spec = gradle.flavors[flavor];
    if (!spec) {
      fail(`${where}: thiếu productFlavors "${flavor}"`);
      continue;
    }
    if (spec.dimension !== gradle.dimensions[0]) {
      fail(`${where}: flavor ${flavor} có dimension ${spec.dimension}, khác flavorDimensions ${gradle.dimensions[0]}`);
    }
    const id = (gradle.applicationId ?? '') + spec.suffix;
    if (gradleIds.has(id)) fail(`${where}: flavor ${gradleIds.get(id)} và ${flavor} cùng ra application ID ${id} (trùng, không cài cạnh nhau được)`);
    gradleIds.set(id, flavor);
    const expected = contract.apps[app].ids[flavor];
    if (id !== expected) fail(`${where}: flavor ${flavor} ra application ID ${id}, hợp đồng là ${expected}`);
  }
  for (const name of names) {
    if (!flavors.includes(name)) fail(`${where}: flavor lạ "${name}"`);
  }

  // Tên app theo flavor.
  for (const flavor of flavors) {
    const strings = path.join(nativeRoot, app, `android/app/src/${flavor}/res/values/strings.xml`);
    const xml = read(strings);
    if (xml === null) continue;
    const name = /<string name="app_name">([^<]*)<\/string>/.exec(xml)?.[1];
    const expected = contract.apps[app].displayNames[flavor];
    if (name !== expected) fail(`${rel(strings)}: app_name là "${name}", hợp đồng là "${expected}"`);
  }
}

// Manifest dùng nhãn và icon theo tài nguyên, không viết cứng.
for (const app of apps) {
  const file = path.join(nativeRoot, app, 'android/app/src/main/AndroidManifest.xml');
  const text = read(file);
  if (text !== null) {
    if (!text.includes('android:label="@string/app_name"')) fail(`${rel(file)}: android:label phải là @string/app_name`);
    if (!text.includes('android:icon="@mipmap/ic_launcher"')) fail(`${rel(file)}: android:icon phải là @mipmap/ic_launcher`);
  }
}

// Mỗi app có đủ các file cấu hình dùng chung (build.gradle.kts áp plugin google-services nên
// settings.gradle.kts phải khai nó; iOS cần script chép plist và hai xcconfig gốc).
for (const app of apps) {
  const settings = read(path.join(nativeRoot, app, 'android/settings.gradle.kts'));
  if (settings !== null && !settings.includes('id("com.google.gms.google-services")')) {
    fail(`${app}/android/settings.gradle.kts: thiếu plugin com.google.gms.google-services`);
  }
  for (const file of ['ios/scripts/copy-google-service-info.sh', 'ios/Flutter/Debug.xcconfig', 'ios/Flutter/Release.xcconfig']) {
    read(path.join(nativeRoot, app, file));
  }
}

// ---------- iOS: xcconfig, Info.plist, AppIcon ----------
function parseXcconfig(text) {
  const settings = {};
  const includes = [];
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (line === '' || line.startsWith('//')) continue;
    const inc = /^#include\??\s+"([^"]+)"$/.exec(line);
    if (inc) {
      includes.push(inc[1]);
      continue;
    }
    const kv = /^([A-Z0-9_]+)\s*=\s*(.*)$/.exec(line);
    if (kv) settings[kv[1]] = kv[2];
  }
  return { settings, includes };
}

const baseOf = { Debug: 'Debug.xcconfig', Profile: 'Release.xcconfig', Release: 'Release.xcconfig' };
const iosIconSets = new Map(); // `${app}/${flavor}` -> tên bộ AppIcon
for (const app of apps) {
  const flutterDir = path.join(nativeRoot, app, 'ios/Flutter');
  for (const flavor of flavors) {
    const flavorFile = path.join(flutterDir, 'Flavors', `${flavor}.xcconfig`);
    const flavorText = read(flavorFile);
    if (flavorText !== null) {
      const { settings } = parseXcconfig(flavorText);
      const expectId = contract.apps[app].ids[flavor];
      const expectName = contract.apps[app].displayNames[flavor];
      if (settings.PRODUCT_BUNDLE_IDENTIFIER !== expectId) {
        fail(`${rel(flavorFile)}: PRODUCT_BUNDLE_IDENTIFIER là ${settings.PRODUCT_BUNDLE_IDENTIFIER}, hợp đồng là ${expectId}`);
      }
      if (settings.APP_DISPLAY_NAME !== expectName) {
        fail(`${rel(flavorFile)}: APP_DISPLAY_NAME là "${settings.APP_DISPLAY_NAME}", hợp đồng là "${expectName}"`);
      }
      if (settings.TN_FLAVOR !== flavor) fail(`${rel(flavorFile)}: TN_FLAVOR phải là ${flavor}`);
      const icon = settings.ASSETCATALOG_COMPILER_APPICON_NAME;
      const expectIcon = flavor === 'prod' ? 'AppIcon' : `AppIcon-${flavor}`;
      if (icon !== expectIcon) fail(`${rel(flavorFile)}: ASSETCATALOG_COMPILER_APPICON_NAME là ${icon}, cần ${expectIcon}`);
      iosIconSets.set(`${app}/${flavor}`, icon);
    }
    for (const [mode, base] of Object.entries(baseOf)) {
      const file = path.join(flutterDir, `${mode}-${flavor}.xcconfig`);
      const text = read(file);
      if (text === null) continue;
      const { includes } = parseXcconfig(text);
      const want = [base, `Flavors/${flavor}.xcconfig`];
      if (includes.join('|') !== want.join('|')) {
        fail(`${rel(file)}: #include phải là ${want.join(' rồi ')}; đang là ${includes.join(' rồi ') || '(không có)'}`);
      }
    }
  }
}

for (const app of apps) {
  const file = path.join(nativeRoot, app, 'ios/Runner/Info.plist');
  const text = read(file);
  if (text !== null) {
    const value = (key) =>
      new RegExp(`<key>${key}</key>\\s*<string>([^<]*)</string>`).exec(text)?.[1];
    if (value('CFBundleDisplayName') !== '$(APP_DISPLAY_NAME)') fail(`${rel(file)}: CFBundleDisplayName phải là $(APP_DISPLAY_NAME)`);
    if (value('CFBundleIdentifier') !== '$(PRODUCT_BUNDLE_IDENTIFIER)') fail(`${rel(file)}: CFBundleIdentifier phải là $(PRODUCT_BUNDLE_IDENTIFIER)`);
  }
}

// ---------- icon: tồn tại, là PNG đúng cỡ, ba flavor khác nhau ----------
function pngInfo(file) {
  if (!fs.existsSync(file)) {
    fail(`thiếu icon ${rel(file)}`);
    return null;
  }
  const buf = fs.readFileSync(file);
  const signature = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  if (buf.length < 24 || !buf.subarray(0, 8).equals(signature)) {
    fail(`${rel(file)} không phải PNG`);
    return null;
  }
  return {
    width: buf.readUInt32BE(16),
    height: buf.readUInt32BE(20),
    hash: crypto.createHash('sha256').update(buf).digest('hex'),
  };
}
function mustDiffer(label, byFlavor) {
  const seen = new Map();
  for (const [flavor, hash] of byFlavor) {
    if (seen.has(hash)) fail(`${label}: icon của ${seen.get(hash)} và ${flavor} giống hệt nhau`);
    seen.set(hash, flavor);
  }
}

for (const app of apps) {
  const densities = { mdpi: 48, hdpi: 72, xhdpi: 96, xxhdpi: 144, xxxhdpi: 192 };
  for (const [density, size] of Object.entries(densities)) {
    const hashes = [];
    for (const flavor of flavors) {
      // prod dùng icon của main; dev và staging ghi đè ở src/<flavor>.
      const sourceSet = flavor === 'prod' ? 'main' : flavor;
      const file = path.join(nativeRoot, app, `android/app/src/${sourceSet}/res/mipmap-${density}/ic_launcher.png`);
      const info = pngInfo(file);
      if (!info) continue;
      if (info.width !== size || info.height !== size) {
        fail(`${rel(file)}: ${info.width}x${info.height}, mipmap-${density} cần ${size}x${size}`);
      }
      hashes.push([flavor, info.hash]);
    }
    mustDiffer(`${app}: Android mipmap-${density}`, hashes);
  }

  const hashes = [];
  for (const flavor of flavors) {
    const set = iosIconSets.get(`${app}/${flavor}`);
    if (!set) continue;
    const dir = path.join(nativeRoot, app, 'ios/Runner/Assets.xcassets', `${set}.appiconset`);
    const contents = read(path.join(dir, 'Contents.json'));
    if (contents === null) continue;
    const image = JSON.parse(contents).images?.[0]?.filename;
    if (!image) {
      fail(`${rel(dir)}/Contents.json không khai báo filename`);
      continue;
    }
    const info = pngInfo(path.join(dir, image));
    if (!info) continue;
    if (info.width !== 1024 || info.height !== 1024) fail(`${rel(dir)}/${image}: cần 1024x1024`);
    hashes.push([flavor, info.hash]);
  }
  mustDiffer(`${app}: iOS AppIcon`, hashes);
}

// ---------- Firebase: chỉ bản .example, giá trị giả, khớp ID của flavor ----------
const realKeyPatterns = [
  /AIza[0-9A-Za-z_-]{35}/, // Google API key
  /-----BEGIN [A-Z ]*PRIVATE KEY-----/,
  /\b\d{12}-[a-z0-9]{20,}\.apps\.googleusercontent\.com\b/,
];
function scanForSecrets(file, text) {
  for (const pattern of realKeyPatterns) {
    if (pattern.test(text)) fail(`${rel(file)}: có chuỗi giống khoá thật (${pattern.source.slice(0, 18)}…)`);
  }
}

function walk(dir, out = []) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, out);
    else out.push(full);
  }
  return out;
}
for (const file of walk(nativeRoot)) {
  const base = path.basename(file);
  if (base === 'google-services.json' || base === 'GoogleService-Info.plist') {
    fail(`${rel(file)}: file Firebase thật không được nằm trong repo, đổi đuôi thành .example`);
  }
}

const androidAppIds = new Map();
for (const app of apps) {
  for (const flavor of flavors) {
    const expected = contract.apps[app].ids[flavor];

    const jsonFile = path.join(nativeRoot, app, `android/app/src/${flavor}/google-services.json.example`);
    const jsonText = read(jsonFile);
    if (jsonText !== null) {
      scanForSecrets(jsonFile, jsonText);
      const json = JSON.parse(jsonText);
      const clients = json.client ?? [];
      const names = clients.map((c) => c.client_info?.android_client_info?.package_name);
      if (!names.includes(expected)) {
        fail(`${rel(jsonFile)}: không có client package_name ${expected} (có: ${names.join(', ')})`);
      }
      for (const c of clients) {
        const appId = c.client_info?.mobilesdk_app_id;
        if (androidAppIds.has(appId)) fail(`${rel(jsonFile)}: mobilesdk_app_id ${appId} trùng với ${androidAppIds.get(appId)}`);
        androidAppIds.set(appId, `${app}/${flavor}`);
        for (const key of c.api_key ?? []) {
          if (!/^FAKE_/.test(key.current_key)) fail(`${rel(jsonFile)}: current_key phải là giá trị giả bắt đầu bằng FAKE_`);
        }
      }
    }

    const plistFile = path.join(nativeRoot, app, `ios/Runner/Firebase/${flavor}/GoogleService-Info.plist.example`);
    const plistText = read(plistFile);
    if (plistText !== null) {
      scanForSecrets(plistFile, plistText);
      const value = (key) => new RegExp(`<key>${key}</key>\\s*<string>([^<]*)</string>`).exec(plistText)?.[1];
      if (value('BUNDLE_ID') !== expected) fail(`${rel(plistFile)}: BUNDLE_ID là ${value('BUNDLE_ID')}, cần ${expected}`);
      if (!/^FAKE_/.test(value('API_KEY') ?? '')) fail(`${rel(plistFile)}: API_KEY phải là giá trị giả bắt đầu bằng FAKE_`);
    }
  }
}

// ---------- file cấu hình Dart khớp tên flavor ----------
for (const flavor of flavors) {
  const file = path.join(root, 'config', `${flavor}.json`);
  const text = read(file);
  if (text === null) continue;
  scanForSecrets(file, text);
  const json = JSON.parse(text);
  if (json.FLAVOR !== flavor) fail(`${rel(file)}: FLAVOR là ${json.FLAVOR}, phải trùng tên file (${flavor})`);
}

report();

function report() {
  if (problems.length === 0) {
    console.log(`OK: ${flavors.length} flavor x ${apps.length} app đều khớp hợp đồng (${seenIds.size} ID khác nhau).`);
    process.exit(0);
  }
  console.error(`${problems.length} vấn đề:`);
  for (const problem of problems) console.error(`  - ${problem}`);
  process.exit(1);
}
