//! Cây tài khoản cha–con bằng `Rc` (nhiều chủ, một luồng) và `Weak` (con trỏ cha).

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::account::AccountId;
use crate::error::LedgerError;
use crate::ledger::Ledger;
use crate::money::Money;

/// Một nút tài khoản. Cha giữ con bằng `Rc` (sở hữu), con giữ cha bằng `Weak` (không sở hữu),
/// nên không có chu trình `Rc` và cây được giải phóng khi chủ cuối cùng buông.
pub struct AccountNode {
    id: AccountId,
    parent: Weak<AccountNode>,
    // `RefCell`: cần thêm con vào nút đã nằm trong `Rc` (chỉ có `&`); xem mục RefCell trong bài.
    children: RefCell<Vec<Rc<AccountNode>>>,
}

impl AccountNode {
    pub fn new_root(id: AccountId) -> Rc<AccountNode> {
        Rc::new(AccountNode {
            id,
            parent: Weak::new(),
            children: RefCell::new(Vec::new()),
        })
    }

    /// Tạo nút con, đăng ký vào cha và trả thêm một `Rc` cho người gọi.
    pub fn add_child(parent: &Rc<AccountNode>, id: AccountId) -> Rc<AccountNode> {
        let child = Rc::new(AccountNode {
            id,
            parent: Rc::downgrade(parent),
            children: RefCell::new(Vec::new()),
        });
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }

    pub fn id(&self) -> &AccountId {
        &self.id
    }

    /// `upgrade` trả `None` nếu cha đã bị giải phóng.
    pub fn parent(&self) -> Option<Rc<AccountNode>> {
        self.parent.upgrade()
    }

    /// Đường đi từ gốc đến nút này, ví dụ `platform > platform:revenue`.
    pub fn path(&self) -> String {
        let mut parts = vec![self.id.to_string()];
        let mut current = self.parent();
        while let Some(node) = current {
            parts.push(node.id.to_string());
            current = node.parent();
        }
        parts.reverse();
        parts.join(" > ")
    }

    /// Số dư của nút cộng số dư mọi nút con cháu.
    pub fn subtree_balance(&self, ledger: &Ledger) -> Result<Money, LedgerError> {
        let mut sum = ledger.balance(&self.id)?;
        for child in self.children.borrow().iter() {
            sum = sum
                .checked_add(child.subtree_balance(ledger)?)
                .ok_or(LedgerError::Overflow)?;
        }
        Ok(sum)
    }
}

/// Cây mẫu: `platform` → `platform:revenue`, `platform:fee`.
/// Hàm trả mỗi `Rc` của nút con cho người gọi một lần rồi buông: chủ còn lại là nút cha.
pub fn platform_tree() -> Result<Rc<AccountNode>, LedgerError> {
    let root = AccountNode::new_root(AccountId::new("platform")?);
    AccountNode::add_child(&root, AccountId::new("platform:revenue")?);
    AccountNode::add_child(&root, AccountId::new("platform:fee")?);
    Ok(root)
}

/// Phản ví dụ: hai nút giữ nhau bằng `Rc` mạnh. Buông cả hai handle mà nút vẫn sống.
struct Leaky {
    other: RefCell<Option<Rc<Leaky>>>,
}

/// `true` nghĩa là bộ nhớ bị rò: sau khi buông `a` và `b`, `Weak` vẫn nâng cấp được.
pub fn strong_cycle_leaks() -> bool {
    let a = Rc::new(Leaky {
        other: RefCell::new(None),
    });
    let b = Rc::new(Leaky {
        other: RefCell::new(Some(Rc::clone(&a))),
    });
    *a.other.borrow_mut() = Some(Rc::clone(&b));
    let probe = Rc::downgrade(&a);
    drop(a);
    drop(b);
    probe.upgrade().is_some()
}

/// `true` nghĩa là cây có `Weak` cha được giải phóng hết sau khi buông các handle.
pub fn weak_parent_tree_is_freed() -> Result<bool, LedgerError> {
    let root = AccountNode::new_root(AccountId::new("platform")?);
    let child = AccountNode::add_child(&root, AccountId::new("platform:revenue")?);
    let probe_root = Rc::downgrade(&root);
    let probe_child = Rc::downgrade(&child);
    drop(child);
    drop(root);
    Ok(probe_root.upgrade().is_none() && probe_child.upgrade().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{JournalEntry, Posting};

    fn acct(raw: &str) -> AccountId {
        AccountId::new(raw).expect("mã hợp lệ")
    }

    #[test]
    fn counts_after_adding_two_children() -> Result<(), LedgerError> {
        let root = AccountNode::new_root(acct("platform"));
        let revenue = AccountNode::add_child(&root, acct("platform:revenue"));
        let _fee = AccountNode::add_child(&root, acct("platform:fee"));
        assert_eq!(Rc::strong_count(&root), 1);
        assert_eq!(Rc::weak_count(&root), 2);
        assert_eq!(Rc::strong_count(&revenue), 2);
        assert_eq!(revenue.path(), "platform > platform:revenue");
        assert!(revenue.parent().is_some());
        Ok(())
    }

    #[test]
    fn strong_cycle_is_a_leak_and_weak_parent_is_not() -> Result<(), LedgerError> {
        assert!(strong_cycle_leaks());
        assert!(weak_parent_tree_is_freed()?);
        Ok(())
    }

    #[test]
    fn moving_money_inside_the_tree_keeps_the_root_total() -> Result<(), LedgerError> {
        let root = platform_tree()?;
        let mut ledger = Ledger::new();
        let entry = |key: &str, from: &str, to: &str, dong: i64| {
            JournalEntry::try_new(
                key,
                vec![
                    Posting::new(acct(from), Money::from_dong(-dong)),
                    Posting::new(acct(to), Money::from_dong(dong)),
                ],
            )
        };
        ledger.post(entry(
            "trip:TN-0001:fee",
            "customer:CU-001",
            "platform:revenue",
            5_100,
        )?)?;
        assert_eq!(root.subtree_balance(&ledger)?, Money::from_dong(5_100));
        ledger.post(entry(
            "fee:TN-0001:reclass",
            "platform:revenue",
            "platform:fee",
            1_000,
        )?)?;
        assert_eq!(root.subtree_balance(&ledger)?, Money::from_dong(5_100));
        Ok(())
    }
}
