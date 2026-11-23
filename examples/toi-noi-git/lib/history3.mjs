// Lịch sử chuẩn của Phần 3: remote, fetch/pull/push, push bị từ chối, pull request (mô phỏng),
// bảo vệ nhánh bằng hook, fork, pull làm rối lịch sử.
//
// "GitHub" của lab là các bare repository trong work/remote/, nên không cần tài khoản hay mạng.
//   work/bao-gia                  repository của Minh (cuối Phần 2)
//   work/remote/bao-gia.git       remote chung của nhóm (origin)
//   work/lan/bao-gia              bản clone của Lan
//   work/remote/lan-bao-gia.git   fork của Lan (GIT-028)
//
// `build3(dir, n)` dựng Phần 1 + Phần 2 rồi chạy n bước của Phần 3. URL remote là đường dẫn tuyệt
// đối nên khác nhau giữa các máy, nhưng không đi vào commit nào: mã hash vẫn tái lập được.

import { existsSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';

import { build2 } from './history2.mjs';
import { freshDir, gitQuiet, readText, writeText } from './lab.mjs';
import { localIdentity } from './history.mjs';

export const PAYMENT_BRANCH = 'tinh-nang/tinh-phi-cho';
export const FORK_BRANCH = 'sua/tai-lieu-phi-cho';

/** Đường dẫn các repository của Phần 3, tính từ repository của Minh. */
export function places(dir) {
  const root = dirname(dir);
  const slash = (p) => p.split('\\').join('/');
  return {
    minh: dir,
    remote: slash(join(root, 'remote', 'bao-gia.git')),
    lan: join(root, 'lan', 'bao-gia'),
    fork: slash(join(root, 'remote', 'lan-bao-gia.git')),
  };
}

// ---------- Nội dung file ----------

export const WAIT_DOC_L1 = (doc) => `${doc}\nKhông tính phí chờ khi khách huỷ chuyến trước khi xe tới.\n`;
export const README_REMOTE = (r) => `${r}\nRepository chung của nhóm: remote \`origin\`. Mọi thay đổi vào \`main\` qua pull request.\n`;
export const PRICE_DOC_CONTACT = (d) => `${d}\nCần đổi bảng giá: liên hệ bộ phận vận hành, không sửa trực tiếp trên server.\n`;

export const WAITING_FEE_V1 = `namespace ToiNoi.BaoGia;

/// <summary>Phí chờ tại điểm đón: 1.000 đ mỗi phút, tối đa 30.000 đ.</summary>
public static class WaitingFee
{
    public static decimal Calculate(int waitedMinutes)
    {
        if (waitedMinutes < 0)
            throw new ArgumentOutOfRangeException(nameof(waitedMinutes));

        return Math.Min(waitedMinutes * 1000m, 30000m);
    }
}
`;

/** Sau review: 5 phút đầu miễn phí (đúng như docs/phi-cho.md). */
export const WAITING_FEE_V2 = WAITING_FEE_V1.replace(
  '/// <summary>Phí chờ tại điểm đón: 1.000 đ mỗi phút, tối đa 30.000 đ.</summary>',
  '/// <summary>Phí chờ tại điểm đón: miễn phí 5 phút đầu, sau đó 1.000 đ mỗi phút, tối đa 30.000 đ.</summary>',
).replace(
  '        return Math.Min(waitedMinutes * 1000m, 30000m);',
  '        var billable = Math.Max(waitedMinutes - FreeMinutes, 0);\n        return Math.Min(billable * 1000m, 30000m);',
).replace(
  'public static class WaitingFee\n{\n',
  'public static class WaitingFee\n{\n    private const int FreeMinutes = 5;\n\n',
);

export const WAITING_TESTS_V1 = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class WaitingFeeTests
{
    [Fact]
    public void Calculate_MoiPhut1000() => Assert.Equal(12000m, WaitingFee.Calculate(12));

    [Fact]
    public void Calculate_ToiDa30000() => Assert.Equal(30000m, WaitingFee.Calculate(90));
}
`;

export const WAITING_TESTS_V2 = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class WaitingFeeTests
{
    [Fact]
    public void Calculate_NamPhutDauMienPhi() => Assert.Equal(0m, WaitingFee.Calculate(5));

    [Fact]
    public void Calculate_MoiPhutSauDo1000() => Assert.Equal(7000m, WaitingFee.Calculate(12));

    [Fact]
    public void Calculate_ToiDa30000() => Assert.Equal(30000m, WaitingFee.Calculate(90));
}
`;

export const FORK_DOC = (d) => d.replace('mỗi phút tiếp theo tính 1.000 đ', 'mỗi phút tiếp theo tính thêm 1.000 đ');
export const README_FORK_UPSTREAM = (r) => `${r}\nBáo lỗi và đề xuất: mở issue trên repository chung.\n`;
export const README_PULL_LAN = (r) => `${r}\nChạy toàn bộ test trước khi push: \`dotnet test\`.\n`;
export const PRICE_DOC_PULL_MINH = (d) => `${d}\nBảng giá có hiệu lực theo giờ Việt Nam (UTC+7).\n`;

/** Hook pre-receive của remote: bảo vệ main và tag phát hành (GIT-027). */
export const PROTECT_HOOK = `#!/bin/sh
# Bảo vệ nhánh main và tag phát hành trên remote của nhóm Tới Nơi.
zero=0000000000000000000000000000000000000000
while read old new ref; do
  case "$ref" in
    refs/heads/main)
      if [ "$new" = "$zero" ]; then
        echo "Từ chối: không được xoá main"; exit 1
      fi
      if [ "$old" != "$zero" ] && ! git merge-base --is-ancestor "$old" "$new"; then
        echo "Từ chối: main được bảo vệ, không cho push ghi đè lịch sử"; exit 1
      fi ;;
    refs/tags/v*)
      if [ "$old" != "$zero" ]; then
        echo "Từ chối: tag phát hành $ref đã có, không được sửa hay xoá"; exit 1
      fi ;;
  esac
done
exit 0
`;

// ---------- Các bước ----------

const g = (d, args, opts) => gitQuiet(d, args, opts);
const commit = (d, msg, opts) => g(d, ['commit', '-q', ...msg.flatMap((m) => ['-m', m])], opts);
const edit = (d, rel, fn) => writeText(d, rel, fn(readText(d, rel)));

export const STEPS = [
  {
    id: 't1',
    note: 'GIT-021: tạo remote chung, Minh push, Lan clone',
    run: (P) => {
      freshDir(P.remote);
      g(dirname(P.remote), ['init', '-q', '--bare', '-b', 'main', P.remote]);
      g(P.minh, ['remote', 'add', 'origin', P.remote]);
      g(P.minh, ['push', '-q', '-u', 'origin', 'main']);
      g(P.minh, ['push', '-q', 'origin', 'phat-hanh/1.0']);
      g(P.minh, ['push', '-q', 'origin', '--tags']);
      freshDir(P.lan);
      g(dirname(P.lan), ['clone', '-q', P.remote, P.lan]);
      localIdentity(P.lan, 'lan');
    },
  },
  {
    id: 't2',
    note: 'GIT-022: Lan push, Minh fetch rồi pull fast-forward',
    run: (P) => {
      edit(P.lan, 'docs/phi-cho.md', WAIT_DOC_L1);
      g(P.lan, ['commit', '-q', '-am', 'Không tính phí chờ khi khách huỷ trước khi xe tới'], { as: 'lan', at: '2026-11-20T09:00:00+07:00' });
      g(P.lan, ['push', '-q']);
      g(P.minh, ['fetch', '-q']);
      g(P.minh, ['pull', '-q', '--ff-only']);
    },
  },
  {
    id: 't3',
    note: 'GIT-023: Lan push trước, push của Minh bị từ chối, Minh pull --rebase rồi push',
    run: (P) => {
      edit(P.lan, 'docs/bang-gia-cuoc.md', PRICE_DOC_CONTACT);
      g(P.lan, ['commit', '-q', '-am', 'Ghi đầu mối đổi bảng giá vào tài liệu'], { as: 'lan', at: '2026-11-20T10:00:00+07:00' });
      g(P.lan, ['push', '-q']);
      edit(P.minh, 'README.md', README_REMOTE);
      g(P.minh, ['commit', '-q', '-am', 'Ghi quy ước dùng remote vào README'], { as: 'minh', at: '2026-11-20T10:05:00+07:00' });
      const r = g(P.minh, ['push', '-q'], { allowFail: true });
      if (r.status === 0) throw new Error('t3: lẽ ra push phải bị từ chối');
      g(P.minh, ['pull', '-q', '--rebase'], { as: 'minh', at: '2026-11-20T10:05:00+07:00', committedAt: '2026-11-20T10:10:00+07:00' });
      g(P.minh, ['push', '-q']);
      g(P.lan, ['pull', '-q', '--ff-only']);
    },
  },
  {
    id: 't4',
    note: 'GIT-026: pull request phí chờ của Lan, review, squash merge, xoá nhánh',
    run: (P) => {
      g(P.lan, ['switch', '-q', '-c', PAYMENT_BRANCH]);
      writeText(P.lan, 'src/BaoGia/WaitingFee.cs', WAITING_FEE_V1);
      writeText(P.lan, 'tests/BaoGia.Tests/WaitingFeeTests.cs', WAITING_TESTS_V1);
      g(P.lan, ['add', '-A']);
      commit(P.lan, ['Tính phí chờ theo phút, tối đa 30.000 đ'], { as: 'lan', at: '2026-11-21T09:00:00+07:00' });
      g(P.lan, ['push', '-q', '-u', 'origin', PAYMENT_BRANCH]);
      writeText(P.lan, 'src/BaoGia/WaitingFee.cs', WAITING_FEE_V2);
      writeText(P.lan, 'tests/BaoGia.Tests/WaitingFeeTests.cs', WAITING_TESTS_V2);
      g(P.lan, ['commit', '-q', '-am', 'Miễn phí 5 phút chờ đầu tiên theo tài liệu'], { as: 'lan', at: '2026-11-21T09:40:00+07:00' });
      g(P.lan, ['push', '-q']);
      g(P.minh, ['fetch', '-q']);
      g(P.minh, ['merge', '-q', '--squash', `origin/${PAYMENT_BRANCH}`]);
      commit(
        P.minh,
        ['Tính phí chờ tại điểm đón', 'Pull request của Lê Lan (2 commit), Trần Minh review: miễn phí 5 phút đầu, tối đa 30.000 đ.'],
        { as: 'lan', by: 'minh', at: '2026-11-21T10:00:00+07:00' },
      );
      g(P.minh, ['push', '-q']);
      g(P.minh, ['push', '-q', 'origin', '--delete', PAYMENT_BRANCH]);
      g(P.lan, ['switch', '-q', 'main']);
      g(P.lan, ['pull', '-q', '--ff-only', '--prune']);
      g(P.lan, ['branch', '-q', '-D', PAYMENT_BRANCH]);
    },
  },
  {
    id: 't5',
    note: 'GIT-027: hook bảo vệ main và tag phát hành trên remote',
    run: (P) => {
      writeText(P.remote, 'hooks/pre-receive', PROTECT_HOOK);
    },
  },
  {
    id: 't6',
    note: 'GIT-028: fork của Lan, đồng bộ upstream, Minh merge từ fork',
    run: (P) => {
      freshDir(P.fork);
      g(dirname(P.fork), ['clone', '-q', '--bare', P.remote, P.fork]);
      g(P.lan, ['remote', 'rename', 'origin', 'upstream']);
      g(P.lan, ['remote', 'add', 'origin', P.fork]);
      g(P.lan, ['fetch', '-q', 'origin']);
      g(P.lan, ['switch', '-q', '-c', FORK_BRANCH]);
      edit(P.lan, 'docs/phi-cho.md', FORK_DOC);
      g(P.lan, ['commit', '-q', '-am', 'Nói rõ phí chờ tính thêm mỗi phút'], { as: 'lan', at: '2026-11-22T09:00:00+07:00' });
      g(P.lan, ['push', '-q', '-u', 'origin', FORK_BRANCH]);
      edit(P.minh, 'README.md', README_FORK_UPSTREAM);
      g(P.minh, ['commit', '-q', '-am', 'Hướng dẫn báo lỗi qua issue'], { as: 'minh', at: '2026-11-22T09:30:00+07:00' });
      g(P.minh, ['push', '-q']);
      g(P.lan, ['fetch', '-q', 'upstream']);
      g(P.lan, ['rebase', '-q', 'upstream/main'], { as: 'lan', at: '2026-11-22T10:00:00+07:00' });
      g(P.lan, ['push', '-q', '--force-with-lease']);
      g(P.minh, ['remote', 'add', 'lan-fork', P.fork]);
      g(P.minh, ['fetch', '-q', 'lan-fork']);
      g(P.minh, ['merge', '-q', '--no-ff', '--no-edit', `lan-fork/${FORK_BRANCH}`], { as: 'minh', at: '2026-11-22T11:00:00+07:00' });
      g(P.minh, ['push', '-q']);
      g(P.minh, ['remote', 'remove', 'lan-fork']);
      g(P.lan, ['switch', '-q', 'main']);
      g(P.lan, ['pull', '-q', '--ff-only', 'upstream', 'main']);
      g(P.lan, ['push', '-q', 'origin', 'main']);
      g(P.lan, ['push', '-q', 'origin', '--delete', FORK_BRANCH]);
      g(P.lan, ['branch', '-q', '-d', FORK_BRANCH]);
    },
  },
  {
    id: 't7',
    note: 'GIT-029: Lan push, Minh có commit riêng; pull --rebase thay cho merge thừa; cấu hình pull.rebase',
    run: (P) => {
      edit(P.lan, 'README.md', README_PULL_LAN);
      g(P.lan, ['commit', '-q', '-am', 'Nhắc chạy test trước khi push'], { as: 'lan', at: '2026-11-23T09:00:00+07:00' });
      g(P.lan, ['push', '-q', 'upstream', 'main']);
      edit(P.minh, 'docs/bang-gia-cuoc.md', PRICE_DOC_PULL_MINH);
      g(P.minh, ['commit', '-q', '-am', 'Ghi múi giờ áp dụng bảng giá'], { as: 'minh', at: '2026-11-23T09:10:00+07:00' });
      g(P.minh, ['config', 'pull.rebase', 'true']);
      g(P.minh, ['pull', '-q'], { as: 'minh', at: '2026-11-23T09:10:00+07:00', committedAt: '2026-11-23T09:20:00+07:00' });
      g(P.minh, ['push', '-q']);
      g(P.lan, ['pull', '-q', '--ff-only', 'upstream', 'main']);
    },
  },
];

/** Dựng Phần 1, Phần 2 rồi n bước của Phần 3. */
export function build3(dir, n = STEPS.length) {
  build2(dir);
  const P = places(dir);
  // Xoá remote, bản clone của Lan và fork còn lại từ lần dựng trước: chúng thuộc các bước sau n.
  for (const p of [P.remote, P.lan, P.fork]) {
    if (!existsSync(p)) continue;
    try {
      rmSync(p, { recursive: true, force: true });
    } catch {
      freshDir(p);
    }
  }
  for (const s of STEPS.slice(0, n)) s.run(P);
  return P;
}

/** Ref của một repository (cục bộ hoặc bare), bỏ refs/remotes/lan-fork vì URL fork khác máy. */
export function refsOf3(dir) {
  const out = gitQuiet(dir, ['for-each-ref', '--format=%(refname) %(objectname)']).stdout.trim();
  return Object.fromEntries(out ? out.split('\n').map((l) => l.split(' ')) : []);
}

const cache = new Map();
/** Ref chuẩn của cả ba repository (Minh, remote, Lan) sau n bước. */
export function expectedRefs3(tmpDir, n) {
  if (!cache.has(n)) {
    const P = build3(tmpDir, n);
    cache.set(n, {
      minh: refsOf3(P.minh),
      remote: n >= 1 ? refsOf3(P.remote) : {},
      lan: n >= 1 ? refsOf3(P.lan) : {},
    });
  }
  return cache.get(n);
}
