// Lịch sử chuẩn của Phần 5: điều tra lỗi, repository lớn và bảo trì.
//
// Dựng trên cuối Phần 4. Câu chuyện: một buổi sáng có tám commit lên main, trong đó một commit
// "làm gọn" phép làm tròn giá cước gây sai cước (bisect tìm ra, revert); khai báo .gitattributes
// cho diff/merge theo loại file; dùng chung thư viện định dạng tiền tệ bằng submodule; lưu bản thiết
// kế bảng giá bằng Git LFS; ký commit và tag phát hành v1.2.0 bằng khoá SSH; hook chung của nhóm;
// cuối cùng là báo cáo điều tra.
//
// `build5(dir, n)` dựng Phần 1–4 rồi chạy n bước của Phần 5.

import { deflateSync } from 'node:zlib';
import { dirname, join } from 'node:path';
import { mkdirSync, writeFileSync, existsSync, rmSync } from 'node:fs';

import { build4 } from './history4.mjs';

export { NIGHT_DEBUG as NIGHT_DEBUG_LINE } from './history4.mjs';
import { places, refsOf3 } from './history3.mjs';
import { LAB, freshDir, gitQuiet, readText, writeText } from './lab.mjs';

// ---------- Đường dẫn ----------

export const SIGN_KEY = join(LAB, 'fixtures', 'khoa-ky-demo').split('\\').join('/');
export const BUG_SCRIPT = join(LAB, 'p05', 'kiem-tra-lam-tron.sh').split('\\').join('/');

/** Thêm vào places(): thư viện tiền tệ (remote bare và bản làm việc của Lan). */
export function places5(dir) {
  const P = places(dir);
  const root = dirname(dir);
  const slash = (p) => p.split('\\').join('/');
  return { ...P, libRemote: slash(join(root, 'remote', 'tien-te.git')), libWork: join(root, 'thu-vien', 'tien-te') };
}

// ---------- Nội dung ----------

export const ROUND_OK = '        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;';
export const ROUND_BUG = '        return Math.Round(atLeastMinimum / 1000m) * 1000m;';
const BUG = (s) =>
  s.replace('        // Làm tròn lên bội số 1.000 đ để khách không phải trả tiền lẻ.\n', '        // Làm tròn tới bội số 1.000 đ.\n').replace(ROUND_OK, ROUND_BUG);

export const README_NIGHT = (s) =>
  `${s}\nVí dụ chuyến đêm: ô tô 4 chỗ, 4 km, bắt đầu 23:00 → 32.000 đ × 1,2 = 38.400 đ, làm tròn lên 39.000 đ.\n`;
export const DOC_NIGHT_PRICE = (s) => `${s}\nPhụ phí đêm: chuyến bắt đầu từ 22:00 đến trước 5:00 cộng thêm 20%, tính trên giá đã làm tròn.\n`;
export const TEST_SECONDS = (s) =>
  s.replace(
    '    public void Calculate_ToiDa30000() => Assert.Equal(30000m, WaitingFee.Calculate(90));\n}\n',
    '    public void Calculate_ToiDa30000() => Assert.Equal(30000m, WaitingFee.Calculate(90));\n\n    [Fact]\n    public void CalculateFromSeconds_LamTronLenPhut() => Assert.Equal(1000m, WaitingFee.CalculateFromSeconds(301));\n}\n',
  );
export const DOC_TYPO = (s) => s.replace('trước khi xe tới', 'trước lúc xe tới');
export const SETTINGS_NIGHT = (s) =>
  s.replace('  "Fare": {', '  "NightSurcharge": {\n    "Rate": 0.2,\n    "StartHour": 22,\n    "EndHour": 5\n  },\n  "Fare": {');
export const NIGHT_COMMENT = (s) =>
  s.replace('    public const decimal Rate = 0.2m;', '    // Khớp NightSurcharge trong appsettings.json; đổi ở đó trước, rồi sửa hằng số này.\n    public const decimal Rate = 0.2m;');
export const README_TEST = (s) => `${s}\nChạy riêng test tính cước: \`dotnet test --filter FareCalculator\`.\n`;

export const GITATTRIBUTES_P05 = (s) =>
  `${s}\n# Diff hiểu cấu trúc C# (tiêu đề hunk là tên hàm)\n*.cs diff=csharp\n\n# Bảng tính: nhị phân\n*.xlsx binary\n\n# CHANGELOG: hai nhánh cùng thêm dòng thì giữ cả hai, không báo xung đột\nCHANGELOG.md merge=union\n`;

export const LIB_V1 = `namespace ToiNoi.TienTe;

/// <summary>Định dạng số tiền theo kiểu Việt Nam: 31.000 đ.</summary>
public static class Vnd
{
    public static string Format(decimal amount) =>
        string.Format(System.Globalization.CultureInfo.GetCultureInfo("vi-VN"), "{0:N0} đ", amount);
}
`;
export const LIB_V2 = LIB_V1.replace(
  '        string.Format(System.Globalization.CultureInfo.GetCultureInfo("vi-VN"), "{0:N0} đ", amount);\n',
  '        string.Format(System.Globalization.CultureInfo.GetCultureInfo("vi-VN"), "{0:N0} đ", amount);\n\n    /// <summary>Dạng ngắn cho màn hình nhỏ: 31k.</summary>\n    public static string Short(decimal amount) => $"{amount / 1000m:0.#}k";\n',
);

/** Ảnh PNG 600×400 nhiễu giả ngẫu nhiên (LCG cố định): nén không được, luôn ra cùng một file. */
export function pngThietKe() {
  const w = 600;
  const h = 400;
  let x = 20261127;
  const raw = Buffer.alloc((w * 3 + 1) * h);
  for (let y = 0; y < h; y++) {
    raw[y * (w * 3 + 1)] = 0;
    for (let i = 1; i <= w * 3; i++) {
      x = (Math.imul(x, 1103515245) + 12345) >>> 0;
      raw[y * (w * 3 + 1) + i] = x >>> 24;
    }
  }
  const crcTable = Array.from({ length: 256 }, (_, n) => {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    return c >>> 0;
  });
  const crc = (buf) => {
    let c = 0xffffffff;
    for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  };
  const chunk = (type, data) => {
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const td = Buffer.concat([Buffer.from(type), data]);
    const c = Buffer.alloc(4);
    c.writeUInt32BE(crc(td));
    return Buffer.concat([len, td, c]);
  };
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

export const ALLOWED_SIGNERS = (pub) => `minh@toinoi.test namespaces="git" ${pub.trim().split(' ').slice(0, 2).join(' ')}\n`;

export const HOOK_PRE_COMMIT = `#!/bin/sh
# Chặn commit còn dòng in debug (Console.WriteLine) trong phần thay đổi C# đã stage.
if git diff --cached -U0 -- '*.cs' | grep -q '^+.*Console\\.WriteLine'; then
  echo "pre-commit: còn Console.WriteLine trong thay đổi .cs đã stage. Bỏ dòng debug rồi commit lại." >&2
  exit 1
fi
exit 0
`;
export const HOOK_COMMIT_MSG = `#!/bin/sh
# Tiêu đề commit không bắt đầu bằng WIP và dài ít nhất 10 byte.
subject=$(head -n 1 "$1")
case "$subject" in
  WIP*|wip*)
    echo "commit-msg: không commit WIP; viết tiêu đề nói rõ thay đổi." >&2
    exit 1 ;;
esac
if [ \${#subject} -lt 10 ]; then
  echo "commit-msg: tiêu đề quá ngắn, cần ít nhất 10 ký tự." >&2
  exit 1
fi
exit 0
`;
export const HOOK_PRE_PUSH = `#!/bin/sh
# Trước khi push: giá cước phải còn làm tròn lên bội số 1.000 đ (GIT-041),
# rồi chuyển cho Git LFS đẩy file lớn (hook trong .githooks thay cho hook LFS trong .git/hooks).
if ! grep -q 'Math.Ceiling(atLeastMinimum' src/BaoGia/FareCalculator.cs; then
  echo "pre-push: FareCalculator không còn làm tròn lên bội số 1.000 đ." >&2
  exit 1
fi
command -v git-lfs >/dev/null 2>&1 || exit 0
exec git lfs pre-push "$@"
`;

export const REPORT = `# Báo cáo điều tra: sai cước do làm tròn (26/11/2026)

## Hiện tượng
Chuyến ô tô 4 chỗ 3,7 km: 12.000 + 5.000 × 3,7 = 30.500 đ. Bảng giá quy định làm tròn **lên** bội số 1.000 đ,
tức 31.000 đ; app báo 30.000 đ.

## Tìm nguyên nhân
\`git bisect start main v1.1.0\`, rồi \`git bisect run sh kiem-tra-lam-tron.sh\` (thực tế: \`dotnet test --filter FareCalculator\`).
Commit lỗi đầu tiên: "Làm gọn phép làm tròn giá cước" (26/11 09:00), đổi \`Math.Ceiling\` thành \`Math.Round\`.
\`Math.Round(30,5)\` làm tròn về số chẵn gần nhất (30), nên mất 1.000 đ ở mọi giá có phần lẻ đúng 500 đ.

## Khắc phục
Revert commit lỗi (26/11 14:00), push. Thêm hook pre-push chặn push nếu FareCalculator không còn \`Math.Ceiling\`.

## Bảo trì kèm theo
.gitattributes cho diff C#, bảng tính và CHANGELOG; thư viện tiền tệ dùng chung bằng submodule \`lib/tien-te\`;
bản thiết kế bảng giá lưu bằng Git LFS; commit-graph, gc và fsck sạch; tag \`v1.2.0\` được ký bằng khoá SSH.
`;

// ---------- Các bước ----------

const g = (d, args, opts) => gitQuiet(d, args, opts);
const edit = (d, rel, fn) => writeText(d, rel, fn(readText(d, rel)));
const M = (t, day) => ({ as: 'minh', at: `2026-11-${day}T${t}:00+07:00` });
const L = (t, day) => ({ as: 'lan', by: 'minh', at: `2026-11-${day}T${t}:00+07:00` });
/** Cấu hình một lần cho các lệnh submodule với remote là thư mục (Git 2.38+ chặn mặc định). */
export const FILE_OK = ['-c', 'protocol.file.allow=always'];

/** Tám commit buổi sáng 26/11: [tiêu đề, file, hàm sửa, người viết, giờ]. Commit thứ 3 gây lỗi. */
export const MORNING = [
  ['Thêm ví dụ báo giá chuyến đêm vào README', 'README.md', README_NIGHT, M, '08:00'],
  ['Ghi phụ phí đêm vào tài liệu bảng giá', 'docs/bang-gia-cuoc.md', DOC_NIGHT_PRICE, L, '08:30'],
  ['Làm gọn phép làm tròn giá cước', 'src/BaoGia/FareCalculator.cs', BUG, M, '09:00'],
  ['Thêm test phí chờ tính theo giây', 'tests/BaoGia.Tests/WaitingFeeTests.cs', TEST_SECONDS, M, '09:30'],
  ['Sửa câu chữ tài liệu phí chờ', 'docs/phi-cho.md', DOC_TYPO, L, '10:00'],
  ['Đưa tham số phụ phí đêm vào cấu hình', 'src/BaoGia/appsettings.json', SETTINGS_NIGHT, M, '10:30'],
  ['Ghi chú nguồn của hằng số phụ phí đêm', 'src/BaoGia/NightSurcharge.cs', NIGHT_COMMENT, M, '11:00'],
  ['Hướng dẫn chạy riêng test tính cước', 'README.md', README_TEST, L, '11:30'],
];

export const STEPS = [
  {
    id: 'x1',
    note: 'GIT-041 (đầu bài): tám commit lên main, commit thứ ba làm sai phép làm tròn; push',
    run: (P) => {
      for (const [msg, file, fn, who, t] of MORNING) {
        edit(P.minh, file, fn);
        g(P.minh, ['commit', '-q', '-am', msg], who(t, 26));
      }
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'x2',
    note: 'GIT-041 (cuối bài): revert commit lỗi bisect tìm ra, push',
    run: (P) => {
      g(P.minh, ['revert', '--no-edit', 'HEAD~5'], M('14:00', 26));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'x3',
    note: 'GIT-043: khai báo diff/merge theo loại file trong .gitattributes',
    run: (P) => {
      edit(P.minh, '.gitattributes', GITATTRIBUTES_P05);
      g(P.minh, ['commit', '-q', '-am', 'Khai báo diff và merge theo loại file'], M('09:00', 27));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'x4',
    note: 'GIT-044: thư viện tiền tệ thành submodule lib/tien-te, rồi cập nhật lên phiên bản mới',
    run: (P) => {
      freshDir(P.libRemote);
      g(dirname(P.libRemote), ['init', '-q', '--bare', '-b', 'main', P.libRemote]);
      freshDir(P.libWork);
      g(dirname(P.libWork), ['clone', '-q', P.libRemote, P.libWork]);
      writeText(P.libWork, 'src/Vnd.cs', LIB_V1);
      g(P.libWork, ['add', '-A']);
      g(P.libWork, ['commit', '-q', '-m', 'Định dạng tiền Việt Nam'], { as: 'lan', at: '2026-11-27T09:30:00+07:00' });
      g(P.libWork, ['push', '-q', 'origin', 'main']);
      g(P.minh, [...FILE_OK, 'submodule', 'add', '-q', '../tien-te.git', 'lib/tien-te']);
      g(P.minh, ['commit', '-q', '-m', 'Dùng thư viện tiền tệ chung qua submodule'], M('10:00', 27));
      writeText(P.libWork, 'src/Vnd.cs', LIB_V2);
      g(P.libWork, ['commit', '-q', '-am', 'Thêm dạng ngắn 31k'], { as: 'lan', at: '2026-11-27T10:30:00+07:00' });
      g(P.libWork, ['push', '-q', 'origin', 'main']);
      g(P.minh, [...FILE_OK, 'submodule', 'update', '-q', '--remote', 'lib/tien-te']);
      g(P.minh, ['commit', '-q', '-am', 'Cập nhật thư viện tiền tệ: dạng ngắn'], M('11:00', 27));
      g(P.minh, ['push', '-q']);
      // Lan lấy main mới từ repository chung, rồi khởi tạo submodule.
      g(P.lan, ['pull', '-q', '--ff-only', 'upstream', 'main']);
      g(P.lan, [...FILE_OK, 'submodule', 'update', '-q', '--init']);
    },
  },
  {
    id: 'x5',
    note: 'GIT-045: bản thiết kế bảng giá lưu bằng Git LFS',
    run: (P) => {
      g(P.minh, ['lfs', 'install', '--local']);
      g(P.minh, ['lfs', 'track', 'docs/thiet-ke/*.png']);
      mkdirSync(join(P.minh, 'docs', 'thiet-ke'), { recursive: true });
      writeFileSync(join(P.minh, 'docs', 'thiet-ke', 'bang-gia-2027.png'), pngThietKe());
      g(P.minh, ['add', '.gitattributes', 'docs/thiet-ke']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm bản thiết kế bảng giá 2027 (Git LFS)'], M('14:00', 27));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'x6',
    note: 'GIT-048: ký commit và tag phát hành v1.2.0 bằng khoá SSH',
    run: (P) => {
      signingConfig(P.minh);
      writeText(P.minh, '.allowed_signers', ALLOWED_SIGNERS(readText(LAB, 'fixtures/khoa-ky-demo.pub')));
      g(P.minh, ['add', '.allowed_signers']);
      g(P.minh, ['commit', '-q', '-S', '-m', 'Thêm danh sách khoá ký của nhóm'], M('09:00', 28));
      g(P.minh, ['tag', '-s', 'v1.2.0', '-m', 'Phát hành 1.2.0: phụ phí đêm, phí chờ theo giây'], M('09:30', 28));
      g(P.minh, ['push', '-q', 'origin', 'main', 'v1.2.0']);
    },
  },
  {
    id: 'x7',
    note: 'GIT-049: hook chung của nhóm trong .githooks, core.hooksPath',
    run: (P) => {
      writeText(P.minh, '.githooks/pre-commit', HOOK_PRE_COMMIT);
      writeText(P.minh, '.githooks/commit-msg', HOOK_COMMIT_MSG);
      writeText(P.minh, '.githooks/pre-push', HOOK_PRE_PUSH);
      g(P.minh, ['add', '.githooks']);
      g(P.minh, ['update-index', '--chmod=+x', '.githooks/pre-commit', '.githooks/commit-msg', '.githooks/pre-push']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm hook chung của nhóm'], M('10:00', 28));
      g(P.minh, ['config', 'core.hooksPath', '.githooks']);
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'x8',
    note: 'GIT-050: báo cáo điều tra, push',
    run: (P) => {
      writeText(P.minh, 'docs/bao-cao-sai-cuoc.md', REPORT);
      g(P.minh, ['add', 'docs/bao-cao-sai-cuoc.md']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm báo cáo điều tra sai cước'], M('11:00', 28));
      g(P.minh, ['push', '-q']);
      g(P.minh, ['commit-graph', 'write', '--reachable', '--changed-paths']);
    },
  },
];

/** Cấu hình ký bằng khoá SSH demo cho một repository. */
export function signingConfig(dir) {
  g(dir, ['config', 'gpg.format', 'ssh']);
  g(dir, ['config', 'user.signingkey', `${SIGN_KEY}.pub`]);
  g(dir, ['config', 'gpg.ssh.allowedSignersFile', '.allowed_signers']);
}

/** Dựng Phần 1–4 rồi n bước của Phần 5. */
export function build5(dir, n = STEPS.length) {
  build4(dir);
  const P = places5(dir);
  for (const p of [P.libRemote, dirname(P.libWork)]) {
    if (existsSync(p)) rmSync(p, { recursive: true, force: true });
  }
  for (const s of STEPS.slice(0, n)) s.run(P);
  return P;
}

const cache = new Map();
/** Ref chuẩn của Minh, remote chung và Lan sau n bước của Phần 5. */
export function expectedRefs5(tmpDir, n) {
  if (!cache.has(n)) {
    const P = build5(tmpDir, n);
    cache.set(n, { minh: refsOf3(P.minh), remote: refsOf3(P.remote), lan: refsOf3(P.lan) });
  }
  return cache.get(n);
}
