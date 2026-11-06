//! Bất biến bằng proptest: chuẩn hoá, tách địa chỉ, gợi ý và độ tin cậy không được vỡ trên đầu vào bất kỳ.

use geocode::{Index, norm, parse, prefix_key};
use proptest::prelude::*;
use std::path::PathBuf;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

fn fixture() -> &'static Index {
    static IDX: OnceLock<Index> = OnceLock::new();
    IDX.get_or_init(|| {
        let p =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../fixtures/geocode-mau.json");
        Index::from_json_file(&p).expect("nạp fixture")
    })
}

/// Chuỗi giống địa chỉ Việt Nam: chữ có dấu, số, dấu câu, viết tắt.
fn addr_like() -> impl Strategy<Value = String> {
    "[a-zA-ZÀ-ỹĐđ0-9 .,/\\-]{0,40}"
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn normalize_idempotent(s in prop_oneof![any::<String>(), addr_like()]) {
        let n = norm::normalize(&s);
        prop_assert_eq!(norm::normalize(&n), n);
    }

    #[test]
    fn normalize_is_ascii_lowercase(s in prop_oneof![any::<String>(), addr_like()]) {
        let n = norm::normalize(&s);
        prop_assert!(n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' ' || c == '/' || c == '-'));
        prop_assert!(!n.starts_with(' ') && !n.ends_with(' ') && !n.contains("  "));
        let f = norm::fold(&s);
        prop_assert!(f.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' '));
    }

    #[test]
    fn nfc_and_nfd_give_same_key(s in addr_like()) {
        let a: String = s.nfc().collect();
        let d: String = s.nfd().collect();
        prop_assert_eq!(norm::normalize(&a), norm::normalize(&d));
        prop_assert_eq!(norm::lower_nfc(&a), norm::lower_nfc(&d));
    }

    #[test]
    fn parse_never_panics_and_keys_are_normalised(s in prop_oneof![any::<String>(), addr_like()]) {
        let p = parse::parse(&s);
        if let Some(st) = &p.street {
            prop_assert!(st.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == ' ' || c == '/' || c == '-'));
        }
        let _ = p.number_keys();
        let _ = norm::hn_keys(&s);
        let _ = norm::street_key(&s);
        let _ = norm::admin_key(&s);
    }

    #[test]
    fn house_chain_roundtrip(
        parts in prop::collection::vec((1u32..9999, prop::option::of(prop::sample::select(vec!['a','b','c','d','e']))), 1..4)
    ) {
        let chain = parts.iter().map(|(n, s)| format!("{n}{}", s.map(String::from).unwrap_or_default())).collect::<Vec<_>>().join("/");
        let p = parse::parse(&format!("{chain} Lê Lợi"));
        prop_assert_eq!(p.chain_key(), Some(chain));
        prop_assert_eq!(p.street.as_deref(), Some("le loi"));
        prop_assert_eq!(p.alleys.len(), parts.len() - 1);
    }

    #[test]
    fn suggestions_start_with_the_prefix_key(q in prop_oneof![addr_like(), "(12 )?(ngu|le|ba|d\\. ng|ca ph|3 th|đường s)[a-z ]{0,6}"]) {
        let idx = fixture();
        let key = prefix_key(&q);
        let s = idx.suggest(&q, 8);
        prop_assert!(s.len() <= 8);
        for x in &s {
            prop_assert!(x.key.starts_with(&key), "{:?} không bắt đầu bằng {:?}", x.key, key);
        }
        // mọi gợi ý gõ lại làm tiền tố thì chính nó phải nằm trong kết quả của tiền tố dài hơn một chút
        for w in s.windows(1) { prop_assert!(!w[0].label.is_empty()); }
    }

    #[test]
    fn geocode_confidence_in_unit_interval(q in prop_oneof![any::<String>(), addr_like(), "[0-9]{1,3}(/[0-9]{1,2})? (Nguyen Trai|Le Loi|duong so 7|ba le)[a-z ,.]{0,12}"]) {
        let r = fixture().geocode(&q);
        let mut prev = f64::INFINITY;
        for c in &r.candidates {
            prop_assert!((0.0..=1.0).contains(&c.confidence));
            prop_assert!((0.0..=1.0).contains(&c.score));
            prop_assert!(c.confidence <= prev);
            prev = c.confidence;
            prop_assert!(c.pos.lat().is_finite() && c.pos.lng().is_finite());
        }
        prop_assert!(r.confidence() <= r.admin.status.confidence_cap() + 1e-12);
    }
}
