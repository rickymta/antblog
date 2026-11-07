//! Sắp xếp lại đơn vị hành chính năm 2025: câu hỏi nêu tên cũ phải được ánh xạ hoặc gắn cờ, không bao giờ trả lời tự tin sai.
//!
//! # Sự kiện đã kiểm (xem `results/p04.md`, mục LOC-31, ngày kiểm 2026-11-07)
//!
//! - Nghị quyết **202/2025/QH15** của Quốc hội, thông qua ngày 12/6/2025: cả nước còn 34 đơn vị cấp tỉnh. Điều 1 khoản 16
//!   nhập Thành phố Hồ Chí Minh, tỉnh Bà Rịa - Vũng Tàu và tỉnh Bình Dương thành Thành phố Hồ Chí Minh mới.
//!   (Trang luatvietnam.vn đã tải ghi số hiệu, ngày và khoản 16; nhandan.vn ghi cùng nội dung nhưng không ghi số hiệu.)
//! - Nghị quyết **203/2025/QH15** (sửa đổi Hiến pháp), thông qua 16/6/2025: bỏ đơn vị hành chính cấp huyện trên cả nước từ
//!   01/7/2025 (trang pbgdpl.cantho.gov.vn đã tải).
//! - Nghị quyết **1685/NQ-UBTVQH15** ngày 16/6/2025 của Ủy ban Thường vụ Quốc hội: sắp xếp các đơn vị hành chính cấp xã của
//!   TP.HCM năm 2025, còn 168 đơn vị cấp xã (113 phường, 54 xã, 1 đặc khu Côn Đảo); chính quyền mới hoạt động từ 01/7/2025
//!   (trang luatvietnam.vn đã tải; mục «Ngày áp dụng» của trang nằm sau đăng nhập).
//!
//! # Bảng ánh xạ phường cũ sang phường mới: **một phần**
//!
//! [`WARD_MAPPINGS`] chỉ gồm các khoản 1 đến 20 và khoản 78 của Điều 1 Nghị quyết 1685 (các phường thuộc Quận 1, 3, 4, 5, 6,
//! một phần Quận 7 cũ và 4 phường của TP. Thủ Đức cũ), đọc từ trang luatvietnam.vn. Toàn bộ 168 khoản **chưa** kiểm; phường
//! không có trong bảng không có nghĩa là không đổi. Phường cũ bị tách vào nhiều phường mới được ghi là *tách*, không chọn
//! bừa một phường. Mọi trường hợp còn lại: [`AdminStatus::OldUnmapped`] kèm cảnh báo và trần độ tin cậy.

use crate::norm;
use crate::parse::ParsedAddress;

/// Ngày kiểm các sự kiện ở đầu module.
pub const VERIFIED_ON: &str = "2026-11-07";

/// Một dòng ánh xạ phường cũ sang phường mới. `district` là khoá quận cũ (`"3"`) nếu trang nguồn nêu quận, hoặc `None`.
#[derive(Debug, Clone, Copy)]
pub struct WardMapping {
    pub district: Option<&'static str>,
    /// Khoá phường cũ (`"9"`, `"ben nghe"`).
    pub ward: &'static str,
    /// Tên phường mới (có dấu). Nhiều hơn một tên nghĩa là phường cũ bị tách.
    pub new: &'static [&'static str],
}

const fn m(
    district: Option<&'static str>,
    ward: &'static str,
    new: &'static [&'static str],
) -> WardMapping {
    WardMapping {
        district,
        ward,
        new,
    }
}

/// Bảng một phần. Nguồn: Nghị quyết 1685/NQ-UBTVQH15, Điều 1 khoản 1–20 và 78, qua trang luatvietnam.vn (tải 2026-11-07).
pub const WARD_MAPPINGS: &[WardMapping] = &[
    // khoản 1–4: Quận 1 cũ và lân cận
    m(None, "ben nghe", &["Sài Gòn"]),
    m(None, "da kao", &["Sài Gòn", "Tân Định"]),
    m(None, "nguyen thai binh", &["Sài Gòn", "Bến Thành"]),
    m(Some("1"), "tan dinh", &["Tân Định"]),
    m(None, "ben thanh", &["Bến Thành"]),
    m(None, "pham ngu lao", &["Bến Thành"]),
    m(None, "cau ong lanh", &["Bến Thành", "Cầu Ông Lãnh"]),
    m(None, "nguyen cu trinh", &["Cầu Ông Lãnh"]),
    m(None, "cau kho", &["Cầu Ông Lãnh"]),
    m(None, "co giang", &["Cầu Ông Lãnh"]),
    // khoản 5–7: Quận 3
    m(Some("3"), "1", &["Bàn Cờ"]),
    m(Some("3"), "2", &["Bàn Cờ"]),
    m(Some("3"), "3", &["Bàn Cờ"]),
    m(Some("3"), "5", &["Bàn Cờ"]),
    m(Some("3"), "4", &["Bàn Cờ", "Xuân Hòa"]),
    m(None, "vo thi sau", &["Xuân Hòa"]),
    m(Some("3"), "9", &["Nhiêu Lộc"]),
    m(Some("3"), "11", &["Nhiêu Lộc"]),
    m(Some("3"), "12", &["Nhiêu Lộc"]),
    m(Some("3"), "14", &["Nhiêu Lộc"]),
    // khoản 8–10: Quận 4
    m(Some("4"), "13", &["Xóm Chiếu"]),
    m(Some("4"), "16", &["Xóm Chiếu"]),
    m(Some("4"), "18", &["Xóm Chiếu"]),
    m(Some("4"), "15", &["Xóm Chiếu", "Khánh Hội"]),
    m(Some("4"), "8", &["Khánh Hội"]),
    m(Some("4"), "9", &["Khánh Hội"]),
    m(Some("4"), "2", &["Khánh Hội", "Vĩnh Hội"]),
    m(Some("4"), "4", &["Khánh Hội", "Vĩnh Hội"]),
    m(Some("4"), "1", &["Vĩnh Hội"]),
    m(Some("4"), "3", &["Vĩnh Hội"]),
    // khoản 11–13: Quận 5
    m(Some("5"), "1", &["Chợ Quán"]),
    m(Some("5"), "2", &["Chợ Quán"]),
    m(Some("5"), "4", &["Chợ Quán"]),
    m(Some("5"), "5", &["An Đông"]),
    m(Some("5"), "7", &["An Đông"]),
    m(Some("5"), "9", &["An Đông"]),
    m(Some("5"), "11", &["Chợ Lớn"]),
    m(Some("5"), "12", &["Chợ Lớn"]),
    m(Some("5"), "13", &["Chợ Lớn"]),
    m(Some("5"), "14", &["Chợ Lớn"]),
    // khoản 14–17: Quận 6
    m(Some("6"), "2", &["Bình Tây"]),
    m(Some("6"), "9", &["Bình Tây"]),
    m(Some("6"), "1", &["Bình Tiên"]),
    m(Some("6"), "7", &["Bình Tiên"]),
    m(Some("6"), "8", &["Bình Tiên"]),
    m(Some("6"), "10", &["Bình Phú"]),
    m(Some("6"), "11", &["Bình Phú"]),
    m(Some("6"), "12", &["Phú Lâm"]),
    m(Some("6"), "13", &["Phú Lâm"]),
    m(Some("6"), "14", &["Phú Lâm"]),
    // khoản 18–20: Quận 7
    m(None, "binh thuan", &["Tân Thuận"]),
    m(None, "tan thuan dong", &["Tân Thuận"]),
    m(None, "tan thuan tay", &["Tân Thuận"]),
    m(None, "phu thuan", &["Phú Thuận"]),
    m(Some("7"), "phu my", &["Phú Thuận", "Tân Mỹ"]),
    m(Some("7"), "tan phu", &["Tân Mỹ"]),
    // khoản 78: TP. Thủ Đức cũ
    m(None, "thu thiem", &["An Khánh"]),
    m(None, "an loi dong", &["An Khánh"]),
    m(None, "thao dien", &["An Khánh"]),
    m(None, "an khanh", &["An Khánh"]),
];

/// Tên phường/xã mới đã thấy trong các khoản 1–20, 78, cộng 5 đơn vị giữ nguyên (Thới Hòa, Long Sơn, Hòa Hiệp, Bình Châu, Thạnh An).
pub const NEW_WARD_NAMES: &[&str] = &[
    "sai gon",
    "tan dinh",
    "ben thanh",
    "cau ong lanh",
    "ban co",
    "xuan hoa",
    "nhieu loc",
    "xom chieu",
    "khanh hoi",
    "vinh hoi",
    "cho quan",
    "an dong",
    "cho lon",
    "binh tay",
    "binh tien",
    "binh phu",
    "phu lam",
    "tan thuan",
    "phu thuan",
    "tan my",
    "an khanh",
    "thoi hoa",
    "long son",
    "hoa hiep",
    "binh chau",
    "thanh an",
];

/// Khoá tỉnh đã nhập vào TP.HCM mới theo Nghị quyết 202/2025/QH15 (Điều 1 khoản 16).
pub const MERGED_INTO_HCM: &[&str] = &["binh duong", "ba ria vung tau", "ba ria", "vung tau"];

/// Kết quả kiểm tên hành chính của một câu hỏi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AdminStatus {
    /// Câu hỏi không nêu phường, quận hay tỉnh.
    NoAdmin,
    /// Có tên phường/xã nhưng không dấu hiệu cũ và không có trong bảng mới đã biết: coi là hiện hành, chưa kiểm được.
    NotChecked,
    /// Tên phường nằm trong danh sách phường mới đã kiểm (kể cả tên giữ nguyên nhưng phạm vi đổi).
    NewWard,
    /// Phường cũ ánh xạ được vào đúng một phường mới.
    OldMapped,
    /// Phường cũ bị tách vào nhiều phường mới: không chọn bừa.
    OldSplit,
    /// Tên quận/huyện hoặc phường số cũ, không có dòng ánh xạ đã kiểm.
    OldUnmapped,
    /// Tỉnh Bình Dương hoặc Bà Rịa - Vũng Tàu cũ, nay thuộc TP.HCM mới.
    ProvinceMerged,
}

/// Ghi chú hành chính kèm kết quả geocode.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AdminNote {
    pub status: AdminStatus,
    /// Tên phường mới (nếu ánh xạ được hoặc tách).
    pub new_wards: Vec<String>,
    pub warnings: Vec<String>,
}

impl AdminStatus {
    /// Trần độ tin cậy cho kết quả geocode khi trạng thái này xảy ra: không bao giờ tin cậy cao khi tên hành chính đã cũ.
    pub fn confidence_cap(self) -> f64 {
        match self {
            Self::OldUnmapped | Self::OldSplit | Self::ProvinceMerged => 0.5,
            Self::OldMapped => 0.8,
            Self::NoAdmin | Self::NotChecked | Self::NewWard => 1.0,
        }
    }
}

fn lookup(district: Option<&str>, ward: &str) -> Option<&'static WardMapping> {
    WARD_MAPPINGS
        .iter()
        .find(|r| r.ward == ward && r.district.is_none_or(|d| Some(d) == district))
}

/// Kiểm tên hành chính trong câu hỏi đã phân tích.
pub fn check(p: &ParsedAddress) -> AdminNote {
    let mut note = AdminNote {
        status: AdminStatus::NoAdmin,
        new_wards: Vec::new(),
        warnings: Vec::new(),
    };
    let province = p
        .province
        .as_deref()
        .or(p.city.as_deref())
        .map(|s| s.to_string());
    if let Some(pr) = province.as_deref()
        && MERGED_INTO_HCM.contains(&pr)
    {
        note.status = AdminStatus::ProvinceMerged;
        note.new_wards = vec!["Thành phố Hồ Chí Minh".into()];
        note.warnings.push(format!(
            "«{pr}» đã nhập vào Thành phố Hồ Chí Minh từ 01/7/2025 (NQ 202/2025/QH15); chỉ mục của lab chỉ phủ khu vực TP.HCM cũ"
        ));
        return note;
    }
    let district = p.district.as_deref();
    let Some(ward) = p.ward.as_deref() else {
        if district.is_some() {
            note.status = AdminStatus::OldUnmapped;
            note.warnings.push(
                "cấp huyện (quận/huyện/TP. Thủ Đức) đã bỏ từ 01/7/2025 (NQ 203/2025/QH15); không có phường để ánh xạ, kết quả chỉ ở mức đường"
                    .into(),
            );
        } else if p.city.is_some() {
            note.status = AdminStatus::NoAdmin;
        }
        return note;
    };

    let numeric = ward.as_bytes().first().is_some_and(u8::is_ascii_digit);
    let kind_vn = match p.ward_kind.as_deref() {
        Some("xa") => "xã",
        Some("thi tran") => "thị trấn",
        Some("dac khu") => "đặc khu",
        _ => "phường",
    };
    let ward_label = format!("{kind_vn} «{ward}»");
    if district.is_some() || numeric {
        note.warnings.push(
            "câu hỏi dùng tên quận/phường số kiểu cũ; cấp huyện đã bỏ và các phường đã sáp nhập từ 01/7/2025 (NQ 203/2025/QH15, NQ 1685/NQ-UBTVQH15)"
                .into(),
        );
    }
    match lookup(district, ward) {
        Some(r) if r.new.len() == 1 => {
            let same = norm::normalize(r.new[0]) == ward;
            note.new_wards = r.new.iter().map(|s| s.to_string()).collect();
            if same && district.is_none() {
                note.status = AdminStatus::NewWard;
            } else {
                note.status = AdminStatus::OldMapped;
                note.warnings.push(format!(
                    "{ward_label} cũ nay thuộc phường {} (bảng ánh xạ một phần, NQ 1685)",
                    r.new[0]
                ));
            }
        }
        Some(r) => {
            note.status = AdminStatus::OldSplit;
            note.new_wards = r.new.iter().map(|s| s.to_string()).collect();
            note.warnings.push(format!(
                "{ward_label} cũ bị tách vào các phường {}; không chọn bừa một phường",
                r.new.join(", ")
            ));
        }
        None if district.is_some() || numeric => {
            note.status = AdminStatus::OldUnmapped;
            note.warnings.push(format!(
                "{ward_label} không có trong bảng ánh xạ đã kiểm (cần kiểm lại với NQ 1685/NQ-UBTVQH15)"
            ));
        }
        None if NEW_WARD_NAMES.contains(&ward) => note.status = AdminStatus::NewWard,
        None => note.status = AdminStatus::NotChecked,
    }
    note
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn status(q: &str) -> AdminStatus {
        check(&parse(q)).status
    }

    #[test]
    fn mapped_split_unmapped() {
        assert_eq!(
            status("12 Lê Lợi, Phường Bến Nghé, Quận 1"),
            AdminStatus::OldMapped
        );
        assert_eq!(status("Phường 9, Quận 3"), AdminStatus::OldMapped);
        assert_eq!(status("Phường Đa Kao, Quận 1"), AdminStatus::OldSplit);
        assert_eq!(status("Phường 12, Quận Tân Bình"), AdminStatus::OldUnmapped);
        // phường số không kèm quận: không đoán
        assert_eq!(status("Phường 9"), AdminStatus::OldUnmapped);
        assert_eq!(status("Quận Bình Thạnh"), AdminStatus::OldUnmapped);
    }

    #[test]
    fn current_and_province() {
        assert_eq!(status("Phường Sài Gòn"), AdminStatus::NewWard);
        assert_eq!(status("Phường Bến Thành"), AdminStatus::NewWard);
        assert_eq!(status("Phường Xyz Abc"), AdminStatus::NotChecked);
        assert_eq!(status("12 Lê Lợi"), AdminStatus::NoAdmin);
        assert_eq!(status("Tỉnh Bình Dương"), AdminStatus::ProvinceMerged);
    }

    #[test]
    fn caps() {
        assert!(AdminStatus::OldUnmapped.confidence_cap() <= 0.5);
        assert!((AdminStatus::NewWard.confidence_cap() - 1.0).abs() < 1e-12);
    }
}
