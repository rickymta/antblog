//! Chuẩn hoá chuỗi địa chỉ tiếng Việt, có dấu và không dấu.
//!
//! Một chữ có dấu có thể đến dưới hai dạng Unicode: dựng sẵn (NFC, `ễ` là một ký tự) hoặc tách rời (NFD,
//! `e` + dấu mũ + dấu ngã). Hai chuỗi trông giống hệt nhau nhưng khác byte, nên so sánh thô sẽ trượt.
//! Quy trình ở đây:
//!
//! 1. đưa về NFC rồi chữ thường (`Đ` thành `đ`),
//! 2. tách thành token: chữ và số giữ lại, `/` và `-` kẹp giữa hai ký tự chữ số được giữ (số nhà `123/45/6`, `12-14`),
//!    mọi dấu câu khác là dấu ngăn; dấu `.` ngay sau token được ghi nhớ (`Đ.` là viết tắt, `đ` đứng một mình thì chưa chắc),
//! 3. bỏ dấu: NFD, xoá dấu kết hợp, `đ` thành `d`,
//! 4. mở rộng viết tắt (`P.`, `Q.`, `TP.`, `Ng.`, `KP`, `ql1a`…), bỏ chữ «số», «No.».
//!
//! Hai dạng đầu ra: [`normalize`] (khoá tìm kiếm, ASCII chữ thường) và [`lower_nfc`] (giữ dấu, dùng để phân biệt
//! các từ chỉ khác dấu như `Bà` và `Ba` khi người dùng có gõ dấu).

use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Đưa về NFC (dạng dựng sẵn). Idempotent.
pub fn nfc(s: &str) -> String {
    s.nfc().collect()
}

/// NFC, chữ thường, gộp khoảng trắng; **giữ dấu**.
pub fn lower_nfc(s: &str) -> String {
    collapse_ws(&nfc(s).to_lowercase())
}

/// Chữ thường và bỏ dấu: NFD, xoá dấu kết hợp (U+0300 trở đi), `đ` thành `d`. Giữ nguyên các ký tự khác,
/// kể cả dấu câu. Nhận cả chuỗi NFC lẫn NFD.
pub fn strip_marks(s: &str) -> String {
    s.to_lowercase()
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| if c == 'đ' { 'd' } else { c })
        .collect()
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Một token sau khi tách. `text` đã bỏ dấu và chỉ gồm `[a-z0-9/-]`; `marked` là dạng NFC chữ thường còn dấu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub marked: String,
    /// Ngay sau token có dấu chấm (`P.`, `Đ.`, `TP.`).
    pub dotted: bool,
    /// Sau token có dấu phẩy, chấm phẩy hoặc gạch ngang đứng riêng: ranh giới giữa hai phần của địa chỉ.
    pub comma_after: bool,
}

fn is_comma(c: char) -> bool {
    matches!(c, ',' | ';' | '-' | '|' | '\n')
}

fn make_token(raw: &str, dotted: bool, comma_after: bool) -> Option<Token> {
    // bỏ dấu có thể làm hai ký tự nối `/` `-` dính nhau (dấu kết hợp đứng giữa): gộp thành một
    let mut text = String::new();
    for c in strip_marks(raw).chars() {
        let joiner = c == '/' || c == '-';
        if c.is_ascii_alphanumeric() || (joiner && !text.ends_with(['/', '-'])) {
            text.push(c);
        }
    }
    let text = text.trim_matches(['/', '-']).to_string();
    if text.is_empty() {
        return None;
    }
    Some(Token {
        text,
        marked: raw.to_string(),
        dotted,
        comma_after,
    })
}

/// Bỏ khoảng trắng quanh dấu `/` khi hai bên đều là chữ hoặc số: `45 / 6` thành `45/6`.
fn join_slashes(s: &str) -> String {
    let cs: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let prev_alnum = |out: &str| out.chars().last().is_some_and(char::is_alphanumeric);
    let skip_ws = |mut k: usize| {
        while k < cs.len() && cs[k].is_whitespace() {
            k += 1;
        }
        k
    };
    let mut i = 0;
    while i < cs.len() {
        let c = cs[i];
        if c.is_whitespace() {
            let j = skip_ws(i);
            if j < cs.len() && cs[j] == '/' {
                let k = skip_ws(j + 1);
                if prev_alnum(&out) && k < cs.len() && cs[k].is_alphanumeric() {
                    out.push('/');
                    i = k;
                    continue;
                }
            }
            out.push(' ');
            i = j;
            continue;
        }
        if c == '/' {
            let k = skip_ws(i + 1);
            if k > i + 1 && k < cs.len() && cs[k].is_alphanumeric() && prev_alnum(&out) {
                out.push('/');
                i = k;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Tách token, chưa mở rộng viết tắt. Xem phần đầu module về quy tắc tách.
pub fn lex(input: &str) -> Vec<Token> {
    let s = join_slashes(&lower_nfc(input));
    let chars: Vec<char> = s.chars().collect();
    let mut toks: Vec<Token> = Vec::new();
    let mut cur = String::new();
    for (i, &c) in chars.iter().enumerate() {
        let next_alnum = chars.get(i + 1).is_some_and(|n| n.is_alphanumeric());
        if c.is_alphanumeric() || ((c == '/' || c == '-') && !cur.is_empty() && next_alnum) {
            cur.push(c);
        } else {
            if cur.is_empty() {
                if let Some(last) = toks.last_mut() {
                    last.dotted |= c == '.';
                    last.comma_after |= is_comma(c);
                }
            } else {
                let tok = make_token(&cur, c == '.', is_comma(c));
                cur.clear();
                match tok {
                    Some(t) => toks.push(t),
                    None => {
                        // token toàn ký tự ngoài ASCII (chữ Hán…): bỏ, nhưng giữ ranh giới phẩy
                        if let Some(last) = toks.last_mut() {
                            last.comma_after |= is_comma(c);
                        }
                    }
                }
            }
        }
    }
    if !cur.is_empty() {
        toks.extend(make_token(&cur, false, false));
    }
    toks
}

fn is_numberish(t: &str) -> bool {
    t.as_bytes().first().is_some_and(u8::is_ascii_digit)
}

fn push(out: &mut Vec<Token>, text: &str) {
    out.push(Token {
        text: text.to_string(),
        marked: text.to_string(),
        dotted: false,
        comma_after: false,
    });
}

fn push_words(out: &mut Vec<Token>, words: &str, comma_after: bool) {
    for w in words.split(' ') {
        push(out, w);
    }
    if let Some(last) = out.last_mut() {
        last.comma_after |= comma_after;
    }
}

/// Tên đầy đủ của thành phố sau «thanh pho»: nếu đầu ra đã kết thúc bằng «thanh pho» thì chỉ thêm phần tên.
fn push_city(out: &mut Vec<Token>, comma_after: bool) {
    let has_prefix =
        out.len() >= 2 && out[out.len() - 2].text == "thanh" && out[out.len() - 1].text == "pho";
    push_words(
        out,
        if has_prefix {
            "ho chi minh"
        } else {
            "thanh pho ho chi minh"
        },
        comma_after,
    );
}

/// Mở rộng viết tắt trên dãy token đã tách. Quy tắc cục bộ theo từng token (xem bảng trong mã), nên kết quả ổn định
/// khi chuẩn hoá lần thứ hai.
pub fn expand(tokens: Vec<Token>) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(tokens.len() + 4);
    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        let t = tok.text.as_str();
        let comma = tok.comma_after;
        let next = tokens.get(i + 1).map(|n| n.text.as_str());
        let next_num = next.is_some_and(is_numberish);
        let prev = out.last().map(|p| p.text.as_str());
        i += 1;

        // đầu câu «địa chỉ …»
        if t == "dia" && next == Some("chi") && out.is_empty() {
            i += 1;
            continue;
        }
        // chữ «số», «số nhà», «No.», «Nr»: cả dãy từ liên tiếp bị bỏ nếu theo sau là số
        // (riêng «số» ngay sau «đường»/«phố» là một phần tên đường, giữ lại một từ)
        if matches!(t, "so" | "no" | "nr" | "num") {
            let start = i - 1;
            let mut j = start;
            while let Some(w) = tokens.get(j).map(|x| x.text.as_str()) {
                let prev_w = if j > start {
                    tokens.get(j - 1).map(|x| x.text.as_str())
                } else {
                    None
                };
                if matches!(w, "so" | "no" | "nr" | "num") || (w == "nha" && prev_w == Some("so")) {
                    j += 1;
                } else {
                    break;
                }
            }
            if tokens.get(j).is_some_and(|x| is_numberish(&x.text)) {
                if t == "so" && matches!(prev, Some("duong" | "pho")) {
                    push_words(&mut out, "so", false);
                }
                i = j;
                continue;
            }
        }
        // viết tắt có dấu chấm hoặc đứng ngay trước số
        let dotted = tok.dotted;
        match t {
            "d" if dotted => push_words(&mut out, "duong", comma),
            "p" if dotted || next_num => push_words(&mut out, "phuong", comma),
            "q" if dotted || next_num => push_words(&mut out, "quan", comma),
            "ng" if dotted => push_words(&mut out, "ngo", comma),
            "h" if dotted && !next_num => push_words(&mut out, "huyen", comma),
            "h" if next_num => push_words(&mut out, "hem", comma),
            "x" if dotted => push_words(&mut out, "xa", comma),
            "t" if dotted => push_words(&mut out, "tinh", comma),
            "district" | "dist" if next.is_some() => push_words(&mut out, "quan", comma),
            "ward" if next.is_some() => push_words(&mut out, "phuong", comma),
            "tp" => push_words(&mut out, "thanh pho", comma),
            "tphcm" | "hcm" | "hcmc" => push_city(&mut out, comma),
            "kp" => push_words(&mut out, "khu pho", comma),
            "tt" => push_words(&mut out, "thi tran", comma),
            "tx" => push_words(&mut out, "thi xa", comma),
            "ql" => push_words(&mut out, "quoc lo", comma),
            "tl" => push_words(&mut out, "tinh lo", comma),
            _ => {
                // dạng dính liền số: p12, q1, h45, kp3, ql1a
                let split = ["kp", "ql", "p", "q", "h"].iter().find_map(|pre| {
                    let rest = t.strip_prefix(pre)?;
                    (is_numberish(rest)).then_some((*pre, rest))
                });
                if let Some((pre, rest)) = split {
                    let words = match pre {
                        "kp" => "khu pho",
                        "ql" => "quoc lo",
                        "p" => "phuong",
                        "q" => "quan",
                        _ => "hem",
                    };
                    push_words(&mut out, words, false);
                    out.push(Token {
                        text: rest.to_string(),
                        marked: rest.to_string(),
                        dotted: false,
                        comma_after: comma,
                    });
                } else {
                    out.push(Token {
                        dotted: false,
                        ..tok.clone()
                    });
                }
            }
        }
    }
    out
}

/// Tách token và mở rộng viết tắt.
pub fn tokens(input: &str) -> Vec<Token> {
    expand(lex(input))
}

/// Khoá tìm kiếm: bỏ dấu, chữ thường, mở rộng viết tắt, các token cách nhau một khoảng trắng. Chỉ gồm `[a-z0-9/-]` và
/// khoảng trắng; không chứa dấu câu. Idempotent: `normalize(normalize(x)) == normalize(x)`.
pub fn normalize(input: &str) -> String {
    tokens(input)
        .iter()
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Chỉ bỏ dấu và chữ thường, không mở rộng viết tắt; thay mọi thứ không phải chữ số/chữ cái bằng khoảng trắng.
pub fn fold(input: &str) -> String {
    let s: String = strip_marks(&nfc(input))
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect();
    collapse_ws(&s)
}

/// Khoá tên đường: [`normalize`], bỏ từ loại đầu «đường»/«phố», bỏ «số» đứng trước số (`đường số 7` và `đường 7` cùng
/// khoá `7`), và đổi `d/m` thành «d tháng m» (`3/2` là đường 3 Tháng 2; chỉ áp cho khoá tên đường, không áp cho số nhà).
///
/// Từ đầu `duong` chỉ bị bỏ nếu nó là từ loại: viết `đường`, hoặc không dấu. Nếu có dấu mà khác (`Dương Quảng Hàm`) thì giữ,
/// vì đó là tên người. Không dấu thì không phân biệt được và coi là từ loại; [`crate::Index::geocode`] thử lại với `duong ` đứng trước.
/// Vì phụ thuộc dấu nên không idempotent trên chuỗi đã bỏ dấu.
pub fn street_key(name: &str) -> String {
    street_key_tokens(&tokens(name))
}

/// Như [`street_key`] trên token đã tách (còn dạng có dấu để phân biệt từ loại với tên riêng).
pub fn street_key_tokens(toks: &[Token]) -> String {
    let mut words: Vec<String> = toks.iter().map(|t| t.text.clone()).collect();
    let type_word = toks.first().is_some_and(|t| {
        matches!(t.text.as_str(), "duong" | "pho")
            && (t.marked == t.text || matches!(t.marked.as_str(), "đường" | "phố"))
    });
    if type_word && words.len() > 1 {
        words.remove(0);
    }
    if words.len() >= 2 && words[0] == "so" && is_numberish(&words[1]) {
        words.remove(0);
    }
    for w in &mut words {
        if let Some((d, m)) = w.split_once('/')
            && let (Ok(dn), Ok(mn)) = (d.parse::<u32>(), m.parse::<u32>())
            && (1..=31).contains(&dn)
            && (1..=12).contains(&mn)
        {
            *w = format!("{dn} thang {mn}");
        }
    }
    words.join(" ")
}

/// Khoá tên đơn vị hành chính: [`normalize`] rồi bỏ từ loại đầu (`phường`, `quận`, `thành phố`…). `Phường 12` thành `12`.
pub fn admin_key(name: &str) -> String {
    let n = normalize(name);
    let rest = [
        "thanh pho ",
        "thi tran ",
        "thi xa ",
        "phuong ",
        "quan ",
        "huyen ",
        "xa ",
        "tinh ",
    ]
    .iter()
    .find_map(|p| n.strip_prefix(p));
    rest.map_or(n.clone(), str::to_string)
}

/// Khoá số nhà của một giá trị đơn: bỏ chữ «số»/«No.», dính liền chữ cái hậu tố (`12 A` thành `12a`), giữ `/` và `-`.
pub fn hn_key(value: &str) -> String {
    lex(value)
        .iter()
        .filter(|t| !matches!(t.text.as_str(), "so" | "no" | "nha" | "nr"))
        .map(|t| t.text.as_str())
        .collect::<String>()
}

/// Nhiều số nhà trong một thẻ OSM (`12;14`, `12, 14`) thành nhiều khoá.
pub fn hn_keys(value: &str) -> Vec<String> {
    value
        .split([';', ','])
        .map(hn_key)
        .filter(|k| !k.is_empty())
        .collect()
}

/// Có dấu thanh/dấu mũ nào trong chuỗi không (người dùng có gõ dấu).
pub fn has_diacritics(s: &str) -> bool {
    let n = nfc(s).to_lowercase();
    n.chars().any(|c| !c.is_ascii() && c.is_alphabetic())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combining_marks_equal_precomposed() {
        let pre = "Nguyễn Trãi";
        let dec = "Nguye\u{0302}\u{0303}n Tra\u{0303}i"; // e + mũ + ngã, a + ngã
        assert_ne!(pre, dec);
        assert_eq!(nfc(dec), pre);
        assert_eq!(normalize(pre), "nguyen trai");
        assert_eq!(normalize(dec), "nguyen trai");
        assert_eq!(lower_nfc(dec), lower_nfc(pre));
    }

    #[test]
    fn d_with_stroke() {
        assert_eq!(normalize("Đường ĐỒNG Khởi"), "duong dong khoi");
        assert_eq!(strip_marks("đĐ"), "dd");
    }

    #[test]
    fn abbreviations() {
        assert_eq!(
            normalize("123 Đ. Lê Lợi, P.Bến Nghé, Q.1, TP.HCM"),
            "123 duong le loi phuong ben nghe quan 1 thanh pho ho chi minh"
        );
        assert_eq!(normalize("P12 Q10"), "phuong 12 quan 10");
        assert_eq!(normalize("Ng. 5 KP3"), "ngo 5 khu pho 3");
        assert_eq!(normalize("tp. Hồ Chí Minh"), "thanh pho ho chi minh");
        assert_eq!(normalize("H45 QL1A"), "hem 45 quoc lo 1a");
    }

    #[test]
    fn number_words_dropped_but_street_number_kept() {
        assert_eq!(normalize("Số 12A Lê Lợi"), "12a le loi");
        assert_eq!(normalize("No. 12 Lê Lợi"), "12 le loi");
        assert_eq!(normalize("Đường số 7"), "duong so 7");
        assert_eq!(street_key("Đường số 7"), "7");
        assert_eq!(street_key("Đường 7"), "7");
        assert_eq!(street_key("Đường 3/2"), "3 thang 2");
        assert_eq!(street_key("3 Tháng 2"), "3 thang 2");
        // «Dương» là tên riêng khi có dấu: giữ; «Đường» và «duong» không dấu là từ loại: bỏ
        assert_eq!(street_key("Dương Quảng Hàm"), "duong quang ham");
        assert_eq!(street_key("Đường Dương Quảng Hàm"), "duong quang ham");
        assert_eq!(street_key("Duong Quang Ham"), "quang ham");
    }

    #[test]
    fn spaced_slash() {
        assert_eq!(normalize("45 / 6 Nguyễn Trãi"), "45/6 nguyen trai");
        assert_eq!(normalize("123/ 45 /6 Lê Lợi"), "123/45/6 le loi");
        assert_eq!(normalize("a / b"), "a/b");
        assert_eq!(normalize("12 /"), "12");
    }

    #[test]
    fn house_number_keys() {
        assert_eq!(hn_key("123/45/6"), "123/45/6");
        assert_eq!(hn_key("12 A"), "12a");
        assert_eq!(hn_key("Số 12-A"), "12-a");
        assert_eq!(hn_keys("12;14"), vec!["12", "14"]);
    }

    #[test]
    fn diacritics_detect() {
        assert!(has_diacritics("Bà"));
        assert!(!has_diacritics("Ba"));
        assert!(has_diacritics("Ba\u{0300}"));
    }

    #[test]
    fn idempotent_on_examples() {
        for s in [
            "Q.1, P. Bến Nghé, TP.HCM",
            "số 12 hẻm 45",
            "hcm",
            "tp hcm",
            "so so 12",
            "d so 5",
            "no no 5",
            "x ql",
        ] {
            let n = normalize(s);
            assert_eq!(normalize(&n), n, "đầu vào: {s:?}");
        }
    }
}
