// Lịch sử chuẩn của Phần 4: hoàn tác và viết lại lịch sử.
//
// Dựng trên cuối Phần 3 (Minh, remote chung, Lan, fork). Câu chuyện: Minh tăng phí chờ tối đa
// rồi phải revert; làm tính năng phụ phí đêm trên một nhánh lộn xộn rồi dọn bằng sửa commit và
// rebase tương tác; lỡ tạo nhánh sửa lỗi từ nhánh tính năng (rebase --onto); nhận patch của Lan
// (git am); ghi nhớ cách giải quyết xung đột (rerere); cuối cùng merge nhánh đã dọn và viết runbook.
//
// `build4(dir, n)` dựng Phần 1–3 rồi chạy n bước của Phần 4.

import { join } from 'node:path';

import { build3, places, refsOf3 } from './history3.mjs';
import { gitQuiet, readText, writeText } from './lab.mjs';

export const NIGHT_BRANCH = 'tinh-nang/phu-phi-dem';
export const ROUND_BRANCH = 'sua/lam-tron-giay-cho';
export const LAN_BRANCH = 'sua/phi-cho-moi-chuyen';

// ---------- Nội dung file ----------

export const WAITING_MAX_50 = (s) =>
  s.replace('tối đa 30.000 đ.</summary>', 'tối đa 50.000 đ.</summary>').replace('30000m);', '50000m);');

export const NIGHT_V1 = `namespace ToiNoi.BaoGia;

/// <summary>Phụ phí đêm: chuyến bắt đầu từ 22:00 đến trước 5:00 cộng thêm 20% giá cước.</summary>
public static class NightSurcharge
{
    public const decimal Rate = 0.2m;

    public static bool Applies(TimeOnly start) => start.Hour >= 22 || start.Hour < 6;

    public static decimal Apply(decimal fare, TimeOnly start)
    {
        if (!Applies(start))
            return fare;

        return Math.Ceiling(fare * (1 + Rate) / 1000m) * 1000m;
    }
}
`;

/** Commit gỡ lỗi chen vào giữa nhánh: in ra console mỗi lần tính. */
export const NIGHT_DEBUG = (s) =>
  s.replace(
    '    public static decimal Apply(decimal fare, TimeOnly start)\n    {\n',
    '    public static decimal Apply(decimal fare, TimeOnly start)\n    {\n        Console.WriteLine($"[debug] phụ phí đêm cho chuyến lúc {start}");\n',
  );

/** Sửa giờ kết thúc: trước 5:00 chứ không phải 6:00. */
export const NIGHT_FIX_5H = (s) => s.replace('start.Hour < 6;', 'start.Hour < 5;');

export const NIGHT_TESTS = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class NightSurchargeTests
{
    [Fact]
    public void Apply_ChuyenLuc23h_Cong20PhanTram() =>
        Assert.Equal(36000m, NightSurcharge.Apply(30000m, new TimeOnly(23, 0)));

    [Fact]
    public void Apply_ChuyenLuc5h_KhongCong() =>
        Assert.Equal(30000m, NightSurcharge.Apply(30000m, new TimeOnly(5, 0)));
}
`;

/** Thêm ca biên 22:00 (commit fixup! của bài GIT-037). */
export const NIGHT_TESTS_22H = (s) =>
  s.replace(
    '        Assert.Equal(30000m, NightSurcharge.Apply(30000m, new TimeOnly(5, 0)));\n}\n',
    '        Assert.Equal(30000m, NightSurcharge.Apply(30000m, new TimeOnly(5, 0)));\n\n    [Fact]\n    public void Apply_ChuyenLuc22h_Cong20PhanTram() =>\n        Assert.Equal(36000m, NightSurcharge.Apply(30000m, new TimeOnly(22, 0)));\n}\n',
  );

const WAIT_LINE = 'Tài xế chờ quá 5 phút tại điểm đón: mỗi phút tiếp theo tính thêm 1.000 đ, tối đa 30.000 đ.';
/** Nhánh phụ phí đêm ghi chú vào cùng dòng tài liệu phí chờ mà patch của Lan cũng sửa. */
export const DOC_NIGHT = (s) => s.replace(WAIT_LINE, `${WAIT_LINE.slice(0, -1)}. Phí chờ không cộng phụ phí đêm.`);
export const DOC_LAN = (s) => s.replace(WAIT_LINE, `${WAIT_LINE.slice(0, -1)} cho mỗi chuyến.`);
export const DOC_RESOLVED = (s) =>
  s.replace(/<<<<<<< [^\n]*\n[^\n]*\n=======\n[^\n]*\n>>>>>>> [^\n]*\n/, `${WAIT_LINE.slice(0, -1)} cho mỗi chuyến. Phí chờ không cộng phụ phí đêm.\n`);

export const WAITING_SECONDS = (s) =>
  s.replace(
    '        return Math.Min(billable * 1000m, 30000m);\n    }\n}\n',
    '        return Math.Min(billable * 1000m, 30000m);\n    }\n\n    /// <summary>Thời gian chờ tính bằng giây, phần lẻ làm tròn lên phút.</summary>\n    public static decimal CalculateFromSeconds(int waitedSeconds) =>\n        Calculate((waitedSeconds + 59) / 60);\n}\n',
  );

export const RUNBOOK = `# Runbook hoàn tác của nhóm Tới Nơi

Trước mọi thao tác dưới đây: \`git status\` và \`git log --oneline -5\`, ghi lại mã commit hiện tại.

| Tình huống | Đã push? | Làm gì | Lệnh |
|---|---|---|---|
| Sửa file sai, chưa stage | Không | Bỏ thay đổi trong working tree | \`git restore <file>\` |
| Lỡ stage | Không | Đưa ra khỏi staging area, giữ thay đổi | \`git restore --staged <file>\` |
| Commit cuối sai nội dung hoặc thiếu file | Không | Sửa commit cuối | \`git commit --amend\` |
| Vài commit cuối cần làm lại | Không | Lùi nhánh, giữ thay đổi | \`git reset --soft\` / \`--mixed <commit>\` |
| Commit đã push gây lỗi | Có | Tạo commit đảo ngược | \`git revert <commit>\` (merge: \`-m 1\`) |
| Lỡ reset --hard, xoá nhánh, rebase hỏng | Không | Tìm vị trí cũ trong reflog | \`git reflog\`, \`git reset --hard <mục reflog>\` |
| Nhánh tính năng lộn xộn trước pull request | Không (hoặc chỉ nhánh của mình) | Dọn bằng rebase tương tác | \`git rebase -i --autosquash main\` |
| Nhánh tạo nhầm gốc | Không | Chuyển chuỗi commit sang gốc đúng | \`git rebase --onto <gốc đúng> <gốc sai> <nhánh>\` |

Không bao giờ: \`reset --hard\`, \`rebase\` hay \`push --force\` trên \`main\` và \`phat-hanh/*\`.
Trong GitStudio: Hoàn tác (Ctrl+Z) chỉ nhớ các thao tác của phiên đang mở; bỏ thay đổi (Huỷ bỏ) không hoàn tác được.
`;

// ---------- Tiện ích ----------

const g = (d, args, opts) => gitQuiet(d, args, opts);
const edit = (d, rel, fn) => writeText(d, rel, fn(readText(d, rel)));
const short = (d, rev) => g(d, ['rev-parse', '--short', rev]).stdout.trim();

/**
 * Môi trường cho `git rebase -i` không cần người gõ: in danh sách việc (todo) Git đề xuất,
 * rồi chạy các lệnh sed trên todo. GIT_EDITOR=true giữ nguyên nội dung commit khi squash.
 */
export function todoEditor(...sedExprs) {
  const sed = sedExprs.map((e) => `sed -i -e '${e}' "$1"; `).join('');
  return {
    GIT_SEQUENCE_EDITOR: `f() { grep -v '^#' "$1" | grep . ; ${sed}}; f`,
    GIT_EDITOR: 'true',
  };
}

/** Dòng todo `pick <mã ngắn> …` của một commit, để sed đổi lệnh hoặc vị trí. */
export const pickOf = (d, rev) => short(d, rev);

// ---------- Các bước ----------

const M = (t, day = 24) => ({ as: 'minh', at: `2026-11-${day}T${t}:00+07:00` });
const C = (t, day = 24) => ({ committedAt: `2026-11-${day}T${t}:00+07:00` });

export const STEPS = [
  {
    id: 'w1',
    note: 'GIT-033: tăng phí chờ tối đa, push, rồi revert',
    run: (P) => {
      edit(P.minh, 'src/BaoGia/WaitingFee.cs', WAITING_MAX_50);
      g(P.minh, ['commit', '-q', '-am', 'Tăng phí chờ tối đa lên 50.000 đ'], M('09:00'));
      g(P.minh, ['push', '-q']);
      g(P.minh, ['revert', '--no-edit', 'HEAD'], M('09:30'));
      g(P.minh, ['push', '-q']);
    },
  },
  {
    id: 'w2',
    note: 'GIT-036 (đầu bài): nhánh phụ phí đêm với bốn commit lộn xộn',
    run: (P) => {
      g(P.minh, ['switch', '-q', '-c', NIGHT_BRANCH]);
      writeText(P.minh, 'src/BaoGia/NightSurcharge.cs', NIGHT_V1);
      edit(P.minh, 'docs/phi-cho.md', DOC_NIGHT);
      g(P.minh, ['add', '-A']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm phụ phi đêm'], M('14:00'));
      edit(P.minh, 'src/BaoGia/NightSurcharge.cs', NIGHT_DEBUG);
      g(P.minh, ['commit', '-q', '-am', 'debug: in giờ bắt đầu chuyến'], M('14:20'));
      writeText(P.minh, 'tests/BaoGia.Tests/NightSurchargeTests.cs', NIGHT_TESTS);
      g(P.minh, ['add', '-A']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm test phụ phí đêm'], M('14:40'));
      edit(P.minh, 'src/BaoGia/NightSurcharge.cs', NIGHT_FIX_5H);
      g(P.minh, ['commit', '-q', '-am', 'Sửa giờ kết thúc phụ phí đêm thành 5h'], M('15:00'));
    },
  },
  {
    id: 'w3',
    note: 'GIT-036 (cuối bài): sửa nội dung commit đầu, bỏ commit debug, đưa commit sửa giờ lên cạnh commit đầu',
    run: (P) => {
      const first = g(P.minh, ["rev-parse", `${NIGHT_BRANCH}~3`]).stdout.trim();
      // Như GitStudio "Sửa nội dung commit": commit "amend!" rỗng rồi rebase --autosquash.
      g(P.minh, ['commit', '-q', '--allow-empty', '--only', '-m', 'amend! Thêm phụ phi đêm', '-m', 'Thêm phụ phí đêm'], M('16:00'));
      g(P.minh, ['rebase', '-q', '-i', '--autosquash', `${first}~1`], { ...C('16:00'), env: todoEditor() });
      // Bỏ commit debug: rebase --onto cha-của-nó nó.
      const debug = `${NIGHT_BRANCH}~2`;
      g(P.minh, ['rebase', '-q', '--onto', `${debug}~1`, debug], C('16:10'));
      // Đổi chỗ: commit sửa giờ lên trước commit test.
      const test = pickOf(P.minh, 'HEAD~1');
      const fix = pickOf(P.minh, 'HEAD');
      g(P.minh, ['rebase', '-q', '-i', 'HEAD~2'], {
        ...C('16:20'),
        env: todoEditor(`s/^pick ${test} /pick TEST /`, `s/^pick ${fix} /pick ${test} /`, `s/^pick TEST /pick ${fix} /`),
      });
    },
  },
  {
    id: 'w4',
    note: 'GIT-037: commit fixup! cho test, rebase -i --autosquash gộp luôn commit sửa giờ vào commit đầu',
    run: (P) => {
      edit(P.minh, 'tests/BaoGia.Tests/NightSurchargeTests.cs', NIGHT_TESTS_22H);
      g(P.minh, ['commit', '-q', '-a', '--fixup', 'HEAD'], M('09:00', 25));
      const fix = pickOf(P.minh, 'HEAD~2');
      g(P.minh, ['rebase', '-q', '-i', '--autosquash', 'main'], { ...C('09:10', 25), env: todoEditor(`s/^pick ${fix} /fixup ${fix} /`) });
    },
  },
  {
    id: 'w5',
    note: 'GIT-038: nhánh sửa lỗi lỡ tạo từ nhánh tính năng, rebase --onto main, merge fast-forward, push',
    run: (P) => {
      g(P.minh, ['switch', '-q', '-c', ROUND_BRANCH]);
      edit(P.minh, 'src/BaoGia/WaitingFee.cs', WAITING_SECONDS);
      g(P.minh, ['commit', '-q', '-am', 'Tính phí chờ từ số giây, làm tròn lên phút'], M('10:00', 25));
      g(P.minh, ['rebase', '-q', '--onto', 'main', NIGHT_BRANCH, ROUND_BRANCH], C('10:05', 25));
      g(P.minh, ['switch', '-q', 'main']);
      g(P.minh, ['merge', '-q', '--ff-only', ROUND_BRANCH]);
      g(P.minh, ['push', '-q']);
      g(P.minh, ['branch', '-q', '-d', ROUND_BRANCH]);
    },
  },
  {
    id: 'w6',
    note: 'GIT-039: Lan gửi patch, Minh git am; bật rerere và thử merge main vào nhánh tính năng để ghi nhớ cách giải quyết',
    run: (P) => {
      g(P.lan, ['fetch', '-q', 'upstream']);
      g(P.lan, ['switch', '-q', '-c', LAN_BRANCH, 'upstream/main']);
      edit(P.lan, 'docs/phi-cho.md', DOC_LAN);
      g(P.lan, ['commit', '-q', '-am', 'Ghi rõ phí chờ tối đa tính cho mỗi chuyến'], { as: 'lan', at: '2026-11-25T11:00:00+07:00' });
      const patch = g(P.lan, ['format-patch', '-1', '-o', '../patches']).stdout.trim();
      g(P.minh, ['am', '-q', join(P.lan, patch)], C('11:30', 25));
      g(P.minh, ['push', '-q']);
      g(P.lan, ['switch', '-q', 'main']);
      g(P.lan, ['branch', '-q', '-D', LAN_BRANCH]);
      g(P.minh, ['config', 'rerere.enabled', 'true']);
      g(P.minh, ['switch', '-q', NIGHT_BRANCH]);
      g(P.minh, ['merge', '-q', 'main'], { ...M('13:00', 25), allowFail: true });
      edit(P.minh, 'docs/phi-cho.md', DOC_RESOLVED);
      g(P.minh, ['add', 'docs/phi-cho.md']);
      g(P.minh, ['merge', '--continue'], M('13:00', 25));
      g(P.minh, ['reset', '-q', '--hard', 'HEAD~1']);
      g(P.minh, ['switch', '-q', 'main']);
    },
  },
  {
    id: 'w7',
    note: 'GIT-040: merge nhánh phụ phí đêm (rerere tự giải quyết), push, xoá nhánh, thêm runbook hoàn tác',
    run: (P) => {
      g(P.minh, ['merge', '-q', '--no-ff', NIGHT_BRANCH], { ...M('14:00', 25), allowFail: true });
      g(P.minh, ['add', 'docs/phi-cho.md']);
      g(P.minh, ['merge', '--continue'], M('14:00', 25));
      g(P.minh, ['push', '-q']);
      g(P.minh, ['branch', '-q', '-d', NIGHT_BRANCH]);
      writeText(P.minh, 'docs/runbook-hoan-tac.md', RUNBOOK);
      g(P.minh, ['add', 'docs/runbook-hoan-tac.md']);
      g(P.minh, ['commit', '-q', '-m', 'Thêm runbook hoàn tác'], M('15:00', 25));
      g(P.minh, ['push', '-q']);
    },
  },
];

/** Dựng Phần 1–3 rồi n bước của Phần 4. */
export function build4(dir, n = STEPS.length) {
  build3(dir);
  const P = places(dir);
  for (const s of STEPS.slice(0, n)) s.run(P);
  return P;
}

const cache = new Map();
/** Ref chuẩn của Minh, remote chung và Lan sau n bước của Phần 4. */
export function expectedRefs4(tmpDir, n) {
  if (!cache.has(n)) {
    const P = build4(tmpDir, n);
    cache.set(n, { minh: refsOf3(P.minh), remote: refsOf3(P.remote), lan: refsOf3(P.lan) });
  }
  return cache.get(n);
}
