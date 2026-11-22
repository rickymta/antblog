// Lịch sử chuẩn của repository `bao-gia` (dịch vụ báo giá của Tới Nơi) trong Phần 1.
//
// Mỗi commit có người viết, thời điểm và nội dung cố định. `build(dir, n)` dựng repository
// với n commit đầu tiên mà không in gì; các bài dùng nó để dựng trạng thái đầu bài và để
// kiểm tra rằng thao tác trong bài tạo ra ĐÚNG commit này (cùng mã hash).

import { freshDir, gitQuiet, moveFile, writeText } from './lab.mjs';

// ---------- Nội dung file ----------

export const README = `# bao-gia

Dịch vụ báo giá cước của Tới Nơi: nhận quãng đường và loại dịch vụ,
trả về giá dự kiến cho khách trước khi đặt chuyến.
`;

/** FareCalculator.cs lúc mới viết (c2). */
export const CALC_V1 = `namespace ToiNoi.BaoGia;

/// <summary>Bảng giá của một loại dịch vụ, đơn vị đồng.</summary>
public sealed record FareRule(decimal BaseFare, decimal PerKm, decimal MinimumFare);

public static class FareCalculator
{
    /// <summary>Giá dự kiến cho một chuyến dài distanceKm km.</summary>
    public static decimal Quote(FareRule rule, decimal distanceKm)
    {
        if (distanceKm < 0)
            throw new ArgumentOutOfRangeException(nameof(distanceKm));

        // Giá = giá mở cửa + đơn giá mỗi km × số km.
        // Ví dụ ô tô 4 chỗ đi 4 km: 12.000 + 4.500 × 4 = 30.000 đ.
        // Chuyến quá ngắn vẫn trả ít nhất giá tối thiểu.
        // Chưa gồm phụ phí giờ cao điểm, phí cầu đường, khuyến mãi.
        var fare = rule.BaseFare + rule.PerKm * distanceKm;
        var atLeastMinimum = Math.Max(fare, rule.MinimumFare);

        return atLeastMinimum;
    }
}
`;

/** c6: thông báo lỗi rõ hơn (hunk thứ nhất của GIT-005). */
export const CALC_V2 = CALC_V1.replace(
  'throw new ArgumentOutOfRangeException(nameof(distanceKm));',
  'throw new ArgumentOutOfRangeException(\n                nameof(distanceKm), distanceKm, "Quãng đường không được âm.");',
);

/** c7: làm tròn lên bội số 1.000 đ (hunk thứ hai của GIT-005). */
export const CALC_V3 = CALC_V2.replace(
  '        return atLeastMinimum;\n',
  '        // Làm tròn lên bội số 1.000 đ để khách không phải trả tiền lẻ.\n        return Math.Ceiling(atLeastMinimum / 1000m) * 1000m;\n',
);

/** Bản sửa trong working tree trước khi tách thành hai commit (GIT-005) = CALC_V3. */
export const CALC_TWO_CHANGES = CALC_V3;

export const SETTINGS_V1 = `{
  "Fare": {
    "Car4": {
      "BaseFare": 12000,
      "PerKm": 4500,
      "MinimumFare": 25000
    },
    "Bike": {
      "BaseFare": 8000,
      "PerKm": 2500,
      "MinimumFare": 12000
    }
  }
}
`;

/** c5: Lan tăng đơn giá mỗi km của ô tô 4 chỗ. */
export const SETTINGS_V2 = SETTINGS_V1.replace('"PerKm": 4500', '"PerKm": 5000');

export const GITIGNORE = `# Kết quả build của .NET
bin/
obj/

# File riêng của IDE
.vs/
*.user

# Cấu hình và bí mật trên máy từng người
appsettings.Development.json
.env
`;

export const TESTS_V1 = `using ToiNoi.BaoGia;
using Xunit;

namespace ToiNoi.BaoGia.Tests;

public class FareCalculatorTests
{
    private static readonly FareRule Car4 = new(BaseFare: 12000m, PerKm: 4500m, MinimumFare: 25000m);

    [Fact]
    public void Quote_ChuyenDai_TinhTheoKm() =>
        Assert.Equal(30000m, FareCalculator.Quote(Car4, 4m));

    [Fact]
    public void Quote_ChuyenNgan_TraGiaToiThieu() =>
        Assert.Equal(25000m, FareCalculator.Quote(Car4, 1m));

    [Fact]
    public void Quote_QuangDuongAm_BaoLoi() =>
        Assert.Throws<ArgumentOutOfRangeException>(() => FareCalculator.Quote(Car4, -1m));
}
`;

/** c7 (sau amend): thêm test làm tròn. 12.000 + 4.500 × 3,3 = 26.850 → 27.000 đ. */
export const TESTS_V2 = TESTS_V1.replace(
  '\n    [Fact]\n    public void Quote_QuangDuongAm_BaoLoi',
  `
    [Fact]
    public void Quote_LamTronLenBoi1000() =>
        Assert.Equal(27000m, FareCalculator.Quote(Car4, 3.3m));

    [Fact]
    public void Quote_QuangDuongAm_BaoLoi`,
);

export const PRICE_DOC = `# Bảng giá cước

| Dịch vụ | Giá mở cửa | Mỗi km | Tối thiểu |
|---|---|---|---|
| Ô tô 4 chỗ | 12.000 đ | 5.000 đ | 25.000 đ |
| Xe máy | 8.000 đ | 2.500 đ | 12.000 đ |

Giá làm tròn lên bội số 1.000 đ. Chưa gồm phụ phí giờ cao điểm.
`;

export const PRICE_DOC_V2 = `${PRICE_DOC}
Nguồn số liệu: \`src/BaoGia/appsettings.json\`.
`;

export const GITATTRIBUTES = `# Mọi file văn bản lưu trong repository với xuống dòng LF
* text=auto eol=lf

# Script Windows cần CRLF khi checkout
*.cmd text eol=crlf
*.bat text eol=crlf

# File nhị phân: không diff, không đổi xuống dòng
*.png binary
*.webp binary
`;

// ---------- Các commit ----------

export const COMMITS = [
  {
    id: 'c1',
    as: 'minh',
    at: '2026-11-02T08:30:00+07:00',
    message: ['Khởi tạo dịch vụ báo giá'],
    apply: (d) => writeText(d, 'README.md', README),
  },
  {
    id: 'c2',
    as: 'minh',
    at: '2026-11-02T09:10:00+07:00',
    message: ['Thêm hàm tính cước theo km'],
    apply: (d) => {
      writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_V1);
      writeText(d, 'src/BaoGia/appsettings.json', SETTINGS_V1);
    },
  },
  {
    id: 'c3',
    as: 'minh',
    at: '2026-11-02T09:25:00+07:00',
    message: ['Bỏ qua kết quả build và cấu hình máy cá nhân'],
    apply: (d) => writeText(d, '.gitignore', GITIGNORE),
  },
  {
    id: 'c4',
    as: 'lan',
    at: '2026-11-03T14:05:00+07:00',
    message: ['Thêm test cho hàm tính cước'],
    apply: (d) => writeText(d, 'tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_V1),
  },
  {
    id: 'c5',
    as: 'lan',
    at: '2026-11-04T10:40:00+07:00',
    message: [
      'Tăng cước mỗi km ô tô 4 chỗ lên 5.000 đ',
      'Theo bảng giá mới của bộ phận vận hành, áp dụng từ 01/12/2026.',
    ],
    apply: (d) => writeText(d, 'src/BaoGia/appsettings.json', SETTINGS_V2),
  },
  {
    id: 'c6',
    as: 'minh',
    at: '2026-11-05T09:00:00+07:00',
    message: ['Báo lỗi rõ hơn khi quãng đường âm'],
    apply: (d) => writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_V2),
  },
  {
    id: 'c7',
    as: 'minh',
    at: '2026-11-05T09:05:00+07:00',
    committedAt: '2026-11-05T09:12:00+07:00',
    message: [
      'Làm tròn cước lên bội số 1.000 đ',
      'Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối.',
    ],
    apply: (d) => {
      writeText(d, 'src/BaoGia/FareCalculator.cs', CALC_V3);
      writeText(d, 'tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_V2);
    },
  },
  {
    id: 'c8',
    as: 'minh',
    at: '2026-11-06T16:20:00+07:00',
    message: ['Thêm tài liệu bảng giá'],
    apply: (d) => writeText(d, 'docs/bang-gia.md', PRICE_DOC),
  },
  {
    id: 'c9',
    as: 'lan',
    at: '2026-11-09T11:00:00+07:00',
    message: ['Đổi tên tài liệu thành bang-gia-cuoc.md'],
    apply: (d) => {
      moveFile(d, 'docs/bang-gia.md', 'docs/bang-gia-cuoc.md');
      writeText(d, 'docs/bang-gia-cuoc.md', PRICE_DOC_V2);
    },
  },
  {
    id: 'c10',
    as: 'minh',
    at: '2026-11-10T08:45:00+07:00',
    message: ['Chuẩn hoá xuống dòng LF bằng .gitattributes'],
    apply: (d) => writeText(d, '.gitattributes', GITATTRIBUTES),
  },
];

/** Đối số `-m` cho git commit từ message của một commit. */
export function messageArgs(message) {
  return message.flatMap((m) => ['-m', m]);
}

/** Cấu hình riêng của repository lab: danh tính mặc định khi bạn commit bằng GitStudio. */
export function localIdentity(dir, who = 'minh') {
  const p = who === 'lan' ? ['Lê Lan', 'lan@toinoi.test'] : ['Trần Minh', 'minh@toinoi.test'];
  gitQuiet(dir, ['config', 'user.name', p[0]]);
  gitQuiet(dir, ['config', 'user.email', p[1]]);
}

/** Dựng repository với n commit đầu tiên của lịch sử chuẩn (n = 0: chỉ init). */
export function build(dir, n = COMMITS.length) {
  freshDir(dir);
  gitQuiet(dir, ['init', '-q', '-b', 'main']);
  localIdentity(dir);
  for (const c of COMMITS.slice(0, n)) commitStep(dir, c);
  return dir;
}

export function commitStep(dir, c) {
  c.apply(dir);
  gitQuiet(dir, ['add', '-A']);
  gitQuiet(dir, ['commit', '-q', ...messageArgs(c.message)], {
    as: c.as,
    at: c.at,
    committedAt: c.committedAt,
  });
}

/** Mã hash của commit thứ n (1..10) trong lịch sử chuẩn, tính bằng cách dựng repository tạm. */
const hashCache = new Map();
export function expectedHashes(tmpDir) {
  if (hashCache.size) return hashCache;
  build(tmpDir);
  const out = gitQuiet(tmpDir, ['rev-list', '--reverse', 'HEAD']).stdout.trim().split('\n');
  out.forEach((h, i) => hashCache.set(COMMITS[i].id, h));
  return hashCache;
}
