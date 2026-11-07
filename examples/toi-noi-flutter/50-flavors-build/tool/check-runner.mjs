// Kiểm project.pbxproj của runner iOS SAU KHI đã tạo scheme và configuration trong Xcode.
//
//   node tool/check-runner.mjs <đường dẫn tới ios/Runner.xcodeproj/project.pbxproj>
//
// Chỉ xét khối configuration của TARGET Runner (khối có INFOPLIST_FILE = Runner/Info.plist),
// không xét khối cùng tên ở mức project hay của target RunnerTests. Với mỗi trong chín
// configuration <Debug|Profile|Release>-<flavor>:
//   1. phải có khối của target Runner, gắn đúng file <tên>.xcconfig làm base;
//   2. buildSettings của chính nó không được đặt PRODUCT_BUNDLE_IDENTIFIER hay
//      ASSETCATALOG_COMPILER_APPICON_NAME, vì Xcode áp settings của target SAU xcconfig,
//      nên dòng đó thắng và giá trị trong Flavors/<flavor>.xcconfig bị bỏ qua.
// Chỉ đọc file. Thoát 1 và in từng vấn đề; 0 khi sạch.
import fs from 'node:fs';

const file = process.argv[2];
if (!file || !fs.existsSync(file)) {
  console.error('Dùng: check-runner.mjs <project.pbxproj>');
  process.exit(2);
}
const text = fs.readFileSync(file, 'utf8');
const flavors = ['dev', 'staging', 'prod'];
const modes = ['Debug', 'Profile', 'Release'];
const overridable = ['PRODUCT_BUNDLE_IDENTIFIER', 'ASSETCATALOG_COMPILER_APPICON_NAME'];

// Mỗi khối XCBuildConfiguration: tên, file xcconfig base (nếu có), nội dung buildSettings.
const configs = [];
for (const m of text.matchAll(/isa = XCBuildConfiguration;([\s\S]*?)\n\t\t\};/g)) {
  const body = m[1];
  const name = /\n\t\t\tname = "?([^";\n]+)"?;/.exec(body)?.[1];
  const base = /baseConfigurationReference = \w+ \/\* ([^*]+?) \*\//.exec(body)?.[1] ?? null;
  const settings = /buildSettings = \{([\s\S]*?)\n\t\t\t\};/.exec(body)?.[1] ?? '';
  const isRunnerTarget = /\n\t*INFOPLIST_FILE = "?Runner\/Info\.plist"?;/.test(settings);
  configs.push({ name, base, settings, isRunnerTarget });
}

const problems = [];
for (const flavor of flavors) {
  for (const mode of modes) {
    const name = `${mode}-${flavor}`;
    const matching = configs.filter((c) => c.name === name && c.isRunnerTarget);
    if (matching.length === 0) {
      problems.push(`target Runner không có configuration ${name} (cần gắn ${name}.xcconfig)`);
      continue;
    }
    for (const config of matching) {
      if (config.base !== `${name}.xcconfig`) {
        problems.push(`${name}: target Runner có base ${config.base ?? '(không có)'}, cần ${name}.xcconfig`);
      }
      for (const key of overridable) {
        const line = new RegExp(`\\n\\t*${key} = ([^;]+);`).exec(config.settings);
        if (line && line[1].trim() !== '"$(inherited)"' && line[1].trim() !== '$(inherited)') {
          problems.push(`${name}: target đặt ${key} = ${line[1].trim()}, ghi đè xcconfig; xoá dòng này trong Build Settings của target Runner`);
        }
      }
    }
  }
}

if (problems.length === 0) {
  console.log(`OK: chín configuration ${modes.join('/')}-${flavors.join('|')} đều gắn đúng xcconfig và không bị target ghi đè.`);
  process.exit(0);
}
console.error(`${problems.length} vấn đề:`);
for (const p of problems) console.error(`  - ${p}`);
process.exit(1);
