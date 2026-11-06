"""Hậu xử lý kết quả OCR: từ danh sách từ (có khung chữ nhật, confidence) đến trường có kiểu.

Luồng: gom từ thành dòng -> tìm nhãn bằng so khớp mờ -> lấy dòng giá trị bên dưới nhãn ->
chuẩn hóa Unicode -> sửa ký tự dễ nhầm THEO NGỮ CẢNH TRƯỜNG -> regex có tên -> kiểm tra
cấu trúc -> so khớp mờ với danh sách đóng. Mỗi bước ghi lại điều nó đã sửa hoặc nghi ngờ,
để người duyệt thấy vì sao một trường bị đưa vào hàng chờ.

Dữ liệu trong lab hoàn toàn là dữ liệu giả (số 099..., GPLX 99...). Kết quả của module này
là ĐẦU VÀO cho người duyệt, không phải quyết định xác minh giấy tờ.
"""

import re
import unicodedata
from dataclasses import dataclass, replace
from datetime import date

from rapidfuzz import fuzz, process

from ocr import Word
from synthetic_card import CITIES, LICENSE_CLASSES

# ---- Chuẩn hóa chuỗi --------------------------------------------------------------------------

# Dấu nháy cong, dấu gạch ngang các loại, khoảng trắng không ngắt, ký tự rộng bằng 0.
_CONFUSABLE = str.maketrans({
    "\u2018": "'", "\u2019": "'", "\u201c": '"', "\u201d": '"',  # nháy cong
    "\u2010": "-", "\u2011": "-", "\u2012": "-", "\u2013": "-", "\u2014": "-",  # các loại gạch ngang
    "\u00a0": " ",  # khoảng trắng không ngắt
    "\u200b": None, "\ufeff": None,  # ký tự rộng bằng 0
})


def normalize_text(text: str) -> str:
    """NFC, gộp ký tự «giống nhau», gộp mọi khoảng trắng về một dấu cách."""
    text = unicodedata.normalize("NFC", text).translate(_CONFUSABLE)
    return " ".join(text.split())  # split() không đối số tách theo mọi khoảng trắng Unicode


def fold(text: str) -> str:
    """Bỏ dấu và hạ chữ thường, để so sánh «gần đúng»: 'Cần Thơ' và 'CAN THO' cho cùng kết quả.

    Đ/đ không phân rã trong NFD nên đổi tay.
    """
    decomposed = unicodedata.normalize("NFD", text.replace("Đ", "D").replace("đ", "d"))
    return "".join(ch for ch in decomposed if not unicodedata.combining(ch)).casefold()


# ---- Sửa ký tự dễ nhầm theo ngữ cảnh ----------------------------------------------------------

# Chỉ dùng khi ta BIẾT trường phải là số. Ở trường tên, bảng ngược lại mới có nghĩa.
TO_DIGIT = str.maketrans({"O": "0", "o": "0", "Q": "0", "I": "1", "l": "1", "|": "1", "!": "1",
                          "Z": "2", "S": "5", "G": "6", "B": "8"})
TO_LETTER = str.maketrans({"0": "O", "1": "I", "5": "S", "8": "B"})


def fix_digits(text: str) -> tuple[str, list[str]]:
    """Đổi ký tự giống chữ số thành chữ số. Trả (chuỗi mới, danh sách sửa để audit)."""
    out, notes = [], []
    for i, ch in enumerate(text):
        fixed = ch.translate(TO_DIGIT)
        if fixed != ch:
            notes.append(f"{ch}->{fixed}@{i}")
        out.append(fixed)
    return "".join(out), notes


def fix_letters(text: str) -> tuple[str, list[str]]:
    """Trong tên người: chữ số nằm GIỮA hai chữ cái thì đổi lại thành chữ cái giống nó.

    Chỉ đổi khi cả ký tự trước và sau đều là chữ cái ('NGUY0N' thành 'NGUYON'). Chữ số ở đầu
    hay cuối từ ('VAN5') hoặc đứng riêng không bị đổi: chúng có thể là chữ số thật hoặc vệt nhiễu,
    nên để nguyên cho `parse_name` từ chối (digit_in_name) thay vì đoán.
    """
    out, notes = [], []
    for i, ch in enumerate(text):
        if (ch in "0158" and 0 < i < len(text) - 1
                and text[i - 1].isalpha() and text[i + 1].isalpha()):
            fixed = ch.translate(TO_LETTER)
            notes.append(f"{ch}->{fixed}@{i}")
            ch = fixed
        out.append(ch)
    return "".join(out), notes


# ---- Regex có tên -----------------------------------------------------------------------------

ID_RE = re.compile(r"(?P<province>\d{3})(?P<century>\d)(?P<year2>\d{2})(?P<serial>\d{6})")
LICENSE_RE = re.compile(r"(?P<prefix>99)(?P<serial>\d{10})")
DATE_RE = re.compile(r"(?P<day>\d{2})/(?P<month>\d{2})/(?P<year>\d{4})")
# Tên: 2 đến 6 từ, mỗi từ chỉ gồm chữ cái (\w trừ chữ số và gạch dưới). Chỉ đúng với chuỗi NFC.
NAME_RE = re.compile(r"[^\W\d_]+(?: [^\W\d_]+){1,5}")
CLASS_RE = re.compile(r"[A-Z][0-9]?")

# Mã tỉnh mà lab chấp nhận: `099` là mã KHÔNG tồn tại, dùng cho dữ liệu giả. Hệ thống thật
# phải nạp bảng mã tỉnh từ văn bản chính thức; lab không tự suy ra bảng đó.
ACCEPTED_PREFIXES = frozenset({"099"})

FUZZY_CITY_MIN = 80.0  # điểm tối thiểu của ứng viên tốt nhất
FUZZY_CITY_MARGIN = 10.0  # ứng viên tốt nhất phải hơn ứng viên thứ hai ít nhất từng này điểm
FUZZY_SEX_MIN = 70.0
LABEL_MIN = 75.0
MIN_WORD_HEIGHT = 8  # từ cao dưới 8 điểm ảnh là vệt nhiễu, không phải chữ in trên thẻ
MAX_VALUE_GAP = 90  # tâm dòng giá trị cách tâm dòng nhãn không quá 90 điểm ảnh (khổ thẻ 1012 x 638)


# ---- Kiểu kết quả -----------------------------------------------------------------------------


@dataclass(frozen=True)
class FieldResult:
    name: str
    raw: str | None  # các từ OCR nối lại, chưa xử lý gì
    value: str | None  # giá trị đã chuẩn hóa; None nếu không dùng được
    confidence: float  # NHỎ NHẤT trong confidence của các từ tạo nên trường (0..1)
    corrections: tuple[str, ...] = ()
    issues: tuple[str, ...] = ()
    word_confs: tuple[float, ...] = ()  # confidence từng từ, để thấy vì sao lấy min

    @property
    def ok(self) -> bool:
        return self.value is not None and not self.issues


@dataclass(frozen=True)
class CardResult:
    doc_type: str  # "cccd" | "gplx"
    fields: dict[str, FieldResult]

    @property
    def ok(self) -> bool:
        return all(f.ok for f in self.fields.values())


# Trường theo loại giấy tờ: (tên trường, nhãn in trên thẻ, kiểu).
SPECS: dict[str, tuple[tuple[str, str, str], ...]] = {
    "cccd": (("id_number", "Số / No.:", "id"), ("full_name", "Họ và tên:", "name"),
             ("date_of_birth", "Ngày sinh:", "date"), ("sex", "Giới tính:", "sex"),
             ("hometown_city", "Quê quán:", "city")),
    "gplx": (("license_number", "Số / No.:", "license"), ("full_name", "Họ và tên:", "name"),
             ("date_of_birth", "Ngày sinh:", "date"), ("license_class", "Hạng / Class:", "class"),
             ("expiry_date", "Có giá trị đến:", "date")),
}


# ---- Phân tích từng kiểu trường ---------------------------------------------------------------
# Mỗi hàm nhận chuỗi đã normalize_text, trả (giá trị hoặc None, sửa, vấn đề).


def parse_id_number(text: str) -> tuple[str | None, list[str], list[str]]:
    compact = re.sub(r"[\s.\-]", "", text)  # OCR hay chèn dấu cách hoặc chấm giữa dãy số
    fixed, notes = fix_digits(compact)
    if ID_RE.fullmatch(fixed) is None:
        return None, notes, [f"not_12_digits:{len(fixed)}"]
    return fixed, notes, []


def parse_license_number(text: str) -> tuple[str | None, list[str], list[str]]:
    compact = re.sub(r"[\s.\-]", "", text)
    fixed, notes = fix_digits(compact)
    if LICENSE_RE.fullmatch(fixed) is None:
        return None, notes, [f"not_99_plus_10_digits:{len(fixed)}"]
    return fixed, notes, []


def parse_date(text: str) -> tuple[str | None, list[str], list[str]]:
    # Dấu phân cách thật có thể thành '-', '.', ',' và OCR hay chèn dấu cách thừa ('1 978').
    unified = re.sub(r"[\-.,:]", "/", re.sub(r"\s+", "", text))
    fixed, notes = fix_digits(unified)
    match = DATE_RE.fullmatch(fixed)
    if match is None:
        return None, notes, ["not_dd/mm/yyyy"]
    try:  # fullmatch chỉ kiểm hình dạng; date() mới biết 31/02 hay 48/10 không tồn tại
        date(int(match["year"]), int(match["month"]), int(match["day"]))
    except ValueError:
        return fixed, notes, ["impossible_date"]
    return fixed, notes, []


def parse_name(text: str) -> tuple[str | None, list[str], list[str]]:
    fixed, notes = fix_letters(text)
    if any(ch.isdigit() for ch in fixed):  # còn chữ số mà bảng không đổi được: không đoán, không xóa lặng lẽ
        return None, notes, ["digit_in_name"]
    kept = "".join(ch for ch in fixed if ch.isalpha() or ch == " ")  # bỏ dấu câu, vệt nhiễu
    kept = " ".join(kept.split()).upper()
    if NAME_RE.fullmatch(kept) is None:
        return None, notes, ["not_a_name"]
    return kept, notes, []


def parse_sex(text: str) -> tuple[str | None, list[str], list[str]]:
    best = process.extractOne(fold(text), {"Nam": "nam", "Nữ": "nu"}, scorer=fuzz.ratio,
                              score_cutoff=FUZZY_SEX_MIN)
    if best is None:
        return None, [], ["sex_unknown"]
    _, score, canonical = best
    return canonical, ([f"fuzzy:{text}->{canonical}@{score:.0f}"] if text != canonical else []), []


def match_city(text: str) -> tuple[str | None, float, float]:
    """So khớp mờ (đã bỏ dấu) với danh sách đóng. Trả (thành phố hoặc None, điểm, điểm hạng nhì)."""
    ranked = process.extract(fold(text), {city: fold(city) for city in CITIES}, scorer=fuzz.ratio, limit=2)
    (_, best_score, best_city), (_, second_score, _) = ranked
    if best_score >= FUZZY_CITY_MIN and best_score - second_score >= FUZZY_CITY_MARGIN:
        return best_city, best_score, second_score
    return None, best_score, second_score


def parse_city(text: str) -> tuple[str | None, list[str], list[str]]:
    city, score, second = match_city(text)
    if city is None:
        return None, [], [f"city_not_confident:{score:.0f}/{second:.0f}"]
    return city, ([f"fuzzy:{text}->{city}@{score:.0f}"] if text != city else []), []


def parse_class(text: str) -> tuple[str | None, list[str], list[str]]:
    compact = re.sub(r"[\s.]", "", text).upper()
    # Chữ đầu là chữ cái, phần sau (nếu có) là chữ số: 'Al' đọc lại thành 'A1' (chữ l thường
    # và số 1 gần như cùng nét trong nhiều font). Upper() đã đổi 'l' thành 'L', nên bảng ở đây
    # xét cả 'L'.
    tail, notes = fix_digits(compact[1:].replace("L", "l"))
    cleaned = compact[:1] + tail
    if CLASS_RE.fullmatch(cleaned) is None or cleaned not in LICENSE_CLASSES:
        return None, notes, ["license_class_unknown"]
    return cleaned, notes, []


PARSERS = {"id": parse_id_number, "license": parse_license_number, "date": parse_date,
           "name": parse_name, "sex": parse_sex, "city": parse_city, "class": parse_class}


# ---- Kiểm tra cấu trúc số CCCD ----------------------------------------------------------------


def expected_century_digit(sex: str, birth_year: int) -> int | None:
    """Thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3. Ngoài hai thế kỷ này: None."""
    base = {1900: 0, 2000: 2}.get(birth_year // 100 * 100)
    if base is None or sex not in ("Nam", "Nữ"):
        return None
    return base + (0 if sex == "Nam" else 1)


def check_id_structure(id_number: str, birth_date: str | None, sex: str | None) -> list[str]:
    """Kiểm số 12 chữ số có KHỚP với ngày sinh và giới tính trên cùng thẻ không.

    Đây là kiểm tra nhất quán chứ không phải checksum: sáu chữ số cuối là số ngẫu nhiên nên
    không có gì để tính lại. Một chữ số cuối bị đọc sai KHÔNG bị bắt ở đây.
    """
    match = ID_RE.fullmatch(id_number)
    if match is None:
        return ["not_12_digits"]
    issues = []
    if match["province"] not in ACCEPTED_PREFIXES:
        issues.append(f"province_code_not_accepted:{match['province']}")
    date_match = DATE_RE.fullmatch(birth_date or "")
    if date_match is None or sex is None:
        return [*issues, "century_and_year_unchecked"]
    year = int(date_match["year"])
    if int(match["century"]) != expected_century_digit(sex, year):
        issues.append(f"century_digit_mismatch:{match['century']}")
    if match["year2"] != f"{year % 100:02d}":
        issues.append(f"year_digits_mismatch:{match['year2']}")
    return issues


def luhn_valid(digits: str) -> bool:
    """Luhn (ISO/IEC 7812): checksum CÔNG KHAI của số thẻ thanh toán. Lab không dùng nó cho
    CCCD hay GPLX; nó ở đây để so sánh với số có chữ số kiểm tra thật."""
    if not digits.isdigit():
        return False
    total = 0
    for i, ch in enumerate(reversed(digits)):
        d = int(ch)
        if i % 2 == 1:
            d = d * 2 - 9 if d > 4 else d * 2
        total += d
    return total % 10 == 0


# ---- Từ danh sách từ OCR đến trường -----------------------------------------------------------


@dataclass(frozen=True)
class Line:
    key: tuple[int, int, int]
    words: tuple[Word, ...]

    @property
    def text(self) -> str:
        return " ".join(w.text for w in self.words)

    @property
    def top(self) -> int:
        return min(w.top for w in self.words)

    @property
    def bottom(self) -> int:
        return max(w.bottom for w in self.words)

    @property
    def left(self) -> int:
        return min(w.left for w in self.words)


def group_lines(words: list[Word]) -> list[Line]:
    by_line: dict[tuple[int, int, int], list[Word]] = {}
    for w in words:
        if w.height >= MIN_WORD_HEIGHT:
            by_line.setdefault(w.line, []).append(w)
    return sorted((Line(k, tuple(sorted(v, key=lambda w: w.left))) for k, v in by_line.items()),
                  key=lambda ln: ln.top)


def label_score(line: Line, label: str) -> float:
    """Độ giống giữa đầu dòng và nhãn, đã bỏ dấu: chịu được 'S6/No.:' hay 'gay sinh:'.

    So `ratio` trên đoạn đầu dòng có độ dài gần bằng nhãn. Không dùng `partial_ratio`: nó
    căn chuỗi ngắn vào chuỗi dài, nên dòng chỉ có chữ 'D' cũng «chứa» trọn nhãn 'Có giá trị đến:' (đã bỏ dấu, nhãn chứa chữ 'd').
    """
    target = fold(label)
    head = fold(normalize_text(line.text))[: len(target) + 2]
    return float(fuzz.ratio(target, head))


def find_labels(lines: list[Line], spec: tuple[tuple[str, str, str], ...]) -> dict[str, tuple[Line, float]]:
    """Gán mỗi nhãn một dòng (mỗi dòng cho tối đa một nhãn), ưu tiên cặp điểm cao trước."""
    pairs = sorted(((label_score(ln, label), name, ln) for name, label, _ in spec for ln in lines),
                   key=lambda p: -p[0])
    found: dict[str, tuple[Line, float]] = {}
    used: set[tuple[int, int, int]] = set()
    for score, name, ln in pairs:
        if score >= LABEL_MIN and name not in found and ln.key not in used:
            found[name] = (ln, score)
            used.add(ln.key)
    return found


def value_words(label_line: Line, label_keys: set[tuple[int, int, int]], lines: list[Line]) -> list[Word]:
    """Từ của dòng giá trị: dòng kế tiếp theo thứ tự đọc có chữ ở cột giá trị.

    Không dùng «đỉnh dòng giá trị phải dưới đáy dòng nhãn»: khung chữ của ảnh mờ hay nghiêng
    chồng lên nhau theo chiều dọc (đã gặp), nên thứ tự dòng đáng tin hơn tọa độ tuyệt đối.
    Gặp nhãn khác thì dừng, vì trường này không có giá trị.
    """
    start = next(i for i, ln in enumerate(lines) if ln.key == label_line.key)
    center = (label_line.top + label_line.bottom) / 2
    for ln in lines[start + 1:]:
        if ln.key in label_keys or (ln.top + ln.bottom) / 2 - center > MAX_VALUE_GAP:
            return []
        # Cột giá trị bắt đầu ở mép trái của nhãn; chữ lệch trái hơn là phần khác của thẻ
        # (ô ảnh chân dung chẳng hạn).
        cols = [w for w in ln.words if w.left >= label_line.left - 12]
        if cols:
            return cols
    return []


def scan_for_number(words: list[Word], kind: str) -> Word | None:
    """Phương án dự phòng khi mất nhãn: quét cả trang tìm MỘT từ có dạng số giấy tờ.

    Chỉ dùng cho số 12 chữ số (hình dạng đủ riêng); ngày thì không, vì ngày sinh và ngày hết
    hạn trông giống nhau. Yêu cầu ít nhất 8 chữ số thật để từ như 'SOSOSOSOSOSO' không lọt.
    """
    pattern = ID_RE if kind == "id" else LICENSE_RE
    for w in words:
        compact = re.sub(r"[\s.\-]", "", normalize_text(w.text))
        fixed, _ = fix_digits(compact)
        if pattern.fullmatch(fixed) and sum(ch.isdigit() for ch in compact) >= 8:
            return w
    return None


def parse_card(words: list[Word], *, doc_type: str | None = None, today: date | None = None) -> CardResult:
    """Từ danh sách từ OCR của một ảnh giấy tờ đến CardResult. Không nâng ngoại lệ vì OCR xấu:
    trường không đọc được có value None và ghi lý do trong issues.

    `doc_type` là loại giấy tờ người dùng chọn lúc tải lên (ứng dụng thật hỏi sẵn); bỏ trống thì
    đoán theo loại có tổng điểm nhãn cao hơn.
    """
    lines = group_lines(words)
    candidates = {doc_type: SPECS[doc_type]} if doc_type else SPECS
    best_type, best_found, best_total = next(iter(candidates)), {}, -1.0
    for name, spec in candidates.items():
        found = find_labels(lines, spec)
        total = sum(score for _, score in found.values())
        if total > best_total:
            best_type, best_found, best_total = name, found, total
    spec = SPECS[best_type]
    label_keys = {ln.key for ln, _ in best_found.values()}
    fields: dict[str, FieldResult] = {}
    for name, _, kind in spec:
        picked: list[Word] = []
        scanned = False
        if name in best_found:
            picked = value_words(best_found[name][0], label_keys, lines)
        if not picked and kind in ("id", "license"):
            found_word = scan_for_number(words, kind)
            picked, scanned = ([found_word] if found_word else []), True
        if not picked:
            reason = "value_not_found" if name in best_found else "label_not_found"
            fields[name] = FieldResult(name, None, None, 0.0, issues=(reason,))
            continue
        raw = " ".join(w.text for w in picked)
        value, notes, issues = PARSERS[kind](normalize_text(raw))
        if scanned:
            notes = [*notes, "scan_without_label"]
        confs = tuple(w.conf for w in picked)
        fields[name] = FieldResult(name, raw, value, min(confs), tuple(notes), tuple(issues), confs)
    if best_type == "cccd":
        fields = _cross_check_id(fields)
    fields = _cross_check_dates(fields, today)
    return CardResult(best_type, fields)


def _cross_check_id(fields: dict[str, FieldResult]) -> dict[str, FieldResult]:
    id_field = fields["id_number"]
    if id_field.value is None:
        return fields
    extra = check_id_structure(id_field.value, fields["date_of_birth"].value, fields["sex"].value)
    if extra:
        fields = {**fields, "id_number": replace(id_field, issues=(*id_field.issues, *extra))}
    return fields


def _cross_check_dates(fields: dict[str, FieldResult], today: date | None) -> dict[str, FieldResult]:
    """Ngày sinh không ở tương lai; giấy phép phải hết hạn SAU ngày sinh."""
    if today is None:
        return fields
    dob_field = fields["date_of_birth"]
    dob = _to_date(dob_field.value)
    if dob is not None and dob > today:
        fields = {**fields, "date_of_birth": replace(dob_field, issues=(*dob_field.issues, "birth_in_future"))}
    if "expiry_date" in fields:
        exp_field = fields["expiry_date"]
        expiry = _to_date(exp_field.value)
        if dob is not None and expiry is not None and expiry <= dob:
            fields = {**fields, "expiry_date": replace(exp_field, issues=(*exp_field.issues, "expiry_before_birth"))}
    return fields


def _to_date(value: str | None) -> date | None:
    match = DATE_RE.fullmatch(value or "")
    if match is None:
        return None
    try:
        return date(int(match["year"]), int(match["month"]), int(match["day"]))
    except ValueError:
        return None
