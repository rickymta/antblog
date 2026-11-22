// Lịch sử chuẩn của Phần 2: nhánh, merge, xung đột, stash, tag, cherry-pick, revert, rebase, worktree.
//
// Xuất phát từ cuối Phần 1 (10 commit, HEAD 89855d5). Mỗi bước (step) là một thao tác git có người
// làm và thời điểm cố định. `build2(dir, n)` dựng Phần 1 rồi chạy n bước đầu, không in gì. Các bài
// so ref của repository mình tạo ra (nhánh, tag) với `expectedRefs2()` để chắc chắn trùng mã hash.

import { join } from 'node:path';

import { CALC_V3, README, PRICE_DOC_V2, SETTINGS_V2, TESTS_V2, build } from './history.mjs';
import { gitQuiet, readText, writeText } from './lab.mjs';

export const FEATURE = 'tinh-nang/phu-phi-gio-cao-diem';
export const RELEASE = 'phat-hanh/1.0';
export const WAITING = 'tinh-nang/phi-cho-doi';

// ---------- Nội dung file ----------

/** p1 (nhánh tính năng): cấu hình giờ cao điểm. */
export const SETTINGS_PEAK = SETTINGS_V2.replace(
  '  }\n}\n',
  `  },
  "Peak": {
    "Multiplier": 1.2,
    "Hours": ["07:00-09:00", "17:00-19:30"]
  }
}
`,
);

const FARE_LINE = '        var fare = rule.BaseFare + rule.PerKm * distanceKm;';
const SIGNATURE = '    public static decimal Quote(FareRule rule, decimal distanceKm)';
const SIGNATURE_PEAK = '    public static decimal Quote(FareRule rule, decimal distanceKm, decimal peakMultiplier = 1m)';
const COMMENT_ANCHOR = '        // Chưa gồm phụ phí giờ cao điểm, phí cầu đường, khuyến mãi.';
const COMMENT_PEAK = '        // Giờ cao điểm: nhân hệ số Peak.Multiplier (mặc định 1, ngoài giờ cao điểm).\n        // Chưa gồm phí cầu đường, khuyến mãi.';
const ROUND_KM = 'Math.Round(distanceKm, 1, MidpointRounding.AwayFromZero)';

/** p2 (nhánh tính năng): Quote nhận hệ số giờ cao điểm. */
export const CALC_PEAK = CALC_V3.replace(SIGNATURE, SIGNATURE_PEAK)
  .replace(COMMENT_ANCHOR, COMMENT_PEAK)
  .replace(FARE_LINE, '        var fare = (rule.BaseFare + rule.PerKm * distanceKm) * peakMultiplier;');

/** m2 (main, sửa gấp): làm tròn quãng đường tới 0,1 km trước khi tính. Đụng đúng dòng p2 đã sửa. */
export const CALC_ROUND_KM = CALC_V3.replace(FARE_LINE, `        var fare = rule.BaseFare + rule.PerKm * ${ROUND_KM};`);

/** Kết quả giải quyết xung đột khi merge nhánh tính năng (GIT-014): giữ cả hai ý. */
export const CALC_MERGED = CALC_PEAK.replace(
  '        var fare = (rule.BaseFare + rule.PerKm * distanceKm) * peakMultiplier;',
  `        var fare = (rule.BaseFare + rule.PerKm * ${ROUND_KM}) * peakMultiplier;`,
);

/** m2: test cho làm tròn quãng đường. 3,12 km → 3,1 km: 12.000 + 4.500 × 3,1 = 25.950 → 26.000 đ. */
export const TESTS_ROUND_KM = TESTS_V2.replace(
  '\n    [Fact]\n    public void Quote_QuangDuongAm_BaoLoi',
  `
    [Fact]
    public void Quote_LamTronQuangDuong01Km() =>
        Assert.Equal(26000m, FareCalculator.Quote(Car4, 3.12m));

    [Fact]
    public void Quote_QuangDuongAm_BaoLoi`,
);

/** p3: test giờ cao điểm. (12.000 + 4.500 × 4) × 1,2 = 36.000 đ. */
export const TESTS_PEAK = TESTS_ROUND_KM.replace(
  '\n    [Fact]\n    public void Quote_QuangDuongAm_BaoLoi',
  `
    [Fact]
    public void Quote_GioCaoDiem_NhanHeSo() =>
        Assert.Equal(36000m, FareCalculator.Quote(Car4, 4m, 1.2m));

    [Fact]
    public void Quote_QuangDuongAm_BaoLoi`,
);

/** d0 (nhánh sửa tài liệu, merge fast-forward). */
export const PRICE_DOC_ROUND = PRICE_DOC_V2.replace(
  'Giá làm tròn lên bội số 1.000 đ.',
  'Quãng đường làm tròn tới 0,1 km trước khi tính. Giá làm tròn lên bội số 1.000 đ.',
);

/** d1 (nhánh tài liệu giờ cao điểm, merge --no-ff). */
export const PRICE_DOC_PEAK = `${PRICE_DOC_ROUND}
## Phụ phí giờ cao điểm

Từ 07:00 tới 09:00 và từ 17:00 tới 19:30, cước nhân hệ số 1,2.
`;

/** Dòng 3 của README: n1 (Lan, nhánh phí chờ) và m3 (Minh, main) cùng sửa → xung đột khi rebase. */
const README_LINE = 'Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường và loại dịch vụ,';
export const README_WAIT = README.replace(README_LINE, 'Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường, thời gian chờ và loại dịch vụ,');
export const README_HOUR = README.replace(README_LINE, 'Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường, giờ đặt và loại dịch vụ,');
export const README_BOTH = README.replace(
  README_LINE,
  'Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường, giờ đặt, thời gian chờ và loại dịch vụ,',
);

/** n2 (nhánh phí chờ): tài liệu phí chờ. */
export const WAIT_DOC = `# Phí chờ

Tài xế chờ quá 5 phút tại điểm đón: mỗi phút tiếp theo tính 1.000 đ, tối đa 30.000 đ.
`;

/** r2 (nhánh phát hành, làm trong worktree): ghi chú phát hành. */
export const CHANGELOG_10 = `# Ghi chú phát hành

## 1.0.1

- Làm tròn quãng đường tới 0,1 km trước khi tính cước.

## 1.0.0

- Tính cước theo km, giá tối thiểu, làm tròn lên bội số 1.000 đ.
`;

export const TAG_MSG = {
  'v1.0.0': 'Phiên bản 1.0.0: tính cước theo km, làm tròn 1.000 đ',
  'v1.1.0': 'Phiên bản 1.1.0: phụ phí giờ cao điểm, làm tròn quãng đường 0,1 km',
  'v1.0.1': 'Phiên bản 1.0.1: bản vá làm tròn quãng đường',
};

// ---------- Các bước ----------

const g = (dir, args, opts) => gitQuiet(dir, args, opts);
const commit = (dir, msg, who, at, extra = []) =>
  g(dir, ['commit', '-q', ...msg.flatMap((m) => ['-m', m]), ...extra], { as: who, at });

export const STEPS = [
  {
    id: 's1',
    note: 'GIT-011: tạo nhánh tính năng, commit p1',
    run: (d) => {
      g(d, ['switch', '-q', '-c', FEATURE]);
      writeText(d, 'src/BaoGia/appsettings.json', SETTINGS_PEAK);
      g(d, ['add', '-A']);
      commit(d, ['Thêm hệ số phụ phí giờ cao điểm vào bảng giá'], 'minh', '2026-11-16T09:00:00+07:00');
    },
  },
  {
    id: 's2',
    note: 'GIT-012: commit p2 trên nhánh tính năng',
    run: (d) => {
      writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_PEAK);
      g(d, ['add', '-A']);
      commit(d, ['Nhân hệ số giờ cao điểm khi tính cước'], 'minh', '2026-11-16T10:30:00+07:00');
    },
  },
  {
    id: 's3',
    note: 'GIT-012: về main, sửa gấp m2',
    run: (d) => {
      g(d, ['switch', '-q', 'main']);
      writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_ROUND_KM);
      writeText(d, 'tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_ROUND_KM);
      g(d, ['add', '-A']);
      commit(
        d,
        ['Làm tròn quãng đường tới 0,1 km trước khi tính cước', 'GPS trả quãng đường tới mét; khách thấy giá nhảy khi xe đứng yên.'],
        'minh',
        '2026-11-16T11:15:00+07:00',
      );
    },
  },
  {
    id: 's4',
    note: 'GIT-013: nhánh sửa tài liệu, merge fast-forward',
    run: (d) => {
      g(d, ['switch', '-q', '-c', 'sua/tai-lieu-lam-tron']);
      writeText(d, 'docs/bang-gia-cuoc.md', PRICE_DOC_ROUND);
      g(d, ['add', '-A']);
      commit(d, ['Ghi cách làm tròn quãng đường vào tài liệu'], 'lan', '2026-11-16T14:00:00+07:00');
      g(d, ['switch', '-q', 'main']);
      g(d, ['merge', '-q', 'sua/tai-lieu-lam-tron']);
      g(d, ['branch', '-q', '-d', 'sua/tai-lieu-lam-tron']);
    },
  },
  {
    id: 's5',
    note: 'GIT-013: nhánh tài liệu giờ cao điểm, merge --no-ff',
    run: (d) => {
      g(d, ['switch', '-q', '-c', 'tai-lieu/gio-cao-diem']);
      writeText(d, 'docs/bang-gia-cuoc.md', PRICE_DOC_PEAK);
      g(d, ['add', '-A']);
      commit(d, ['Mô tả phụ phí giờ cao điểm trong tài liệu'], 'lan', '2026-11-16T15:00:00+07:00');
      g(d, ['switch', '-q', 'main']);
      g(d, ['merge', '-q', '--no-ff', '--no-edit', 'tai-lieu/gio-cao-diem'], { as: 'minh', at: '2026-11-16T15:30:00+07:00' });
      g(d, ['branch', '-q', '-d', 'tai-lieu/gio-cao-diem']);
    },
  },
  {
    id: 's6',
    note: 'GIT-014: merge nhánh tính năng, xung đột ở dòng tính cước',
    run: (d) => {
      const r = g(d, ['merge', '--no-edit', FEATURE], { as: 'minh', at: '2026-11-17T09:00:00+07:00', allowFail: true });
      if (r.status === 0) throw new Error('s6: lẽ ra merge phải xung đột');
      writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_MERGED);
      g(d, ['add', 'src/BaoGia/FareCalculator.cs']);
      g(d, ['merge', '--continue'], { as: 'minh', at: '2026-11-17T09:00:00+07:00' });
      g(d, ['branch', '-q', '-d', FEATURE]);
    },
  },
  {
    id: 's7',
    note: 'GIT-015: test giờ cao điểm (sau khi stash/pop)',
    run: (d) => {
      writeText(d, 'tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_PEAK);
      g(d, ['add', '-A']);
      commit(d, ['Thêm test cho phụ phí giờ cao điểm'], 'minh', '2026-11-17T10:20:00+07:00');
    },
  },
  {
    id: 's8',
    note: 'GIT-016: tag v1.0.0 và v1.1.0',
    run: (d, refs) => {
      g(d, ['tag', '-a', 'v1.0.0', '-m', TAG_MSG['v1.0.0'], refs.p1end], { as: 'minh', at: '2026-11-17T11:00:00+07:00' });
      g(d, ['tag', '-a', 'v1.1.0', '-m', TAG_MSG['v1.1.0']], { as: 'minh', at: '2026-11-17T11:05:00+07:00' });
    },
  },
  {
    id: 's9',
    note: 'GIT-017: nhánh phát hành 1.0, cherry-pick bản vá, tag v1.0.1',
    run: (d, refs) => {
      g(d, ['switch', '-q', '-c', RELEASE, 'v1.0.0']);
      g(d, ['cherry-pick', '-x', refs.m2], { as: 'minh', at: '2026-11-18T09:00:00+07:00' });
      g(d, ['tag', '-a', 'v1.0.1', '-m', TAG_MSG['v1.0.1']], { as: 'minh', at: '2026-11-18T09:10:00+07:00' });
      g(d, ['switch', '-q', 'main']);
    },
  },
  {
    id: 's10',
    note: 'GIT-017: revert cấu hình giờ cao điểm trên main',
    run: (d, refs) => {
      g(d, ['revert', '--no-edit', refs.p1], { as: 'lan', at: '2026-11-18T14:00:00+07:00' });
    },
  },
  {
    id: 's11',
    note: 'GIT-018: nhánh phí chờ của Lan (từ v1.1.0), main sửa README, rebase có xung đột, merge ff',
    run: (d) => {
      g(d, ['switch', '-q', '-c', WAITING, 'v1.1.0']);
      writeText(d, 'README.md', README_WAIT);
      g(d, ['add', '-A']);
      commit(d, ['Nhắc thời gian chờ trong mô tả dịch vụ'], 'lan', '2026-11-18T10:00:00+07:00');
      writeText(d, 'docs/phi-cho.md', WAIT_DOC);
      g(d, ['add', '-A']);
      commit(d, ['Thêm tài liệu phí chờ'], 'lan', '2026-11-18T11:00:00+07:00');
      g(d, ['switch', '-q', 'main']);
      writeText(d, 'README.md', README_HOUR);
      g(d, ['add', '-A']);
      commit(d, ['Nhắc giờ đặt chuyến trong mô tả dịch vụ'], 'minh', '2026-11-18T15:00:00+07:00');
      g(d, ['switch', '-q', WAITING]);
      const r = g(d, ['rebase', 'main'], { as: 'lan', at: '2026-11-18T16:00:00+07:00', allowFail: true });
      if (r.status === 0) throw new Error('s11: lẽ ra rebase phải xung đột');
      writeText(d, 'README.md', README_BOTH);
      g(d, ['add', 'README.md']);
      g(d, ['rebase', '--continue'], { as: 'lan', at: '2026-11-18T16:00:00+07:00' });
      g(d, ['switch', '-q', 'main']);
      g(d, ['merge', '-q', '--ff-only', WAITING]);
      g(d, ['branch', '-q', '-d', WAITING]);
    },
  },
  {
    id: 's12',
    note: 'GIT-019: worktree cho nhánh phát hành, ghi chú phát hành',
    run: (d) => {
      const wt = join(d, '..', 'bao-gia-1.0');
      g(d, ['worktree', 'add', '-q', wt, RELEASE]);
      writeText(wt, 'CHANGELOG.md', CHANGELOG_10);
      g(wt, ['add', '-A']);
      commit(wt, ['Thêm ghi chú phát hành 1.0.1'], 'minh', '2026-11-19T09:00:00+07:00');
      g(d, ['worktree', 'remove', wt]);
    },
  },
];

/** Hash của các commit P1/P2 cần dùng làm tham số cho bước sau (tính trong lúc dựng). */
function collect(d, refs, id) {
  const rev = (r) => g(d, ['rev-parse', r]).stdout.trim();
  if (id === 's1') refs.p1 = rev('HEAD');
  if (id === 's3') refs.m2 = rev('HEAD');
}

/** Dựng Phần 1 đầy đủ rồi chạy n bước của Phần 2. */
export function build2(dir, n = STEPS.length) {
  build(dir);
  const refs = { p1end: g(dir, ['rev-parse', 'HEAD']).stdout.trim() };
  for (const s of STEPS.slice(0, n)) {
    s.run(dir, refs);
    collect(dir, refs, s.id);
  }
  return refs;
}

/** Mọi ref (nhánh, tag) sau n bước, dạng { 'refs/heads/main': hash, ... }. */
export function refsOf(dir) {
  const out = g(dir, ['for-each-ref', '--format=%(refname) %(objectname)']).stdout.trim();
  return Object.fromEntries(out ? out.split('\n').map((l) => l.split(' ')) : []);
}

const refsCache = new Map();
export function expectedRefs2(tmpDir, n) {
  if (!refsCache.has(n)) {
    build2(tmpDir, n);
    refsCache.set(n, refsOf(tmpDir));
  }
  return refsCache.get(n);
}

export { readText };
