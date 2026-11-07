//! Phân tích chuỗi địa chỉ Việt Nam thành [`ParsedAddress`].
//!
//! Quy tắc, theo thứ tự áp dụng (sau khi [`crate::norm::tokens`] đã bỏ dấu, mở rộng viết tắt):
//!
//! 1. **Đơn vị hành chính** (tách trước). Từ khoá `phuong`, `quan`, `huyen`, `xa`, `thi xa`, `thi tran`, `thanh pho`,
//!    `tinh`, `dac khu` mở một đoạn tên. Đoạn tên dài tối đa 4 token, kết thúc ở dấu phẩy, ở từ khoá kế tiếp, hoặc ngay
//!    sau một số (`phuong 12`, `quan 1`). `ho chi minh` đứng trần ở cuối chuỗi, hoặc ngay sau `thanh pho`, là thành phố.
//!    Từ khoá bị bỏ qua nếu đứng ngay sau `duong`/`pho` (tên đường), hoặc nếu người dùng có gõ dấu mà dấu không phải của
//!    từ khoá (`Phương`, `Huyền`, `Quân` là tên người, không phải `Phường`, `Huyện`, `Quận`).
//! 2. **Số nhà và hẻm** ở đầu phần còn lại. Chuỗi `123/45/6` đọc là: số 123 ngoài đường chính, rẽ vào hẻm 45, nhà số 6.
//!    Phần cuối là số nhà ([`ParsedAddress::house`]), các phần trước là chuỗi hẻm ([`ParsedAddress::alleys`]).
//!    `hẻm 45`, `ngõ 45`, `ngách 3`, `kiệt 12` thêm một phần vào chuỗi hẻm. Mỗi phần có dạng `số + hậu tố chữ`
//!    (`12A`, `45b`, `12bis`); `12-14` giữ nguyên là một phần. Một chữ cái đơn `a`–`e` đứng sau số và trước một từ
//!    khác được ghép thành hậu tố (`12 A Lê Lợi`). Số chỉ được lấy **một lần** ở đầu: `3 Tháng 2` (không có số nhà
//!    trước nó) và `12 3 Tháng 2` đều đọc đúng, vì một số theo sau là `thang <số>` thì thuộc tên đường.
//! 3. **Tên đường** là phần còn lại, qua [`crate::norm::street_key`] (bỏ `đường`, `phố`, `số` trước số).
//!    Nếu không có số nhà thì phần còn lại cũng được giữ ở [`ParsedAddress::name_words`] để tra tên địa điểm.

use crate::norm::{self, Token};
use serde::{Deserialize, Serialize};

/// Một phần của số nhà/hẻm: `45` hay `12a`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct NumPart {
    pub num: String,
    pub suffix: String,
}

impl NumPart {
    /// `12` + `a` thành `12a`.
    pub fn key(&self) -> String {
        format!("{}{}", self.num, self.suffix)
    }

    /// Giá trị số của phần chữ số đầu tiên (`12-14` cho 12), dùng để tìm số nhà gần nhất.
    pub fn leading_number(&self) -> Option<u32> {
        let d: String = self.num.chars().take_while(char::is_ascii_digit).collect();
        d.parse().ok()
    }
}

/// Địa chỉ đã tách thành phần. Mọi chuỗi ở đây là khoá đã chuẩn hoá (không dấu, chữ thường) trừ [`Self::raw`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParsedAddress {
    pub raw: String,
    /// Chuỗi hẻm từ đường chính vào: `123/45/6` cho `[123, 45]`; `hẻm 45` cho `[45]`.
    pub alleys: Vec<NumPart>,
    /// Số nhà cuối cùng (`6` trong `123/45/6`).
    pub house: Option<NumPart>,
    /// Chuỗi số nhà đúng như viết, dạng OSM (`123/45/6`), nếu có.
    pub chain: Vec<NumPart>,
    /// Khoá tên đường ([`norm::street_key`]).
    pub street: Option<String>,
    /// Tên đường như người dùng gõ (NFC, chữ thường, còn dấu), để phân biệt `Bà` và `Ba`.
    pub street_typed: Option<String>,
    /// Phần còn lại sau khi bỏ hành chính và số, không bỏ `đường`: dùng tra tên địa điểm.
    pub name_words: Option<String>,
    pub ward: Option<String>,
    /// `phuong`, `xa`, `thi tran`, `dac khu`.
    pub ward_kind: Option<String>,
    pub district: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    /// Người dùng có gõ dấu.
    pub typed_with_marks: bool,
}

/// Cách khớp một khoá số nhà: nguyên văn hay chỉ lối vào hẻm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberMatch {
    Exact,
    AlleyRoot,
}

impl ParsedAddress {
    /// Số nhà dạng OSM: các phần của `chain` nối bằng `/`.
    pub fn chain_key(&self) -> Option<String> {
        if self.chain.is_empty() {
            return None;
        }
        Some(
            self.chain
                .iter()
                .map(NumPart::key)
                .collect::<Vec<_>>()
                .join("/"),
        )
    }

    /// Khoá số nhà để thử, theo thứ tự ưu tiên: nguyên văn trước, rồi lối vào hẻm (số đầu tiên, tức nhà trên đường chính).
    pub fn number_keys(&self) -> Vec<(String, NumberMatch)> {
        let mut out: Vec<(String, NumberMatch)> = Vec::new();
        let marker_alleys = self.alleys.len() - self.chain.len().saturating_sub(1);
        if let Some(full) = self.chain_key() {
            out.push((full, NumberMatch::Exact));
            if marker_alleys > 0 {
                let mut all: Vec<String> = self.alleys[..marker_alleys]
                    .iter()
                    .map(NumPart::key)
                    .collect();
                all.extend(self.chain.iter().map(NumPart::key));
                out.push((all.join("/"), NumberMatch::Exact));
            }
        }
        let root = if self.chain.len() > 1 {
            Some(self.chain[0].key())
        } else if self.chain.is_empty() {
            self.alleys.first().map(NumPart::key)
        } else {
            None
        };
        if let Some(r) = root
            && !out.iter().any(|(k, _)| *k == r)
        {
            out.push((r, NumberMatch::AlleyRoot));
        }
        out
    }

    /// Có thành phần hành chính nào trong câu hỏi không.
    pub fn has_admin(&self) -> bool {
        self.ward.is_some()
            || self.district.is_some()
            || self.city.is_some()
            || self.province.is_some()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kw {
    Ward(&'static str),
    District,
    City,
    Province,
}

/// Từ khoá hành chính ở vị trí `i`: (loại, số token của từ khoá).
fn keyword_at(toks: &[Token], i: usize) -> Option<(Kw, usize)> {
    let t = toks[i].text.as_str();
    let n1 = toks.get(i + 1).map(|x| x.text.as_str());
    // tên đường đứng ngay sau «đường»/«phố» thì không phải từ khoá
    if i > 0 && matches!(toks[i - 1].text.as_str(), "duong" | "pho") {
        return None;
    }
    // người dùng có gõ dấu mà không phải dấu của từ khoá: tên riêng (Phương, Huyền, Quân…)
    let marked_ok = |canon: &str| toks[i].marked == toks[i].text || toks[i].marked == canon;
    match (t, n1) {
        ("phuong", _) if marked_ok("phường") => Some((Kw::Ward("phuong"), 1)),
        ("quan", _) if marked_ok("quận") => Some((Kw::District, 1)),
        ("huyen", _) if marked_ok("huyện") => Some((Kw::District, 1)),
        ("xa", Some("lo")) => None,
        ("xa", _) if marked_ok("xã") => Some((Kw::Ward("xa"), 1)),
        ("thi", Some("xa")) => Some((Kw::District, 2)),
        ("thi", Some("tran")) => Some((Kw::Ward("thi tran"), 2)),
        ("dac", Some("khu")) => Some((Kw::Ward("dac khu"), 2)),
        ("thanh", Some("pho")) => Some((Kw::City, 2)),
        ("tinh", _) if marked_ok("tỉnh") => Some((Kw::Province, 1)),
        _ => None,
    }
}

fn is_number_token(t: &str) -> bool {
    t.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

/// Tách một phần `45`, `12a`, `12-14`, `12-a`. `None` nếu không đúng dạng.
fn parse_part(s: &str) -> Option<NumPart> {
    let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &s[digits.len()..];
    if let Some(r) = rest.strip_prefix('-') {
        if !r.is_empty() && r.chars().all(|c| c.is_ascii_digit()) {
            return Some(NumPart {
                num: format!("{digits}-{r}"),
                suffix: String::new(),
            });
        }
        return parse_suffix(&digits, r);
    }
    parse_suffix(&digits, rest)
}

fn parse_suffix(digits: &str, suffix: &str) -> Option<NumPart> {
    if suffix.len() <= 3 && suffix.chars().all(|c| c.is_ascii_lowercase()) {
        Some(NumPart {
            num: digits.to_string(),
            suffix: suffix.to_string(),
        })
    } else {
        None
    }
}

pub(crate) fn parse_chain(tok: &str) -> Option<Vec<NumPart>> {
    // `12-14/3`: tách theo '/', mỗi phần có thể chứa '-' trong dạng khoảng
    let parts: Option<Vec<NumPart>> = tok.split('/').map(parse_part).collect();
    parts.filter(|p| !p.is_empty())
}

const ALLEY_WORDS: [&str; 4] = ["hem", "ngo", "ngach", "kiet"];

/// Phân tích một chuỗi địa chỉ. Không bao giờ panic, mọi đầu vào đều cho một kết quả (có thể rỗng).
pub fn parse(input: &str) -> ParsedAddress {
    let toks = norm::tokens(input);
    let mut out = ParsedAddress {
        raw: input.to_string(),
        typed_with_marks: norm::has_diacritics(input),
        ..ParsedAddress::default()
    };

    // 1. đơn vị hành chính
    let n = toks.len();
    let mut head: Vec<Token> = Vec::new();
    let mut i = 0;
    while i < n {
        if let Some((kw, klen)) = keyword_at(&toks, i) {
            let start = i + klen;
            let mut j = start;
            let mut name: Vec<&str> = Vec::new();
            // nếu từ khoá đã kết thúc bằng dấu phẩy thì không có tên
            let mut closed = toks[start - 1].comma_after;
            while !closed && j < n && name.len() < 4 {
                if keyword_at(&toks, j).is_some() {
                    break;
                }
                if name.is_empty() && is_number_token(&toks[j].text) {
                    name.push(&toks[j].text);
                    j += 1;
                    break;
                }
                name.push(&toks[j].text);
                closed = toks[j].comma_after;
                j += 1;
            }
            if !name.is_empty() {
                let val = name.join(" ");
                match kw {
                    Kw::Ward(kind) => {
                        out.ward = Some(val);
                        out.ward_kind = Some(kind.to_string());
                    }
                    Kw::District => out.district = Some(val),
                    Kw::Province => out.province = Some(val),
                    Kw::City => {
                        if val == "thu duc" {
                            out.district = Some("thanh pho thu duc".into());
                        } else {
                            out.city = Some(val);
                        }
                    }
                }
                i = j;
                continue;
            }
        }
        // thành phố đứng trần ở cuối chuỗi (kèm «city», «Việt Nam» nếu có); «Việt Nam» đứng riêng thì bỏ
        let tail = &toks[i..];
        let after_street_word =
            i > 0 && matches!(toks[i - 1].text.as_str(), "duong" | "pho" | "khu");
        let country = |t: &[Token]| {
            let v: Vec<&str> = t.iter().map(|x| x.text.as_str()).collect();
            matches!(v.as_slice(), ["vietnam"] | ["viet", "nam"])
        };
        let city_len = ["ho chi minh", "sai gon"].iter().find_map(|c| {
            let w: Vec<&str> = c.split(' ').collect();
            let hit = tail.len() >= w.len() && tail.iter().zip(&w).all(|(t, x)| t.text == *x);
            let rest = tail.get(w.len()..).unwrap_or(&[]);
            let tidy = rest
                .iter()
                .all(|t| matches!(t.text.as_str(), "city" | "viet" | "nam" | "vietnam"));
            (hit && tidy).then_some((*c, w.len()))
        });
        if i > 0 && !after_street_word {
            if let Some((city, _)) = city_len {
                out.city = Some(city.to_string());
                break;
            }
            if country(tail) {
                break;
            }
        }
        head.push(toks[i].clone());
        i += 1;
    }

    // đoạn sau dấu phẩy đầu tiên không phải tên đường: thành phố/quốc gia thì ghi nhận, còn lại là phường viết không từ khoá.
    // Dấu phẩy ngay sau số nhà (`Số 12, Nguyễn Trãi`) không phải ranh giới: đoạn đầu phải có ít nhất một từ.
    let mut has_word = false;
    let mut cut = None;
    for (k, t) in head.iter().enumerate() {
        has_word |= !is_number_token(&t.text) && !ALLEY_WORDS.contains(&t.text.as_str());
        if t.comma_after && has_word {
            cut = Some(k);
            break;
        }
    }
    if let Some(cut) = cut.filter(|c| c + 1 < head.len()) {
        let tail: Vec<Token> = head.split_off(cut + 1);
        let seg_end = tail
            .iter()
            .position(|t| t.comma_after)
            .map_or(tail.len(), |p| p + 1);
        let words = |ts: &[Token]| {
            ts.iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        let (joined, first_seg) = (words(&tail), words(&tail[..seg_end]));
        let cityish = |s: &str| s.contains("ho chi minh") || s.starts_with("sai gon");
        if out.city.is_none() && cityish(&joined) {
            out.city = Some(
                if joined.starts_with("sai gon") {
                    "sai gon"
                } else {
                    "ho chi minh"
                }
                .into(),
            );
        } else if out.ward.is_none()
            && !cityish(&first_seg)
            && !first_seg.contains("viet nam")
            && first_seg != "vietnam"
            && first_seg.split(' ').count() <= 4
        {
            out.ward = Some(first_seg);
        }
    }

    // 2. số nhà và hẻm ở đầu phần còn lại
    let mut marker_alleys: Vec<NumPart> = Vec::new();
    let mut k = 0;
    let mut chain: Vec<NumPart> = Vec::new();
    while let Some(t) = head.get(k) {
        let text = t.text.as_str();
        if ALLEY_WORDS.contains(&text) {
            // «Hẻm 65» đứng cuối, không còn từ nào sau nó: đó là tên đường (đường gọi là «Hẻm 65»), không phải hẻm
            if k + 2 < head.len()
                && let Some(parts) = head.get(k + 1).and_then(|x| parse_chain(&x.text))
            {
                marker_alleys.extend(parts);
                k += 2;
                continue;
            }
            break;
        }
        if text == "so" && head.get(k + 1).is_some_and(|x| is_number_token(&x.text)) {
            k += 1;
            continue;
        }
        if chain.is_empty() && is_number_token(text) {
            // «3 thang 2»: số đứng trước `thang <số>` thuộc tên đường
            let date_street = head.get(k + 1).is_some_and(|x| x.text == "thang")
                && head.get(k + 2).is_some_and(|x| is_number_token(&x.text));
            if date_street {
                break;
            }
            if let Some(mut parts) = parse_chain(text) {
                k += 1;
                // hậu tố chữ đứng riêng: «12 a lê lợi»
                if let (Some(next), Some(after)) = (head.get(k), head.get(k + 1)) {
                    let single =
                        next.text.len() == 1 && matches!(next.text.as_bytes()[0], b'a'..=b'e');
                    let word_after = !is_number_token(&after.text) && !next.dotted;
                    if single
                        && word_after
                        && let Some(last) = parts.last_mut()
                        && last.suffix.is_empty()
                    {
                        last.suffix = next.text.clone();
                        k += 1;
                    }
                }
                chain = parts;
                continue;
            }
        }
        break;
    }

    let rest: Vec<&str> = head[k..].iter().map(|t| t.text.as_str()).collect();
    let rest_typed: Vec<&str> = head[k..].iter().map(|t| t.marked.as_str()).collect();
    if !rest.is_empty() {
        let key = norm::street_key_tokens(&head[k..]);
        if !key.is_empty() {
            out.street = Some(key);
            let typed = rest_typed.join(" ");
            let typed = typed
                .strip_prefix("đường ")
                .or_else(|| typed.strip_prefix("phố "))
                .unwrap_or(&typed);
            out.street_typed = Some(typed.to_string());
        }
        out.name_words = Some(rest.join(" "));
    }
    out.alleys = marker_alleys;
    if let Some((last, init)) = chain.split_last() {
        out.alleys.extend(init.iter().cloned());
        out.house = Some(last.clone());
    }
    out.chain = chain;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(p: &ParsedAddress) -> (Vec<String>, Option<String>) {
        (
            p.alleys.iter().map(NumPart::key).collect(),
            p.house.as_ref().map(NumPart::key),
        )
    }

    #[test]
    fn full_address() {
        let p = parse("123/45/6 Nguyễn Trãi, P. Bến Thành, Q.1, TP.HCM");
        assert_eq!(k(&p), (vec!["123".into(), "45".into()], Some("6".into())));
        assert_eq!(p.chain_key().as_deref(), Some("123/45/6"));
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        assert_eq!(p.ward.as_deref(), Some("ben thanh"));
        assert_eq!(p.district.as_deref(), Some("1"));
        assert_eq!(p.city.as_deref(), Some("ho chi minh"));
        assert!(p.typed_with_marks);
    }

    #[test]
    fn suffix_letters() {
        let p = parse("123/45A Lê Lợi");
        assert_eq!(p.house.as_ref().unwrap().suffix, "a");
        assert_eq!(p.chain_key().as_deref(), Some("123/45a"));
        let p = parse("12 A Lê Lợi");
        assert_eq!(p.chain_key().as_deref(), Some("12a"));
        assert_eq!(p.street.as_deref(), Some("le loi"));
        let p = parse("số 12a lê lợi");
        assert_eq!(p.chain_key().as_deref(), Some("12a"));
    }

    #[test]
    fn alley_markers() {
        let p = parse("hẻm 45 số 12 Nguyễn Trãi");
        assert_eq!(k(&p), (vec!["45".into()], Some("12".into())));
        assert_eq!(
            p.number_keys(),
            vec![
                ("12".into(), NumberMatch::Exact),
                ("45/12".into(), NumberMatch::Exact),
            ]
        );
        let p = parse("Hẻm 45 Nguyễn Trãi");
        assert_eq!(k(&p), (vec!["45".into()], None));
        assert_eq!(p.number_keys(), vec![("45".into(), NumberMatch::AlleyRoot)]);
        let p = parse("45/6 Nguyễn Trãi");
        assert_eq!(
            p.number_keys(),
            vec![
                ("45/6".into(), NumberMatch::Exact),
                ("45".into(), NumberMatch::AlleyRoot)
            ]
        );
    }

    #[test]
    fn numbered_street_and_date_street() {
        let p = parse("12 Đường số 7, P. Tân Phú");
        assert_eq!(p.chain_key().as_deref(), Some("12"));
        assert_eq!(p.street.as_deref(), Some("7"));
        let p = parse("3 Tháng 2");
        assert!(p.house.is_none());
        assert_eq!(p.street.as_deref(), Some("3 thang 2"));
        let p = parse("12 3 Tháng 2, Q10");
        assert_eq!(p.chain_key().as_deref(), Some("12"));
        assert_eq!(p.street.as_deref(), Some("3 thang 2"));
        assert_eq!(p.district.as_deref(), Some("10"));
        let p = parse("Đường 3/2");
        assert_eq!(p.street.as_deref(), Some("3 thang 2"));
    }

    #[test]
    fn homograph_keywords() {
        // có dấu: Huyền là tên người, không phải «huyện»
        let p = parse("Huyền Trân Công Chúa, Quận 1");
        assert_eq!(p.street.as_deref(), Some("huyen tran cong chua"));
        assert_eq!(p.district.as_deref(), Some("1"));
        // sau «đường»: tên đường
        let p = parse("12 Đường Huyen Tran Cong Chua");
        assert_eq!(p.street.as_deref(), Some("huyen tran cong chua"));
        assert!(p.district.is_none());
        // không dấu, không «đường»: không phân biệt được, bị đọc là huyện (giới hạn đã biết)
        let p = parse("Huyen Tran Cong Chua");
        assert!(p.street.is_none());
        assert_eq!(p.district.as_deref(), Some("tran cong chua"));
    }

    #[test]
    fn admin_variants() {
        let p = parse("Phường 12 Quận Tân Bình");
        assert_eq!(p.ward.as_deref(), Some("12"));
        assert_eq!(p.district.as_deref(), Some("tan binh"));
        assert!(p.street.is_none());
        let p = parse("Lê Lợi Bến Nghé Quận 1 Ho Chi Minh");
        assert_eq!(p.city.as_deref(), Some("ho chi minh"));
        let p = parse("Đường Hồ Chí Minh");
        assert!(p.city.is_none());
        let p = parse("TP. Thủ Đức");
        assert_eq!(p.district.as_deref(), Some("thanh pho thu duc"));
        let p = parse("Bình Dương");
        assert!(p.province.is_none() && p.street.is_some());
        let p = parse("Tỉnh Bình Dương");
        assert_eq!(p.province.as_deref(), Some("binh duong"));
    }

    #[test]
    fn trailing_segments_do_not_leak_into_the_street() {
        let p = parse("12 Nguyễn Trãi, Ho Chi Minh City");
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        assert_eq!(p.city.as_deref(), Some("ho chi minh"));
        let p = parse("12 Nguyen Trai, Ho Chi Minh City, Vietnam");
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        assert_eq!(p.city.as_deref(), Some("ho chi minh"));
        let p = parse("12 Nguyen Trai, Viet Nam");
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        assert!(p.ward.is_none());
        // phường viết không có từ khoá, theo kiểu địa chỉ Việt Nam
        let p = parse("12 Nguyễn Trãi, Bến Nghé, Quận 1");
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        assert_eq!(p.ward.as_deref(), Some("ben nghe"));
        assert_eq!(p.district.as_deref(), Some("1"));
        // dấu phẩy ngay sau số nhà không phải ranh giới
        let p = parse("Số 12, Nguyễn Trãi");
        assert_eq!(p.chain_key().as_deref(), Some("12"));
        assert_eq!(p.street.as_deref(), Some("nguyen trai"));
        let p = parse("12 Nguyen Trai, District 1, Ward 5");
        assert_eq!(p.district.as_deref(), Some("1"));
        assert_eq!(p.ward.as_deref(), Some("5"));
        let p = parse("Đường Việt Nam");
        assert_eq!(p.street.as_deref(), Some("viet nam"));
    }

    #[test]
    fn named_alley_as_street() {
        // «Hẻm 65» là tên đường (addr:street) khi không còn từ nào sau nó
        let p = parse("65/59 Hẻm 65");
        assert_eq!(p.chain_key().as_deref(), Some("65/59"));
        assert_eq!(p.street.as_deref(), Some("hem 65"));
        // còn từ phía sau thì «hẻm 65» là hẻm
        let p = parse("hẻm 65 Lê Lợi");
        assert_eq!(p.street.as_deref(), Some("le loi"));
        assert_eq!(p.alleys.len(), 1);
        // tên riêng «Dương» có dấu không bị bỏ như từ loại
        let p = parse("Dương Quảng Hàm");
        assert_eq!(p.street.as_deref(), Some("duong quang ham"));
        let p = parse("Đường Dương Quảng Hàm");
        assert_eq!(p.street.as_deref(), Some("duong quang ham"));
    }

    #[test]
    fn empty_and_garbage() {
        assert_eq!(parse("").street, None);
        assert_eq!(parse(" , . / - ").street, None);
        let _ = parse("////---...,,,");
        let _ = parse("99999999999999999999999 a/b/c");
    }
}
