//! Sổ bằng chứng: mỗi lý do, cái giá và điều kiện «quyết định sai» của ADR bài 01
//! gắn với loại bằng chứng trong loạt và điều loạt KHÔNG chứng minh.

/// Loại bằng chứng, xếp theo mức chặn từ cứng tới mềm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    /// Trình biên dịch từ chối code sai.
    Compiler,
    /// Test hoặc lệnh kiểm chạy được và đỏ khi bất biến bị phá.
    InvariantTest,
    /// Số đo trên máy viết bài, không phải SLA.
    Measurement,
    /// Chỉ là lập luận: loạt không có thí nghiệm nào.
    Argument,
}

impl Evidence {
    pub const ALL: [Evidence; 4] = [
        Evidence::Compiler,
        Evidence::InvariantTest,
        Evidence::Measurement,
        Evidence::Argument,
    ];

    /// `match` vét cạn, không nhánh `_`: thêm loại mới mà quên nhãn là lỗi E0004.
    pub fn label(self) -> &'static str {
        match self {
            Evidence::Compiler => "biên dịch chặn",
            Evidence::InvariantTest => "test bất biến",
            Evidence::Measurement => "số đo",
            Evidence::Argument => "lập luận",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Reason,
    Cost,
    Condition,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Reason => "Lý do chọn",
            Section::Cost => "Cái giá",
            Section::Condition => "Khi nào sai",
        }
    }
}

pub struct Claim {
    pub id: &'static str,
    pub section: Section,
    pub text: &'static str,
    pub evidence: &'static [Evidence],
    /// Số các bài chứa bằng chứng (hoặc nơi vấn đề được nói tới).
    pub articles: &'static [u8],
    /// Điều bằng chứng KHÔNG chứng minh. Không được để trống.
    pub gap: &'static str,
}

use Evidence::{Argument, Compiler, InvariantTest, Measurement};
use Section::{Condition, Cost, Reason};

/// 15 dòng của sổ. Nội dung lấy từ các bài đã viết; ô `gap` trích giới hạn các bài tự nêu.
pub const CLAIMS: &[Claim] = &[
    Claim {
        id: "R1",
        section: Reason,
        text: "Tiền không lặng lẽ sai: Money(i64) và checked_add",
        evidence: &[Compiler, InvariantTest],
        articles: &[3, 12, 19, 46],
        gap: "release tắt overflow-checks mặc định nên kỷ luật checked_* vẫn là việc người viết; property test không là chứng minh",
    },
    Claim {
        id: "R2",
        section: Reason,
        text: "Mọi trạng thái đều được xử lý: match vét cạn, typestate",
        evidence: &[Compiler],
        articles: &[1, 5, 14, 32, 58],
        gap: "chỉ trong code Rust; ở biên (cột state, JSON) vẫn phải tin dữ liệu đưa vào",
    },
    Claim {
        id: "R3",
        section: Reason,
        text: "Một chủ, không alias sửa dữ liệu: borrow checker và Send/Sync",
        evidence: &[Compiler],
        articles: &[4, 22],
        gap: "chặn data race trong một process; không chặn cuộc đua giữa hai transaction (bài 24, 53)",
    },
    Claim {
        id: "R4",
        section: Reason,
        text: "Append-only, idempotent, bút toán cân",
        evidence: &[InvariantTest],
        articles: &[24, 25, 50, 61, 67],
        gap: "do PostgreSQL bảo đảm (trigger, unique), không do Rust; chủ bảng tắt được trigger (bài 50, bài 67 tắt trong schema tạm)",
    },
    Claim {
        id: "R5",
        section: Reason,
        text: "Độ trễ p99 và bộ nhớ dễ đoán vì không có GC",
        evidence: &[Measurement, Argument],
        articles: &[30, 47, 59, 61],
        gap: "bài 59 chỉ ủng hộ nửa bộ nhớ (đếm được block, RSS nhỏ); chưa đo p99 dưới tải thật, bài 61 chỉ có hướng ở 20 request/giây; số đo ở máy tải cao qua Docker Desktop; so với .NET chỉ ở bài 59, khối lượng nhỏ, không phải production",
    },
    Claim {
        id: "R6",
        section: Reason,
        text: "Thông lượng KHÔNG phải lý do (bài 01 tự nói)",
        evidence: &[Measurement],
        articles: &[30, 61],
        gap: "ở bài 30 chỉ gộp lô unnest vượt 10.000 bút toán/giây, chưa gồm account_balances; bài 61 có idempotency và số dư thì vượt ở load 100 khi chưa có trigger, thêm trigger còn 8.815 đến 9.108, load 430 không đạt; không có phép đo .NET cùng tải",
    },
    Claim {
        id: "C1",
        section: Cost,
        text: "Đường cong học: ownership và borrow checker",
        evidence: &[Argument],
        articles: &[1, 4],
        gap: "loạt không đo thời gian học của đội nào",
    },
    Claim {
        id: "C2",
        section: Cost,
        text: "Tuyển dụng: ít người có kinh nghiệm Rust production",
        evidence: &[Argument],
        articles: &[1],
        gap: "loạt không có dữ liệu thị trường lao động",
    },
    Claim {
        id: "C3",
        section: Cost,
        text: "Thời gian build và số crate",
        evidence: &[Measurement],
        articles: &[51, 58, 59, 63, 64],
        gap: "số crate biên dịch ổn định, thời gian tường nhiễu vì máy tải (26 đến 94 giây cho cùng bước ở bài 63); không có phép đo dotnet build để so",
    },
    Claim {
        id: "C4",
        section: Cost,
        text: "Hệ sinh thái nhỏ hơn: không ORM, rdkafka bọc thư viện C",
        evidence: &[Argument],
        articles: &[23, 26, 36, 63],
        gap: "quan sát khi viết lab, không so độ rộng với NuGet",
    },
    Claim {
        id: "C5",
        section: Cost,
        text: "Hai toolchain: CI, image, triển khai, giám sát",
        evidence: &[Measurement, Argument],
        articles: &[63, 64, 65, 66],
        gap: "đo được image (bài 63) và lệnh CI chạy local (bài 64); workflow chưa chạy trên GitHub, chưa triển khai cloud thật, image chỉ linux/arm64; chi phí on-call hai ngôn ngữ không đo",
    },
    Claim {
        id: "W1",
        section: Condition,
        text: "Đội không giữ được kỹ năng Rust",
        evidence: &[Argument],
        articles: &[1],
        gap: "không có thí nghiệm nào về đội, nhân sự hay bàn giao",
    },
    Claim {
        id: "W2",
        section: Condition,
        text: "Lượng nhỏ và SLO đã đạt trên .NET",
        evidence: &[Measurement, Argument],
        articles: &[30, 59],
        gap: "loạt không có bản .NET của ledger đủ tính năng, không có p99 .NET và chưa đặt SLO nào để đạt; bài 59 chỉ so RSS và GC trên khối lượng nhỏ trong bộ nhớ",
    },
    Claim {
        id: "W3",
        section: Condition,
        text: "Chi phí vận hành hai ngôn ngữ vượt lợi ích",
        evidence: &[Argument],
        articles: &[64, 65, 67],
        gap: "loạt chưa vận hành hệ thống thật nên không đo được công on-call hay chi phí hai ngôn ngữ",
    },
    Claim {
        id: "W4",
        section: Condition,
        text: "Ranh giới Rust bị xói mòn",
        evidence: &[InvariantTest],
        articles: &[51, 55, 58],
        gap: "kiểm ranh giới giữa crate trong một service, không kiểm ranh giới giữa service hay giữa đội; Go và Python mới đọc sự kiện từ tệp",
    },
];

/// Kiểm sổ: id duy nhất, có loại bằng chứng, số bài hợp lệ và tăng dần, ô `gap` không trống.
pub fn check(claims: &[Claim], article_count: u8) -> Vec<String> {
    let mut problems = Vec::new();
    for (i, claim) in claims.iter().enumerate() {
        let id = claim.id;
        if claims[..i].iter().any(|c| c.id == id) {
            problems.push(format!("sổ bằng chứng: id {id} trùng"));
        }
        if claim.evidence.is_empty() {
            problems.push(format!("sổ bằng chứng {id}: chưa có loại bằng chứng"));
        }
        if claim.articles.is_empty() {
            problems.push(format!("sổ bằng chứng {id}: chưa trỏ tới bài nào"));
        }
        if claim.gap.trim().len() < 20 {
            problems.push(format!(
                "sổ bằng chứng {id}: phải nói điều KHÔNG chứng minh"
            ));
        }
        if claim.articles.windows(2).any(|w| w[0] >= w[1]) {
            problems.push(format!("sổ bằng chứng {id}: số bài phải tăng dần"));
        }
        if let Some(bad) = claim
            .articles
            .iter()
            .find(|&&n| n == 0 || n > article_count)
        {
            problems.push(format!("sổ bằng chứng {id}: bài {bad} không tồn tại"));
        }
    }
    problems
}

/// Đếm theo loại. `only_argument` là số dòng không có bằng chứng máy kiểm nào.
#[derive(Debug, PartialEq, Eq)]
pub struct Summary {
    pub total: usize,
    pub per_kind: [usize; 4],
    pub only_argument: usize,
}

pub fn summarize(claims: &[Claim]) -> Summary {
    let mut per_kind = [0; 4];
    for claim in claims {
        for (slot, kind) in Evidence::ALL.iter().enumerate() {
            if claim.evidence.contains(kind) {
                per_kind[slot] += 1;
            }
        }
    }
    Summary {
        total: claims.len(),
        per_kind,
        only_argument: claims.iter().filter(|c| c.evidence == [Argument]).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_ledger_is_clean() {
        assert!(check(CLAIMS, 68).is_empty());
    }

    #[test]
    fn check_rejects_missing_gap_unknown_article_and_duplicates() {
        let bad = [
            Claim {
                id: "X1",
                section: Reason,
                text: "t",
                evidence: &[Measurement],
                articles: &[69],
                gap: "",
            },
            Claim {
                id: "X1",
                section: Cost,
                text: "t",
                evidence: &[],
                articles: &[],
                gap: "một điều đủ dài để qua kiểm độ dài",
            },
        ];
        let problems = check(&bad, 68);
        assert!(problems.iter().any(|p| p.contains("trùng")));
        assert!(problems.iter().any(|p| p.contains("KHÔNG chứng minh")));
        assert!(problems.iter().any(|p| p.contains("bài 69")));
        assert!(problems.iter().any(|p| p.contains("chưa có loại")));
    }

    #[test]
    fn check_is_formal_only_it_cannot_judge_truth() {
        // Một dòng khoác loại «số đo» với lời giới hạn đủ dài vẫn qua kiểm hình thức:
        // việc đối chiếu số với bài là của người review.
        let dressed_up = [Claim {
            id: "X9",
            section: Reason,
            text: "Rust nhanh hơn .NET gấp N lần",
            evidence: &[Measurement],
            articles: &[59],
            gap: "đo ở một máy, khối lượng nhỏ, không phải production",
        }];
        assert!(check(&dressed_up, 68).is_empty());
    }

    #[test]
    fn summary_counts_kinds_and_argument_only() {
        let s = summarize(CLAIMS);
        assert_eq!(s.total, CLAIMS.len());
        assert_eq!(
            s.per_kind[3],
            CLAIMS
                .iter()
                .filter(|c| c.evidence.contains(&Argument))
                .count()
        );
        assert!(s.only_argument <= s.per_kind[3]);
        assert!(
            s.only_argument > 0,
            "loạt phải thừa nhận có chỗ chỉ là lập luận"
        );
    }
}
