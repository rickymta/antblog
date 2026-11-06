//! Cách làm KHÔNG nên: lưu số dư thành một ô rồi đọc–tính–ghi đè.
//! Chỉ để so sánh với projection; không dùng trong sổ cái.

use crate::money::Money;

/// Một ô số dư như cột `balance` trong bảng: đọc ra, tính ở ngoài, ghi đè lại.
struct BalanceCell(Money);

impl BalanceCell {
    fn read(&self) -> Money {
        self.0
    }

    fn write(&mut self, value: Money) {
        self.0 = value;
    }
}

/// Hai bên cùng cộng `delta` vào một ô số dư, xen kẽ đúng thứ tự gây mất cập nhật:
/// A đọc, B đọc, A ghi, B ghi. Trả về `(giá trị ô, tổng đúng tính từ posting)`.
pub fn lost_update(delta: Money) -> (Money, Money) {
    let mut cell = BalanceCell(Money::ZERO);

    let seen_by_a = cell.read(); // A đọc số dư
    let seen_by_b = cell.read(); // B đọc số dư (trước khi A ghi)
    cell.write(seen_by_a.checked_add(delta).expect("không tràn")); // A ghi
    cell.write(seen_by_b.checked_add(delta).expect("không tràn")); // B ghi đè lên A

    // Projection: cộng mọi posting, không có bước đọc-rồi-ghi nào để xen vào.
    let projected = [delta, delta]
        .into_iter()
        .try_fold(Money::ZERO, Money::checked_add)
        .expect("không tràn");
    (cell.read(), projected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interleaved_writers_lose_one_update() {
        let (stored, projected) = lost_update(Money::from_dong(20_400));
        assert_eq!(stored, Money::from_dong(20_400));
        assert_eq!(projected, Money::from_dong(40_800));
    }
}
