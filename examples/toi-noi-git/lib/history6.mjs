// Lịch sử chuẩn của Phần 6: quy trình nhóm và tự động hoá.
//
// Dựng trên cuối Phần 5. Câu chuyện (30/11–07/12/2026): nhóm Tới Nơi thống nhất Conventional Commits
// và sinh CHANGELOG từ lịch sử; phát hành 1.3.0, vá 1.3.1 trên nhánh phát hành; thêm repository app
// tài xế và bản clone thư viện tiền tệ (làm việc nhiều repository); CI tính phiên bản từ tag; Lan lỡ
// commit khoá API lên nhánh trong fork và xoá khỏi lịch sử; CODEOWNERS và quyết định giữ hai
// repository; thư viện tiền tệ chuyển từ submodule sang subtree giữ lịch sử; cấu hình git chung của
// nhóm; một commit lỡ push lên main được revert; cuối cùng phát hành 1.4.0 theo runbook.
//
// `build6(dir, n)` dựng Phần 1–5 rồi chạy n bước của Phần 6.

import { spawnSync } from 'node:child_process';
import { existsSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';

import { refsOf3 } from './history3.mjs';
import { build5, places5 } from './history5.mjs';
import { freshDir, gitEnv, gitQuiet, readText, shPath, writeText } from './lab.mjs';

// ---------- Đường dẫn ----------

/** Thêm vào places5(): app tài xế, bản clone thư viện của Minh, repository tài liệu tách ra. */
export function places6(dir) {
  const P = places5(dir);
  const root = dirname(dir);
  const slash = (p) => p.split('\\').join('/');
  return {
    ...P,
    appRemote: slash(join(root, 'remote', 'app-tai-xe.git')),
    app: join(root, 'app-tai-xe'),
    libMinh: join(root, 'tien-te'),
    docsRemote: slash(join(root, 'remote', 'tai-lieu.git')),
  };
}

// ---------- Nội dung ----------

export const CONTRIBUTING = `# Đóng góp vào dịch vụ báo giá

## Tiêu đề commit: Conventional Commits

\`\`\`text
<loại>(<phạm vi>): <mô tả ngắn, viết thường, không chấm cuối>
\`\`\`

| Loại | Dùng khi | Vào CHANGELOG |
|---|---|---|
| \`feat\` | Tính năng mới cho người dùng | Tính năng mới |
| \`fix\` | Sửa lỗi người dùng gặp | Sửa lỗi |
| \`perf\` | Nhanh hơn, không đổi hành vi | Hiệu năng |
| \`docs\`, \`test\`, \`refactor\`, \`style\` | Tài liệu, test, sắp xếp lại code, định dạng | Không |
| \`build\`, \`ci\`, \`chore\` | Script, CI, việc vặt (phát hành, cấu hình) | Không |

Phạm vi: \`cuoc\`, \`phi-cho\`, \`ban-do\`, \`tien-te\`, \`readme\`, \`release\`, \`hook\`.
Thay đổi phá vỡ tương thích: thêm \`!\` sau loại (\`feat(cuoc)!: …\`) và đoạn \`BREAKING CHANGE: …\` trong thân commit.

Hook \`.githooks/commit-msg\` kiểm tra tiêu đề; bật một lần: \`git config core.hooksPath .githooks\`.
`;

export const HOOK_COMMIT_MSG_CC = `#!/bin/sh
# Tiêu đề commit theo Conventional Commits: <loại>(<phạm vi>): <mô tả>.
# Merge và Revert do git tạo, fixup!/squash! của rebase --autosquash được bỏ qua.
subject=$(head -n 1 "$1")
case "$subject" in
  Merge\\ *|Revert\\ \\"*|fixup!\\ *|squash!\\ *) exit 0 ;;
esac
if ! printf '%s\\n' "$subject" | grep -Eq '^(feat|fix|perf|docs|test|refactor|style|build|ci|chore|revert)(\\([a-z0-9-]+\\))?!?: .{3,}'; then
  echo "commit-msg: tiêu đề phải có dạng <loại>(<phạm vi>): <mô tả>, ví dụ fix(phi-cho): báo lỗi khi số giây chờ âm" >&2
  echo "commit-msg: loại hợp lệ: feat fix perf docs test refactor style build ci chore revert" >&2
  exit 1
fi
exit 0
`;

export const HOLIDAY_V1 = `namespace ToiNoi.BaoGia;

/// <summary>Phụ phí ngày lễ: cộng thêm 30% giá cước vào các ngày nghỉ lễ dương lịch cố định.</summary>
public static class HolidaySurcharge
{
    public const decimal Rate = 0.3m;

    private static readonly (int Month, int Day)[] Holidays = [(1, 1), (4, 30), (5, 1), (9, 2)];

    public static bool Applies(DateOnly date) => Holidays.Contains((date.Month, date.Day));

    public static decimal Apply(decimal fare, DateOnly date) =>
        Applies(date) ? fare * (1 + Rate) : fare;
}
`;
export const HOLIDAY_FIX = (s) =>
  s.replace(
    '        Applies(date) ? fare * (1 + Rate) : fare;',
    '        Applies(date) ? Math.Ceiling(fare * (1 + Rate) / 1000m) * 1000m : fare;',
  );

export const HOLIDAY_TESTS = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class HolidaySurchargeTests
{
    [Fact]
    public void Apply_NgayLe_Cong30PhanTram() => Assert.Equal(39000m, HolidaySurcharge.Apply(30000m, new DateOnly(2027, 9, 2)));

    [Fact]
    public void Apply_NgayThuong_GiuNguyen() => Assert.Equal(30000m, HolidaySurcharge.Apply(30000m, new DateOnly(2027, 9, 3)));
}
`;
export const HOLIDAY_TEST_ROUND = (s) =>
  s.replace(
    '    public void Apply_NgayThuong_GiuNguyen()',
    '    public void Apply_NgayLe_LamTronLen1000() => Assert.Equal(40000m, HolidaySurcharge.Apply(30500m, new DateOnly(2027, 4, 30)));\n\n    [Fact]\n    public void Apply_NgayThuong_GiuNguyen()',
  );

export const WAIT_NEGATIVE = (s) =>
  s.replace(
    '    public static decimal CalculateFromSeconds(int waitedSeconds) =>\n        Calculate((waitedSeconds + 59) / 60);',
    '    public static decimal CalculateFromSeconds(int waitedSeconds) =>\n        waitedSeconds < 0\n            ? throw new ArgumentOutOfRangeException(nameof(waitedSeconds))\n            : Calculate((waitedSeconds + 59) / 60);',
  );
export const README_HOLIDAY = (s) =>
  `${s}\nVí dụ ngày lễ (2/9): ô tô 4 chỗ, 4 km → 32.000 đ × 1,3 = 41.600 đ, làm tròn lên 42.000 đ.\n`;

export const CHANGELOG_SH = `#!/bin/sh
# Sinh một mục CHANGELOG từ tiêu đề commit theo Conventional Commits.
# Dùng: sh scripts/changelog.sh <phiên bản> <ngày> <tag trước>
ver=$1; day=$2; from=$3
log=$(git log --no-merges --reverse --format='%s' "$from..HEAD")
echo "## $ver ($day)"
for t in feat fix perf; do
  case $t in
    feat) title="Tính năng mới" ;;
    fix) title="Sửa lỗi" ;;
    perf) title="Hiệu năng" ;;
  esac
  lines=$(printf '%s\\n' "$log" | grep -E "^$t(\\(|!|:)" | sed -E 's/^[a-z]+\\(([^)]*)\\)!?: /- \\1: /; s/^[a-z]+!?: /- /')
  if [ -n "$lines" ]; then
    printf '\\n### %s\\n%s\\n' "$title" "$lines"
  fi
done
`;

export const CHANGELOG_HEAD = `# Nhật ký thay đổi

Những thay đổi người dùng thấy được của dịch vụ báo giá, mới nhất ở trên.
Phiên bản theo [Semantic Versioning](https://semver.org/lang/vi/); mỗi mục sinh bằng \`sh scripts/changelog.sh\`.
`;

/** Chèn mục mới ngay dưới phần đầu của CHANGELOG (mục mới nhất ở trên). */
export function addChangelogEntry(changelog, entry) {
  const head = changelog.startsWith(CHANGELOG_HEAD) ? CHANGELOG_HEAD : '';
  const rest = changelog.slice(head.length).replace(/^\n+/, '');
  return `${head || CHANGELOG_HEAD}\n${entry.trimEnd()}\n${rest ? `\n${rest}` : ''}`;
}

// App tài xế (Flutter) và thư viện.
export const APP_PUBSPEC = `name: app_tai_xe
description: App tài xế Tới Nơi.
version: 0.4.0
environment:
  sdk: ^3.5.0
dependencies:
  flutter:
    sdk: flutter
`;
export const APP_MAIN = `import 'package:flutter/material.dart';

void main() => runApp(const AppTaiXe());

class AppTaiXe extends StatelessWidget {
  const AppTaiXe({super.key});

  @override
  Widget build(BuildContext context) => const MaterialApp(home: NhanChuyen());
}

class NhanChuyen extends StatelessWidget {
  const NhanChuyen({super.key});

  @override
  Widget build(BuildContext context) => const Scaffold(body: Center(child: Text('Chờ chuyến mới…')));
}
`;
export const APP_PRICE = (s) =>
  s.replace(
    "  Widget build(BuildContext context) => const Scaffold(body: Center(child: Text('Chờ chuyến mới…')));",
    "  Widget build(BuildContext context) =>\n      const Scaffold(body: Center(child: Text('Chờ chuyến mới… Giá ước tính lấy từ dịch vụ báo giá.')));",
  );
export const APP_WIP = (s) => `${s}\n// TODO: hiện phụ phí ngày lễ trên màn hình nhận chuyến.\n`;
export const LIB_README = `# Thư viện tiền tệ Tới Nơi

\`Vnd.Format(31000m)\` → \`31.000 đ\`; \`Vnd.Short(31000m)\` → \`31k\`.
`;

export const PHIEN_BAN_SH = `#!/bin/sh
# In phiên bản của commit hiện tại theo tag gần nhất:
#   đúng tag v1.3.1            → 1.3.1
#   2 commit sau tag v1.3.1    → 1.3.2-dev.2+g<mã commit>
d=$(git describe --tags --match 'v[0-9]*' --long 2>/dev/null) || {
  echo "phien-ban: không tìm thấy tag v*: CI cần checkout đủ lịch sử và tag (fetch-depth: 0)" >&2
  exit 1
}
tag=\${d%-*-g*}
rest=\${d#"$tag"-}
n=\${rest%%-*}
hash=\${rest#*-g}
v=\${tag#v}
if [ "$n" = 0 ]; then
  echo "$v"
else
  major=\${v%%.*}; minor=\${v#*.}; minor=\${minor%%.*}; patch=\${v##*.}
  echo "$major.$minor.$((patch + 1))-dev.$n+g$hash"
fi
`;

export const CI_YML = `name: ci

on:
  push:
    branches: [main, 'phat-hanh/**']
    tags: ['v*']
  pull_request:

# Mặc định chỉ đọc; job phát hành cần ghi (tạo release) thì xin riêng.
permissions:
  contents: read

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0 # cần đủ lịch sử và tag để scripts/phien-ban.sh tính phiên bản
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '9.0.x'
          cache: true
          cache-dependency-path: '**/packages.lock.json'
      - name: Tiêu đề commit theo Conventional Commits (dùng lại hook của nhóm)
        if: github.event_name == 'pull_request'
        run: |
          git log --no-merges --format=%s "origin/\${{ github.base_ref }}..HEAD" | while read -r s; do
            printf '%s\\n' "$s" > "$RUNNER_TEMP/msg"
            sh .githooks/commit-msg "$RUNNER_TEMP/msg" || exit 1
          done
      - name: Phiên bản từ tag
        run: echo "VERSION=$(sh scripts/phien-ban.sh)" >> "$GITHUB_ENV"
      - run: dotnet test
      - run: dotnet publish src/BaoGia -c Release -p:Version="$VERSION"
`;

export const SECRET_LINE = '    "MapsApiKey": "tn_live_demo_7Hq2Kx9Lm4Pz",';
export const STAGING_JSON = `{
  "Maps": {
    "Endpoint": "https://maps.staging.toinoi.test",
${SECRET_LINE}
    "TimeoutSeconds": 3
  }
}
`;
export const MAPS_V1 = `namespace ToiNoi.BaoGia;

/// <summary>Ước lượng quãng đường (km) giữa hai điểm bằng dịch vụ bản đồ.</summary>
public sealed class MapsDistance(HttpClient http, string apiKey)
{
    public async Task<decimal> KmAsync(string from, string to)
    {
        var url = $"/distance?from={Uri.EscapeDataString(from)}&to={Uri.EscapeDataString(to)}&key={apiKey}";
        var meters = await http.GetFromJsonAsync<int>(url);
        return Math.Round(meters / 1000m, 1);
    }
}
`;
export const MAPS_TESTS = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class MapsDistanceTests
{
    [Fact]
    public void Km_LamTronMotChuSoThapPhan() => Assert.Equal(3.7m, Math.Round(3684 / 1000m, 1));
}
`;
export const GITIGNORE_STAGING = (s) =>
  s.replace(
    'appsettings.Development.json\n',
    'appsettings.Development.json\nappsettings.Staging.json\nappsettings.*.local.json\n',
  );
export const README_MAPS = (s) =>
  `${s}\nKhoá dịch vụ bản đồ không nằm trong repository: đặt biến môi trường \`Maps__MapsApiKey\` (xem docs/ban-do.md).\n`;
export const DOC_MAPS = `# Dịch vụ bản đồ

Báo giá ước lượng quãng đường bằng dịch vụ bản đồ khi khách chưa chọn tuyến.

- Endpoint và thời gian chờ: \`appsettings.json\`, mục \`Maps\`.
- Khoá API: biến môi trường \`Maps__MapsApiKey\`, **không** commit. Máy dev dùng \`dotnet user-secrets\`.
- Lỡ commit khoá: xoay khoá trước, rồi làm theo GIT-055.
`;

export const HOOK_PRE_COMMIT_SECRET = (s) =>
  s.replace(
    'exit 0\n',
    `# Chặn khoá API dạng tn_live_… trong mọi thay đổi đã stage (GIT-055).
if git diff --cached -U0 | grep -Eq '^\\+.*tn_live_[A-Za-z0-9_]{8,}'; then
  echo "pre-commit: có khoá API (tn_live_…) trong thay đổi đã stage. Đưa khoá vào biến môi trường rồi commit lại." >&2
  exit 1
fi
exit 0
`,
  );

export const CODEOWNERS = `# Ai cần review thay đổi ở từng khu vực. GitHub/GitLab tự thêm reviewer khi mở pull request.
# Dòng sau thắng dòng trước; tên nhóm và tài khoản là ví dụ của Tới Nơi.
*                 @toinoi/backend
/src/BaoGia/      @minh-toinoi
/docs/            @lan-toinoi @toinoi/van-hanh
/lib/tien-te/     @lan-toinoi
/.github/         @minh-toinoi
/.githooks/       @minh-toinoi
`;
export const ADR_REPOS = `# 0001. Báo giá và app tài xế: hai repository, thư viện tiền tệ gộp vào báo giá

Ngày: 03/12/2026 · Trạng thái: đã chấp nhận

## Bối cảnh
- Báo giá (C#, deploy nhiều lần mỗi tuần) và app tài xế (Flutter, phát hành qua kho ứng dụng 2 tuần/lần)
  khác ngôn ngữ, công cụ build, nhịp phát hành và người review.
- Thư viện tiền tệ (C#) chỉ báo giá dùng. Submodule \`lib/tien-te\` gây quên \`submodule update --init\`
  (GIT-050) và mỗi lần đổi thư viện phải sửa hai repository.

## Quyết định
- Giữ báo giá và app tài xế ở **hai repository**; hợp đồng giữa hai bên là API báo giá có phiên bản.
- Gộp thư viện tiền tệ vào báo giá bằng **git subtree**, giữ lịch sử (GIT-057); repository tien-te ngừng nhận commit.

## Hệ quả
- CI mỗi repository nhỏ và nhanh; thay đổi API phải tương thích ngược hoặc tăng phiên bản.
- Không còn submodule: clone một lần là đủ.
`;

export const TEAM_GITCONFIG = `# Cấu hình git khuyến nghị cho nhóm Tới Nơi (GIT-058). Bật cho một repository:
#   git config --local include.path ../docs/git/toinoi.gitconfig
# Đường dẫn tương đối tính từ file chứa nó (.git/config). Cấu hình riêng của bạn vẫn thắng nếu đặt sau.
[alias]
	st = status -sb
	lg = log --oneline --graph --decorate -12
	last = log -1 --stat
	unstage = restore --staged
	amend = commit --amend --no-edit
	nhanh = branch --format='%(committerdate:short) %(refname:short)'
[pull]
	ff = only
[push]
	autoSetupRemote = true
[fetch]
	prune = true
[rebase]
	autoStash = true
	updateRefs = true
[rerere]
	enabled = true
[diff]
	algorithm = histogram
[merge]
	conflictStyle = zdiff3
[branch]
	sort = -committerdate
[tag]
	sort = -version:refname
`;

export const RUNBOOK = `# Runbook: phát hành dịch vụ báo giá

Người phát hành: trực tuần. Thời gian: 30 phút. Phiên bản theo semver: \`feat\` → tăng MINOR, chỉ \`fix\` → tăng PATCH,
có \`!\`/BREAKING CHANGE → tăng MAJOR.

## 1. Chuẩn bị
- [ ] \`main\` sạch, trùng \`origin/main\`: \`git switch main && git pull --ff-only\`
- [ ] CI xanh trên commit cuối của \`main\`
- [ ] Phiên bản mới: \`git describe --tags --abbrev=0\` là bản trước; đọc \`git log --oneline <bản trước>..main\`

## 2. Ghi CHANGELOG
- [ ] \`sh scripts/changelog.sh <phiên bản> <ngày> <bản trước>\`, dán vào đầu CHANGELOG.md, sửa câu chữ nếu cần
- [ ] \`git commit -am "chore(release): <phiên bản>"\`

## 3. Tag và nhánh phát hành
- [ ] \`git tag -s v<phiên bản> -m "Phát hành <phiên bản>: <tóm tắt>"\` (ký bằng khoá SSH, GIT-048)
- [ ] \`git tag -v v<phiên bản>\` báo Good signature
- [ ] \`git branch phat-hanh/<MAJOR.MINOR> v<phiên bản>\` (bản vá sau này đi vào nhánh này)
- [ ] \`git push origin main phat-hanh/<MAJOR.MINOR> v<phiên bản>\`

## 4. Sau phát hành
- [ ] CI của tag xanh, bản build có đúng phiên bản (\`scripts/phien-ban.sh\`)
- [ ] Báo nhóm vận hành kèm mục CHANGELOG
- [ ] \`node kiem-tra.mjs\` của lab (GIT-060) đạt 10/10

## Vá bản đã phát hành
Sửa trên \`phat-hanh/<MAJOR.MINOR>\`, ghi CHANGELOG, tag bản PATCH, push nhánh và tag, rồi merge nhánh phát hành về \`main\`
(bản vá và mục CHANGELOG về \`main\` cùng lúc). Nhánh phát hành cũ hơn cần cùng bản vá: \`git cherry-pick -x <commit>\`,
và khi ghi CHANGELOG bản sau thì bỏ mục trùng.

## Khi có sự cố
- Tag sai commit nhưng **chưa** push: \`git tag -d\`, tag lại. **Đã** push: không sửa tag, phát hành bản PATCH mới.
- Push nhầm lên \`main\`: \`git revert\`, không force push (GIT-059).
`;

// ---------- Các bước ----------

const g = (d, args, opts) => gitQuiet(d, args, opts);
const edit = (d, rel, fn) => writeText(d, rel, fn(readText(d, rel)));
const M = (t, day) => ({ as: 'minh', at: `2026-${day}T${t}:00+07:00` });
const L = (t, day) => ({ as: 'lan', at: `2026-${day}T${t}:00+07:00` });
const LM = (t, day) => ({ as: 'lan', by: 'minh', at: `2026-${day}T${t}:00+07:00` });
/** gc/repack một luồng: pack ra cùng kích thước trên mọi máy. */
export const ONE_THREAD = { GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'pack.threads', GIT_CONFIG_VALUE_0: '1' };

/** Chạy một script sh trong repository với môi trường cô lập của lab; trả stdout. */
export function shQuiet(dir, args, opts = {}) {
  const r = spawnSync(shPath(), args, { cwd: dir, env: gitEnv(opts), encoding: 'utf8' });
  if (r.error) throw r.error;
  if (r.status !== 0 && !opts.allowFail) throw new Error(`sh ${args.join(' ')} (mã ${r.status})\n${r.stdout}${r.stderr}`);
  return { stdout: r.stdout ?? '', stderr: r.stderr ?? '', status: r.status };
}

/** Ghi mục CHANGELOG bằng scripts/changelog.sh rồi commit chore(release). */
export function releaseCommit(dir, ver, day, from, when) {
  const entry = shQuiet(dir, ['scripts/changelog.sh', ver, day, from]).stdout;
  const cl = existsSync(join(dir, 'CHANGELOG.md')) ? readText(dir, 'CHANGELOG.md') : CHANGELOG_HEAD;
  writeText(dir, 'CHANGELOG.md', addChangelogEntry(cl, entry));
  g(dir, ['add', 'CHANGELOG.md']);
  g(dir, ['commit', '-q', '-m', `chore(release): ${ver}`], when);
}

/** Commit của ngày 30/11 theo Conventional Commits: [tiêu đề, file, nội dung hoặc hàm sửa, người, giờ]. */
export const CC_COMMITS = [
  ['feat(cuoc): phụ phí ngày lễ 30%', 'src/BaoGia/HolidaySurcharge.cs', HOLIDAY_V1, LM, '09:30'],
  ['test(cuoc): test phụ phí ngày lễ', 'tests/BaoGia.Tests/HolidaySurchargeTests.cs', HOLIDAY_TESTS, LM, '10:00'],
  ['fix(phi-cho): báo lỗi khi số giây chờ âm', 'src/BaoGia/WaitingFee.cs', WAIT_NEGATIVE, M, '10:30'],
  ['docs(readme): ví dụ phụ phí ngày lễ', 'README.md', README_HOLIDAY, M, '11:00'],
];

export const STEPS = [
  {
    id: 'y1',
    note: 'GIT-051: Conventional Commits, hook commit-msg mới, CHANGELOG 1.3.0',
    run: (P) => {
      writeText(P.minh, 'CONTRIBUTING.md', CONTRIBUTING);
      writeText(P.minh, '.githooks/commit-msg', HOOK_COMMIT_MSG_CC);
      g(P.minh, ['add', 'CONTRIBUTING.md', '.githooks/commit-msg']);
      g(P.minh, ['commit', '-q', '-m', 'docs: quy ước commit theo Conventional Commits'], M('09:00', '11-30'));
      for (const [msg, file, body, who, t] of CC_COMMITS) {
        if (typeof body === 'function') edit(P.minh, file, body);
        else writeText(P.minh, file, body);
        g(P.minh, ['add', file]);
        g(P.minh, ['commit', '-q', '-m', msg], who(t, '11-30'));
      }
      writeText(P.minh, 'scripts/changelog.sh', CHANGELOG_SH);
      g(P.minh, ['add', 'scripts/changelog.sh']);
      g(P.minh, ['update-index', '--chmod=+x', 'scripts/changelog.sh']);
      g(P.minh, ['commit', '-q', '-m', 'build: script sinh changelog từ tiêu đề commit'], M('14:00', '11-30'));
      releaseCommit(P.minh, '1.3.0', '2026-11-30', 'v1.2.0', M('14:30', '11-30'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y2',
    note: 'GIT-052: tag v1.3.0, nhánh phat-hanh/1.3, vá 1.3.1 trên nhánh phát hành, merge về main',
    run: (P) => {
      g(P.minh, ['tag', '-s', 'v1.3.0', '-m', 'Phát hành 1.3.0: phụ phí ngày lễ'], M('09:00', '12-01'));
      g(P.minh, ['push', '-q', 'origin', 'main', 'v1.3.0']);
      g(P.minh, ['switch', '-q', '-c', 'phat-hanh/1.3', 'v1.3.0']);
      g(P.minh, ['push', '-q', '-u', 'origin', 'phat-hanh/1.3']);
      // Vá trên nhánh phát hành (main chưa có gì mới sau 1.3.0), rồi merge nhánh về main.
      edit(P.minh, 'src/BaoGia/HolidaySurcharge.cs', HOLIDAY_FIX);
      edit(P.minh, 'tests/BaoGia.Tests/HolidaySurchargeTests.cs', HOLIDAY_TEST_ROUND);
      g(P.minh, ['commit', '-q', '-am', 'fix(cuoc): làm tròn phụ phí ngày lễ lên bội số 1.000 đ'], LM('10:00', '12-01'));
      releaseCommit(P.minh, '1.3.1', '2026-12-01', 'v1.3.0', M('10:45', '12-01'));
      g(P.minh, ['tag', '-s', 'v1.3.1', '-m', 'Phát hành 1.3.1: làm tròn phụ phí ngày lễ'], M('11:00', '12-01'));
      g(P.minh, ['push', '-q', 'origin', 'phat-hanh/1.3', 'v1.3.1']);
      g(P.minh, ['switch', '-q', 'main']);
      g(P.minh, ['merge', '-q', '--no-ff', '--no-edit', 'phat-hanh/1.3'], M('11:15', '12-01'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y3',
    note: 'GIT-053: repository app tài xế, bản clone thư viện của Minh; làm việc trên nhiều repository',
    run: (P) => {
      freshDir(P.appRemote);
      g(dirname(P.appRemote), ['init', '-q', '--bare', '-b', 'main', P.appRemote]);
      freshDir(P.app);
      g(dirname(P.app), ['clone', '-q', P.appRemote, P.app]);
      writeText(P.app, 'pubspec.yaml', APP_PUBSPEC);
      writeText(P.app, 'lib/main.dart', APP_MAIN);
      g(P.app, ['add', '-A']);
      g(P.app, ['commit', '-q', '-m', 'feat: màn hình nhận chuyến'], LM('08:30', '12-02'));
      edit(P.app, 'lib/main.dart', APP_PRICE);
      g(P.app, ['commit', '-q', '-am', 'feat(gia): hiện giá ước tính từ dịch vụ báo giá'], M('09:00', '12-02'));
      g(P.app, ['push', '-q', 'origin', 'main']);
      // Minh clone thư viện tiền tệ để theo dõi; sau đó Lan đẩy thêm một commit (Minh đi sau 1 commit).
      freshDir(P.libMinh);
      g(dirname(P.libMinh), ['clone', '-q', P.libRemote, P.libMinh]);
      writeText(P.libWork, 'README.md', LIB_README);
      g(P.libWork, ['add', 'README.md']);
      g(P.libWork, ['commit', '-q', '-m', 'docs: cách dùng Vnd.Format và Vnd.Short'], L('09:30', '12-02'));
      g(P.libWork, ['push', '-q', 'origin', 'main']);
      g(P.libMinh, ['fetch', '-q']);
      // Việc dở trong app tài xế (chưa commit).
      edit(P.app, 'lib/main.dart', APP_WIP);
    },
  },
  {
    id: 'y4',
    note: 'GIT-054: workflow CI và script tính phiên bản từ tag',
    run: (P) => {
      writeText(P.minh, '.github/workflows/ci.yml', CI_YML);
      writeText(P.minh, 'scripts/phien-ban.sh', PHIEN_BAN_SH);
      g(P.minh, ['add', '.github', 'scripts/phien-ban.sh']);
      g(P.minh, ['update-index', '--chmod=+x', 'scripts/phien-ban.sh']);
      g(P.minh, ['commit', '-q', '-m', 'ci: build, test và tính phiên bản từ tag trên mỗi push'], M('14:00', '12-02'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y5',
    note: 'GIT-055: Lan lỡ commit khoá API lên nhánh trong fork; xoá khỏi lịch sử, force-with-lease; Minh merge, thêm chặn trong hook',
    run: (P) => {
      g(P.lan, ['fetch', '-q', 'upstream']);
      g(P.lan, ['switch', '-q', '-c', 'tinh-nang/ban-do', 'upstream/main']);
      writeText(P.lan, 'src/BaoGia/MapsDistance.cs', MAPS_V1);
      writeText(P.lan, 'src/BaoGia/appsettings.Staging.json', STAGING_JSON);
      g(P.lan, ['add', 'src/BaoGia']);
      g(P.lan, ['commit', '-q', '-m', 'feat(ban-do): ước lượng quãng đường bằng dịch vụ bản đồ'], L('09:00', '12-03'));
      writeText(P.lan, 'tests/BaoGia.Tests/MapsDistanceTests.cs', MAPS_TESTS);
      g(P.lan, ['add', 'tests']);
      g(P.lan, ['commit', '-q', '-m', 'test(ban-do): làm tròn quãng đường một chữ số thập phân'], L('09:30', '12-03'));
      g(P.lan, ['push', '-q', '-u', 'origin', 'tinh-nang/ban-do']);
      cleanSecret(P.lan);
      g(P.lan, ['push', '-q', '--force-with-lease', 'origin', 'tinh-nang/ban-do']);
      purgeOld(P.lan);
      // Minh merge pull request từ fork của Lan, rồi thêm chặn khoá API vào pre-commit.
      g(P.minh, ['fetch', '-q', P.fork, 'tinh-nang/ban-do']);
      g(P.minh, ['merge', '-q', '--no-ff', '-m', 'Merge pull request #7 từ lan/tinh-nang/ban-do', 'FETCH_HEAD'], M('11:00', '12-03'));
      edit(P.minh, '.githooks/pre-commit', HOOK_PRE_COMMIT_SECRET);
      g(P.minh, ['commit', '-q', '-am', 'chore(hook): chặn khoá API tn_live_ trong thay đổi đã stage'], M('11:30', '12-03'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y6',
    note: 'GIT-056: CODEOWNERS và quyết định hai repository',
    run: (P) => {
      writeText(P.minh, '.github/CODEOWNERS', CODEOWNERS);
      g(P.minh, ['add', '.github/CODEOWNERS']);
      g(P.minh, ['commit', '-q', '-m', 'chore: CODEOWNERS cho từng khu vực'], M('14:00', '12-03'));
      writeText(P.minh, 'docs/quyet-dinh/0001-hai-repository.md', ADR_REPOS);
      g(P.minh, ['add', 'docs/quyet-dinh']);
      g(P.minh, ['commit', '-q', '-m', 'docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá'], M('14:30', '12-03'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y7',
    note: 'GIT-057: bỏ submodule, gộp thư viện tiền tệ bằng subtree giữ lịch sử; tách tài liệu ra repository riêng',
    run: (P) => {
      removeSubmodule(P.minh);
      g(P.minh, ['commit', '-q', '-m', 'refactor(tien-te): bỏ submodule lib/tien-te'], M('09:00', '12-04'));
      g(P.minh, ['subtree', 'add', '-q', '--prefix=lib/tien-te', '../remote/tien-te.git', 'main', '-m', 'refactor(tien-te): gộp thư viện tiền tệ bằng subtree, giữ lịch sử'], M('09:30', '12-04'));
      g(P.minh, ['push', '-q']);
      // Bản sao docs/ kèm lịch sử cho bộ phận vận hành.
      freshDir(P.docsRemote);
      g(dirname(P.docsRemote), ['init', '-q', '--bare', '-b', 'main', P.docsRemote]);
      g(P.minh, ['subtree', 'split', '-q', '--prefix=docs', '-b', 'tach/tai-lieu']);
      // URL file:// (không phải đường dẫn trần): Git LFS cần URL để đẩy bản thiết kế trong docs/thiet-ke.
      g(P.minh, ['push', '-q', `file:///${P.docsRemote.replace(/^\//, '')}`, 'tach/tai-lieu:main']);
      g(P.minh, ['branch', '-q', '-D', 'tach/tai-lieu']);
    },
  },
  {
    id: 'y8',
    note: 'GIT-058: cấu hình git chung của nhóm qua include.path',
    run: (P) => {
      writeText(P.minh, 'docs/git/toinoi.gitconfig', TEAM_GITCONFIG);
      g(P.minh, ['add', 'docs/git']);
      g(P.minh, ['commit', '-q', '-m', 'docs: cấu hình git khuyến nghị cho nhóm'], M('11:00', '12-04'));
      g(P.minh, ['push', '-q']);
      g(P.minh, ['config', '--local', 'include.path', '../docs/git/toinoi.gitconfig']);
    },
  },
  {
    id: 'y9',
    note: 'GIT-059: commit lỡ push lên main được revert, giữ lại trên nhánh riêng',
    run: (P) => {
      edit(P.minh, 'src/BaoGia/appsettings.json', MAPS_DEFAULT_ON);
      g(P.minh, ['commit', '-q', '-am', 'chore(ban-do): bật ước lượng quãng đường mặc định'], M('14:00', '12-04'));
      g(P.minh, ['push', '-q']);
      g(P.minh, ['branch', 'tinh-nang/ban-do-mac-dinh']);
      g(P.minh, ['revert', '--no-edit', 'HEAD'], M('14:20', '12-04'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'y10',
    note: 'GIT-060: phát hành 1.4.0 theo runbook',
    run: (P) => {
      writeText(P.minh, 'docs/quy-trinh-phat-hanh.md', RUNBOOK);
      g(P.minh, ['add', 'docs/quy-trinh-phat-hanh.md']);
      g(P.minh, ['commit', '-q', '-m', 'docs: runbook quy trình phát hành'], M('09:00', '12-07'));
      releaseCommit(P.minh, '1.4.0', '2026-12-07', 'v1.3.1', M('09:30', '12-07'));
      g(P.minh, ['tag', '-s', 'v1.4.0', '-m', 'Phát hành 1.4.0: ước lượng quãng đường bằng bản đồ'], M('09:45', '12-07'));
      g(P.minh, ['branch', 'phat-hanh/1.4', 'v1.4.0']);
      g(P.minh, ['push', '-q', 'origin', 'main', 'phat-hanh/1.4', 'v1.4.0']);
    },
  },
];

export const MAPS_DEFAULT_ON = (s) => s.replace('  "Fare": {', '  "Maps": {\n    "EstimateByDefault": true\n  },\n  "Fare": {');

/** Lệnh viết lại nhánh: bỏ appsettings.Staging.json khỏi mọi commit, bỏ commit thành rỗng (như filter-repo). */
export const FILTER_ARGS = ['filter-branch', '-f', '--prune-empty', '--index-filter', 'git rm -q --cached --ignore-unmatch src/BaoGia/appsettings.Staging.json', '--', 'upstream/main..tinh-nang/ban-do'];
export const FILTER_ENV = { FILTER_BRANCH_SQUELCH_WARNING: '1' };

/** Xoá appsettings.Staging.json khỏi mọi commit của nhánh, rồi commit .gitignore và tài liệu. */
export function cleanSecret(dir) {
  g(dir, FILTER_ARGS, { env: FILTER_ENV });
  g(dir, ['update-ref', '-d', 'refs/original/refs/heads/tinh-nang/ban-do']);
  rmSync(join(dir, 'src', 'BaoGia', 'appsettings.Staging.json'), { force: true });
  edit(dir, '.gitignore', GITIGNORE_STAGING);
  writeText(dir, 'docs/ban-do.md', DOC_MAPS);
  edit(dir, 'README.md', README_MAPS);
  g(dir, ['add', '.gitignore', 'docs/ban-do.md', 'README.md']);
  g(dir, ['commit', '-q', '-m', 'chore(ban-do): khoá API đọc từ biến môi trường, không commit'], { as: 'lan', at: '2026-12-03T10:30:00+07:00' });
}

/** Sau khi force push: quên reflog và xoá ngay object không còn ref nào trỏ tới (bản cũ chứa khoá). */
export function purgeOld(dir) {
  g(dir, ['reflog', 'expire', '--expire=now', '--all']);
  g(dir, ['gc', '-q', '--prune=now'], { env: ONE_THREAD });
}

/** Bỏ submodule lib/tien-te: xoá khỏi index và .gitmodules (file rỗng thì xoá luôn), dọn .git/modules. */
export function removeSubmodule(dir) {
  g(dir, ['submodule', 'deinit', '-q', '-f', 'lib/tien-te']);
  g(dir, ['rm', '-q', 'lib/tien-te']);
  if (!readText(dir, '.gitmodules').trim()) g(dir, ['rm', '-q', '-f', '.gitmodules']);
  rmSync(join(dir, '.git', 'modules', 'lib', 'tien-te'), { recursive: true, force: true });
}

/** Dựng Phần 1–5 rồi n bước của Phần 6. */
export function build6(dir, n = STEPS.length) {
  build5(dir);
  const P = places6(dir);
  for (const p of [P.appRemote, P.app, P.libMinh, P.docsRemote]) {
    if (existsSync(p)) rmSync(p, { recursive: true, force: true });
  }
  for (const s of STEPS.slice(0, n)) s.run(P);
  return P;
}

const cache = new Map();
/** Ref chuẩn của Minh, remote chung và Lan sau n bước của Phần 6 (dựng một lần, ghi lại sau từng bước). */
export function expectedRefs6(tmpDir, n) {
  if (!cache.has(n)) {
    const P = build6(tmpDir, 0);
    const snap = () => ({ minh: refsOf3(P.minh), remote: refsOf3(P.remote), lan: refsOf3(P.lan) });
    cache.set(0, snap());
    STEPS.forEach((s, i) => {
      s.run(P);
      cache.set(i + 1, snap());
    });
  }
  return cache.get(n);
}
