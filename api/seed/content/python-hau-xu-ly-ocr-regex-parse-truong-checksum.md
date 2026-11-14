---
slug: python-hau-xu-ly-ocr-regex-parse-truong-checksum
title: "Hậu xử lý OCR: regex, tách trường CCCD/GPLX, checksum và fuzzy matching"
summary: "Bài 44: biến chữ OCR thô thành trường có kiểu: chuẩn hoá Unicode, regex có tên, sửa ký tự theo ngữ cảnh, kiểm cấu trúc số, rapidfuzz, confidence là min các từ. Có output thật và lab chạy được."
category: python-advanced
level: advanced
tags: [python, toi-noi, ocr, regex, rapidfuzz]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) chụp căn cước, ảnh đi vào `kyc-ocr`, và Tesseract trả về những thứ như `'0990901 59083'` cho số thẻ, `'43/03/1 990'` cho ngày sinh, `'can Tho'` cho quê quán. Đó là chữ, chưa phải dữ liệu: các service sau cần `id_number` đúng 12 chữ số, `date_of_birth` là ngày có thật, thành phố nằm trong danh sách. Quãng đường từ chữ đến dữ liệu là **hậu xử lý**, và nó sai được theo hai hướng: từ chối oan vì một dấu cách thừa, hoặc để lọt số sai vì «sửa» quá tay. Bài này dựng hậu xử lý cho thẻ căn cước và giấy phép lái xe giả, rồi đo từng bước thực sự làm được gì trên 64 ảnh.

Đây là bài 44 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần biết chạy Tesseract (bài 43). Ảnh nghiêng được xử lý ở [bài 42](/articles/python-opencv-phat-hien-canh-perspective-deskew), nhưng lab này cố tình đưa ảnh nghiêng vào thẳng để thấy hậu xử lý chịu được đến đâu. Mọi thẻ trong bài là **dữ liệu giả** sinh bằng Pillow: số `099...`, GPLX `99...`, tên mẫu, ô «ảnh» chỉ là hình chữ nhật xám.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Trên 64 ảnh thẻ giả (sạch, mờ, nghiêng, tối), loại sửa lỗi nào cứu được nhiều trường sai nhất: (a) đổi chữ <code>O</code> thành số <code>0</code> trong trường số, (b) bỏ dấu cách OCR chèn giữa dãy số hoặc ngày, (c) so khớp mờ tên thành phố với danh sách?</p>
<details><summary>Xem đáp án</summary><p><b>(b).</b> Trong 14 trường mà chữ OCR thô sai còn bản hậu xử lý đúng, 9 trường chỉ cần bỏ dấu cách chèn thừa, 3 trường nhờ so khớp mờ thành phố, 2 trường nhờ đổi <code>l</code> thành <code>1</code> ở hạng bằng. Bảng sửa chưa lần nào phải đổi <code>O</code> thành <code>0</code> trên bộ này, dù ta vẫn nên có nó.</p></details>
</div>

## Từ chữ OCR đến trường có kiểu

Với đuôi `tsv`, Tesseract trả từng từ kèm khung chữ nhật và độ tin cậy (`conf`, từ 0 đến 100). Lab gọi lệnh sau, và `ocr.py` đọc mỗi từ thành `Word(text, conf, left, top, width, height, line)`. Output thật của thẻ seed 7 sạch (trích):

```bash
tesseract stdin stdout -l vie+eng --psm 6 tsv
```

```text
[0] Từ Tesseract nhận ra (cccd-7-clean, trích): dòng (block, par, line), khung, conf, chữ
  dòng        left   top  rộng  cao       conf  chữ
  (1, 1, 5)    331   208   238   26  96.458328  099090159083
  (1, 1, 10)   333   351   147   22  96.554115  13/03/1990
  (1, 1, 12)   332   419    60   22  96.378326  Nam
```

Trong `Word`, `conf` đã đổi về khoảng 0 đến 1; `postprocess.py` đi qua năm bước (hình 1). Thẻ in nhãn ở dòng trên, giá trị ở dòng dưới, nên ta tìm **nhãn** trước (so khớp mờ đầu dòng với «Số / No.», «Ngày sinh»...) rồi lấy dòng kế tiếp làm giá trị.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 270" role="img" aria-label="Năm bước nối tiếp: từ Tesseract, tìm nhãn, chuẩn hoá NFC, sửa và regex, kiểm tra; kết quả là FieldResult, còn khi không chắc thì ghi cờ lý do thay vì đoán."><defs><marker id="py44-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py44-0-arrow-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="20" y="30" width="148" height="96" rx="10" class="f-phys-soft s-phys"/><text x="32" y="62" class="t-b">1. Tesseract</text><text x="32" y="88" class="t-small">từ + khung + conf</text><text x="32" y="106" class="t-small">dạng TSV</text><path d="M168 78 L193 78" class="f-none s-phys" stroke-width="2" marker-end="url(#py44-0-arrow)"/><rect x="193" y="30" width="148" height="96" rx="10" class="f-pk-soft s-pk"/><text x="205" y="62" class="t-b">2. Tìm nhãn</text><text x="205" y="88" class="t-small">so khớp mờ đầu dòng</text><text x="205" y="106" class="t-small">lấy dòng giá trị</text><path d="M341 78 L366 78" class="f-none s-phys" stroke-width="2" marker-end="url(#py44-0-arrow)"/><rect x="366" y="30" width="148" height="96" rx="10" class="f-pk-soft s-pk"/><text x="378" y="62" class="t-b">3. Chuẩn hoá</text><text x="378" y="88" class="t-small">NFC, gộp ký tự lạ</text><text x="378" y="106" class="t-small">bỏ dấu cách thừa</text><path d="M514 78 L539 78" class="f-none s-phys" stroke-width="2" marker-end="url(#py44-0-arrow)"/><rect x="539" y="30" width="148" height="96" rx="10" class="f-pk-soft s-pk"/><text x="551" y="62" class="t-b">4. Sửa + regex</text><text x="551" y="88" class="t-small">theo ngữ cảnh trường</text><text x="551" y="106" class="t-small">fullmatch, fuzzy</text><path d="M687 78 L712 78" class="f-none s-phys" stroke-width="2" marker-end="url(#py44-0-arrow)"/><rect x="712" y="30" width="148" height="96" rx="10" class="f-pk-soft s-pk"/><text x="724" y="62" class="t-b">5. Kiểm tra</text><text x="724" y="88" class="t-small">ngày có thật không</text><text x="724" y="106" class="t-small">cấu trúc số khớp thẻ</text><path d="M786 126 L786 168" class="f-none s-phys" stroke-width="2" marker-end="url(#py44-0-arrow)"/><path d="M440 126 L440 168" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py44-0-arrow-warn)"/><rect x="20" y="170" width="440" height="80" rx="10" class="f-warn-soft s-warn dash"/><text x="36" y="200" class="t-b">Không chắc: ghi issues, không đoán</text><text x="36" y="224" class="t-small">không dùng được thì value = None (label_not_found...)</text><text x="36" y="240" class="t-small">ngày không có thật: giữ chữ, cờ impossible_date</text><rect x="500" y="170" width="360" height="80" rx="10" class="f-phys-soft s-phys"/><text x="516" y="200" class="t-b">FieldResult</text><text x="516" y="224" class="t-small">raw, value, corrections, issues</text><text x="516" y="240" class="t-small">confidence = min(conf từng từ)</text></svg>
</div>
<figcaption>Hình 1. Luồng hậu xử lý. Mỗi bước ghi lại điều nó đã sửa (<code>corrections</code>) hoặc nghi ngờ (<code>issues</code>); bước nào không chắc thì ghi cờ (và để trống giá trị khi không dùng được) thay vì đoán.</figcaption>
</figure>

Kết quả mỗi trường là một giá trị bất biến (`postprocess.py`, trích):

```python
@dataclass(frozen=True)
class FieldResult:
    name: str
    raw: str | None  # các từ OCR nối lại, chưa xử lý gì
    value: str | None  # giá trị đã chuẩn hóa; None nếu không dùng được
    confidence: float  # NHỎ NHẤT trong confidence của các từ tạo nên trường (0..1)
    corrections: tuple[str, ...] = ()
    issues: tuple[str, ...] = ()
    word_confs: tuple[float, ...] = ()
```

Khung chữ của ảnh mờ hay nghiêng chồng lên nhau theo chiều dọc (đã gặp), nên `value_words` chọn dòng giá trị theo **thứ tự đọc** (dòng kế tiếp có chữ ở cột giá trị, gặp nhãn khác thì dừng) thay vì so `top` với `bottom`. Vệt chữ vô nghĩa từ ô ảnh chân dung bị loại bằng chiều cao (dưới 8 điểm ảnh) và vị trí (lệch trái hơn mép nhãn). Cả hai dựa trên **bố cục cố định** của thẻ giả.

Thẻ seed 7 bị nghiêng cho ra kết quả từ đầu đến cuối như sau (output thật, `confidence` làm tròn hai chữ số):

```text
[1] Một thẻ từ đầu đến cuối: cccd-7-rotate
  id_number      thô '0990901 59083'  -> '099090159083'   confidence 0.90  sửa=[] vấn đề=[]
  full_name      thô 'NGUYEN VAN A'   -> 'NGUYEN VAN A'   confidence 0.62  sửa=[] vấn đề=[]
  date_of_birth  thô '43/03/1 990'    -> '43/03/1990'     confidence 0.84  sửa=[] vấn đề=['impossible_date']
  sex            thô 'Nam'            -> 'Nam'            confidence 0.97  sửa=[] vấn đề=[]
  hometown_city  thô 'can Tho'        -> 'Cần Thơ'        confidence 0.96  sửa=['fuzzy:can Tho->Cần Thơ@100'] vấn đề=[]
```

Dấu cách trong số thẻ được bỏ, thành phố về dạng chuẩn có dấu, ngày `43/03/1990` bị cờ vì không có ngày 43, và tên có `confidence` thấp nhất thẻ (0,62).

## Chuẩn hoá Unicode trước khi so khớp

Cùng một chữ «Ễ» có hai cách mã hoá: một mã dựng sẵn (NFC) hoặc chữ `E` cộng hai dấu rời (NFD). Mắt người không thấy khác, `==` và regex thì có:

```python
def normalize_text(text: str) -> str:
    """NFC, gộp ký tự «giống nhau», gộp mọi khoảng trắng về một dấu cách."""
    text = unicodedata.normalize("NFC", text).translate(_CONFUSABLE)
    return " ".join(text.split())  # split() không đối số tách theo mọi khoảng trắng Unicode


def fold(text: str) -> str:
    """Bỏ dấu và hạ chữ thường, để so sánh «gần đúng»: 'Cần Thơ' và 'CAN THO' cho cùng kết quả."""
    decomposed = unicodedata.normalize("NFD", text.replace("Đ", "D").replace("đ", "d"))
    return "".join(ch for ch in decomposed if not unicodedata.combining(ch)).casefold()
```

`_CONFUSABLE` đổi nháy cong về nháy thẳng, các loại gạch ngang về `-`, khoảng trắng không ngắt về dấu cách và xoá ký tự rộng bằng 0. Tác động:

```text
[2] Unicode: cùng một chữ, hai dãy mã
  NFC: 6 mã, NFD: 8 mã, bằng nhau: False
  NAME_RE khớp NFC: True, khớp NFD: False
  normalize_text('  Nguyễn\xa0Văn\u200b  A  ') -> 'Nguyễn Văn A'
  fold('Đà Nẵng') -> 'da nang'
```

Regex tên `[^\W\d_]+(?: [^\W\d_]+){1,5}` (hai đến sáu từ toàn chữ cái) khớp chuỗi NFC nhưng trượt chuỗi NFD, vì dấu rời không phải chữ cái. Không có lỗi nào được ném, chỉ có tên «không hợp lệ» khó hiểu. Quy tắc: `normalize_text` ngay sau khi nhận chữ, trước mọi so sánh hay regex.

`fold` phục vụ một thực tế: trên bộ 64 ảnh, chỉ 6 tên được đọc **đúng nguyên văn có dấu**; phần còn lại mất dấu (`NGUYEN VAN A` thay vì `NGUYỄN VĂN A`). Không có danh sách để tra thì không «khôi phục» được dấu của tên người, nên lab so tên **không dấu** và giữ chữ OCR cho người duyệt. Thành phố thì khác: danh sách đóng cho phép trả về dạng chuẩn có dấu (mục fuzzy).

## Regex có tên và sửa ký tự theo ngữ cảnh

Mỗi kiểu trường có một regex, dùng với `fullmatch` (không phải `search`: chuỗi thừa một ký tự cũng phải là lỗi) và nhóm có tên để lấy từng phần:

```python
ID_RE = re.compile(r"(?P<province>\d{3})(?P<century>\d)(?P<year2>\d{2})(?P<serial>\d{6})")
LICENSE_RE = re.compile(r"(?P<prefix>99)(?P<serial>\d{10})")
DATE_RE = re.compile(r"(?P<day>\d{2})/(?P<month>\d{2})/(?P<year>\d{4})")
```

Regex chỉ kiểm **hình dạng**. `48/10/1985` đúng hình `dd/mm/yyyy` nhưng không tồn tại, nên `parse_date` còn gọi `datetime.date(...)` và bắt `ValueError`: hình dạng cho biết ta tách đúng chuỗi, `date` cho biết chuỗi có nghĩa.

Sửa ký tự dễ nhầm là phần dễ hỏng nhất: `O` hay `0`, `l` hay `1` chỉ phân biệt được **khi biết trường chứa gì**. Ta có hai bảng ngược nhau, mỗi hàm `parse_*` chỉ dùng bảng của trường mình:

```python
TO_DIGIT = str.maketrans({"O": "0", "o": "0", "Q": "0", "I": "1", "l": "1", "|": "1", "!": "1",
                          "Z": "2", "S": "5", "G": "6", "B": "8"})
TO_LETTER = str.maketrans({"0": "O", "1": "I", "5": "S", "8": "B"})
```

`fix_digits` dùng cho số thẻ, số GPLX, ngày và hạng bằng; `fix_letters` dùng cho tên và chỉ đổi chữ số có chữ cái ở **cả hai bên**. Mỗi lần đổi ghi vào `corrections` (`'O->0@0'`: đổi `O` ở vị trí 0) để kiểm toán. Kết quả trên chuỗi dựng tay:

```text
[3] Sửa ký tự dễ nhầm theo ngữ cảnh trường
  id_number      'O99O9OI59O83'     -> '099090159083'   sửa=['O->0@0', 'O->0@3', 'O->0@5', 'I->1@6', 'O->0@9'] vấn đề=[]
  id_number      '0990 9015 9083'   -> '099090159083'   sửa=[] vấn đề=[]
  date_of_birth  'l3-O3-199O'       -> '13/03/1990'     sửa=['l->1@0', 'O->0@3', 'O->0@9'] vấn đề=[]
  date_of_birth  '13 / 03 / 1 990'  -> '13/03/1990'     sửa=[] vấn đề=[]
  date_of_birth  '48/10/1985'       -> '48/10/1985'     sửa=[] vấn đề=['impossible_date']
  full_name      'NGUY3N V4N A'     -> None             sửa=[] vấn đề=['digit_in_name']
  full_name      'NGUYEN VAN5'      -> None             sửa=[] vấn đề=['digit_in_name']
  full_name      'NGUY0N V1N A'     -> 'NGUYON VIN A'   sửa=['0->O@4', '1->I@8'] vấn đề=[]
  license_class  'Al'               -> 'A1'             sửa=['l->1@0'] vấn đề=[]
  license_class  'Cc'               -> None             sửa=[] vấn đề=['license_class_unknown']
```

Ba quyết định. Một: chữ số ở đầu hay cuối từ không bị đổi (`VAN5` không thành `VANS`, vì có thể là chữ số thật hoặc vệt nhiễu), và tên còn chữ số sau khi sửa thì **từ chối** (`digit_in_name`) thay vì xoá chữ số cho khớp regex; xoá lặng lẽ biến `NGUY3N` thành `NGUYN`, một tên hợp lệ nhưng sai. Hai: `'Cc'` không đoán thành `'C'`. Ba: `Al` thành `A1` vì vị trí thứ hai của hạng bằng chỉ có thể là chữ số; ca này có thật trong hai ảnh của bộ, không chỉ là chuỗi dựng tay.

Tóm lại, trên bộ 64 ảnh bảng sửa chưa lần nào phải đổi `O` thành `0`. Cái cứu được nhiều nhất là bỏ dấu cách chèn thừa: `re.sub(r"[\s.\-]", "", text)` ở số thẻ và số GPLX (6 trường), và `re.sub(r"\s+", "", text)` ở đầu `parse_date` (3 ngày). Đo trước, rồi mới thêm luật.

## Kiểm cấu trúc, và chuyện checksum

Số căn cước giả của loạt bài có 12 chữ số theo quy ước dữ liệu giả: mã tỉnh `099` (không tồn tại), 1 chữ số giới tính cộng thế kỷ sinh (thế kỷ 20: nam 0, nữ 1; thế kỷ 21: nam 2, nữ 3), 2 chữ số cuối của năm sinh, 6 chữ số ngẫu nhiên. Hai nhóm (thế kỷ cộng giới tính, và năm sinh) **trùng với thông tin in ở chỗ khác trên thẻ** nên đối chiếu được; mã tỉnh thì chỉ so với một danh sách cho phép. Hàm dưới đây không tính chữ số kiểm tra nào, nó kiểm sự nhất quán:

```python
def check_id_structure(id_number: str, birth_date: str | None, sex: str | None) -> list[str]:
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
```

(`ACCEPTED_PREFIXES` chỉ chứa `099`; hệ thống thật phải nạp bảng mã tỉnh từ văn bản chính thức của cơ quan cấp.) Thử trên một số hợp lệ, đổi từng thông tin, rồi đo năng lực phát hiện bằng cách thay **mỗi** chữ số bằng 9 chữ số khác (12 vị trí × 9 = 108 phép sai):

```text
[4] Kiểm cấu trúc số CCCD giả (không phải checksum)
  099085791907, nam, 18/10/1985 -> []
  099085791907, nữ,  18/10/1985 -> ['century_digit_mismatch:0']
  099085791907, nam, 18/10/1995 -> ['year_digits_mismatch:85']
  099085791907, nam, 18/10/2005 -> ['century_digit_mismatch:0', 'year_digits_mismatch:85']
  sai một chữ số: bắt được 54/108; theo vị trí 1..12 = [9, 9, 9, 9, 9, 9, 0, 0, 0, 0, 0, 0]
  trong 54: 27 do mã tỉnh nằm ngoài danh sách cho phép ['099'], 27 do đối chiếu giới tính và năm sinh (giả định hai thông tin này đọc đúng)
  Luhn (số thẻ thanh toán thử): 4111111111111111 -> True, 4111111111111112 -> False
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Số 099085791907 chia bốn nhóm: mã tỉnh, thế kỷ và giới tính, năm sinh, sáu số ngẫu nhiên. Mã tỉnh so với danh sách cho phép, hai nhóm kế đối chiếu với thẻ, nên ba nhóm đầu bắt được mọi phép sai một chữ số; sáu số cuối thì không."><rect x="140" y="30" width="150" height="56" class="f-pk-soft s-pk"/><text x="215" y="66" text-anchor="middle" class="t-big">099</text><rect x="290" y="30" width="50" height="56" class="f-phys-soft s-phys"/><text x="315" y="66" text-anchor="middle" class="t-big">0</text><rect x="340" y="30" width="100" height="56" class="f-phys-soft s-phys"/><text x="390" y="66" text-anchor="middle" class="t-big">85</text><rect x="440" y="30" width="300" height="56" class="f-warn-soft s-warn"/><text x="590" y="66" text-anchor="middle" class="t-big">791907</text><path d="M446 86 L446 118" class="f-none s-muted dash"/><text x="454" y="130" class="t-small">6 số ngẫu nhiên: đổi một chữ số bắt được 0/9, không có gì để tính lại</text><path d="M346 86 L346 146" class="f-none s-muted dash"/><text x="354" y="158" class="t-small">2 số cuối năm sinh: đối chiếu với thẻ, bắt 9/9</text><path d="M296 86 L296 174" class="f-none s-muted dash"/><text x="304" y="186" class="t-small">1 số thế kỷ và giới tính: đối chiếu với thẻ, bắt 9/9</text><path d="M146 86 L146 202" class="f-none s-muted dash"/><text x="154" y="214" class="t-small">mã tỉnh: so với danh sách cho phép (chỉ 099), bắt 9/9 mỗi vị trí</text></svg>
</div>
<figcaption>Hình 2. Kiểm cấu trúc bắt được 54 trên 108 phép sai một chữ số: 27 nhờ danh sách mã tỉnh, 27 nhờ đối chiếu với thẻ, không phép nào ở sáu chữ số cuối.</figcaption>
</figure>

Con số 54 chia đôi: 27 phép bắt nhờ hằng `ACCEPTED_PREFIXES = {099}` (chỉ đúng cho dữ liệu giả), 27 nhờ đối chiếu, và 27 này giả định ngày sinh với giới tính đã đọc đúng. Nói thẳng về checksum. **Ta không biết nguồn công khai nào mô tả chữ số kiểm tra của số CCCD**, nên lab không bịa thuật toán và không gọi phép kiểm trên là «checksum». Nó chỉ bắt được sai ở nửa đầu của số, và chỉ khi thông tin đối chiếu đọc đúng (hình 2). Số có checksum thật thì khác: Luhn (ISO/IEC 7812) của số thẻ thanh toán bắt mọi sai sót một chữ số, như ví dụ cuối của output. Chỉ dùng checksum khi **tổ chức phát hành công bố thuật toán**; còn không, phần số không kiểm được phải dựa vào `confidence` và người duyệt. Với GPLX giả, lab chỉ kiểm dạng `99` cộng 10 chữ số và không khẳng định cấu trúc số giấy phép thật.

## Fuzzy matching với danh sách đóng

Giới tính, hạng bằng và (ở lab này) quê quán chỉ nhận giá trị trong danh sách cho trước; quê quán là 6 thành phố của Tới Nơi. Ta so khớp mờ bằng `rapidfuzz` trên chuỗi đã `fold`, và chỉ nhận ứng viên đầu khi nó **đủ cao và hơn hạng nhì đủ xa**.

```python
FUZZY_CITY_MIN = 80.0  # điểm tối thiểu của ứng viên tốt nhất
FUZZY_CITY_MARGIN = 10.0  # ứng viên tốt nhất phải hơn ứng viên thứ hai ít nhất từng này điểm


def match_city(text: str) -> tuple[str | None, float, float]:
    ranked = process.extract(fold(text), {city: fold(city) for city in CITIES}, scorer=fuzz.ratio, limit=2)
    (_, best_score, best_city), (_, second_score, _) = ranked
    if best_score >= FUZZY_CITY_MIN and best_score - second_score >= FUZZY_CITY_MARGIN:
        return best_city, best_score, second_score
    return None, best_score, second_score
```

Giá trị trả về là khoá của từ điển, tức dạng chuẩn có dấu, nên so khớp mờ vừa sửa chính tả vừa khôi phục dấu. Output thật:

```text
[5] rapidfuzz với danh sách đóng (tối thiểu 80 điểm, hơn hạng nhì 10 điểm)
  'CAN THO'      -> 'Cần Thơ'    điểm 100.0, hạng nhì  50.0
  'Da Nang'      -> 'Đà Nẵng'    điểm 100.0, hạng nhì  62.5
  'TP HCM'       -> 'TP.HCM'     điểm  83.3, hạng nhì  30.8
  'Nha Trag'     -> 'Nha Trang'  điểm  94.1, hạng nhì  53.3
  'Ha Nam'       -> None         điểm  66.7, hạng nhì  61.5
  'Ho Chi Minh'  -> None         điểm  40.0, hạng nhì  35.3
  không có ngưỡng: 'Ha Nam' -> 'Hà Nội' (66.7 điểm)
  tên OCR 'LE VAN CUONG' so với hồ sơ 'Lê Văn Cường': 100.0
  tên OCR 'LE VAN C' so với hồ sơ 'Lê Văn Cường': 80.0
  tên OCR 'TRAN THI B' so với hồ sơ 'Lê Văn Cường': 27.3
```

Hai dòng đáng đọc kỹ. `Ha Nam` là một địa danh nhưng không thuộc 6 thành phố; chọn ứng viên cao nhất mà không có ngưỡng thì nó thành `Hà Nội` (66,7 điểm). Ngưỡng 80 và biên 10 điểm từ chối nó. Ngược lại `Ho Chi Minh` bị từ chối dù người đọc hiểu ngay, vì bảng không có tên gọi khác: **so khớp mờ sửa chính tả, không sửa được thiếu từ điển**.

`rapidfuzz` còn làm hai việc. Một là tìm nhãn: `label_score` so đầu dòng với nhãn đã bỏ dấu nên `S6/No.:` vẫn nhận ra «Số / No.:», `gay sinh:` vẫn nhận ra «Ngày sinh:». Ta dùng `fuzz.ratio` trên đoạn đầu dòng, không dùng `partial_ratio`: nó căn chuỗi ngắn vào chuỗi dài nên dòng chỉ có chữ `D` được 100 điểm với nhãn «Có giá trị đến:». Hai là so tên OCR với tên đã nhập trong hồ sơ (ba dòng cuối của output), vẫn chỉ là đầu vào cho người duyệt.

## Confidence của trường là nhỏ nhất của các từ

Mỗi từ có `conf` riêng. Lab đặt `confidence` của trường bằng `min(w.conf for w in picked)`: trường chỉ đáng tin bằng mắt xích yếu nhất của nó. Ca chênh lớn nhất trong bộ:

```text
[7] Confidence của trường = nhỏ nhất của các từ
  cccd-1-dark full_name: 'a LE VAN C'
    confidence từng từ [0.0, 0.96, 0.93, 0.91]
    min 0.00, trung bình 0.70
```

Từ `a` đứng đầu là vệt nhiễu lọt vào cột giá trị; ba từ còn lại đều trên 0,9. Trung bình cho 0,70, trông gần chấp nhận được; `min` cho 0,00 và đưa đúng trường này vào hàng chờ. Đây là tín hiệu của Tesseract, không phải xác suất đúng; đo xem nó tốt đến đâu là việc của bài 45.

## Chạy và kiểm chứng

Cần Tesseract kèm gói `vie` (`brew install tesseract tesseract-lang`); lần đầu `uv` cần mạng để tải `numpy`, `pillow` và `rapidfuzz`. Ảnh và cache OCR nằm trong `out/` (gitignore), không commit ảnh nào.

```bash
uv run --project examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum/main.py
uv run --project examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum examples/toi-noi-python/44-hau-xu-ly-ocr-regex-checksum/main.py --verify
```

Bộ ảnh này khác bộ của bài 41 và 42 (GPLX cùng seed là người khác, tham số làm mờ, nghiêng, tối khác), nên số liệu chỉ so được trong cùng một bài. Lab dùng `-l vie+eng --psm 6` (cấu hình bài 43) trên thẻ phẳng chưa qua bài 42. Đo riêng ngoài lab trên đúng 64 ảnh này, số trường đúng trên 320 là 257 (`vie --psm 4`), 277 (`vie --psm 6`), 251 (`vie+eng --psm 4`), 266 (`vie+eng --psm 6`); bài không đổi cấu hình cho số khớp bài khác. Môi trường: MacBook Pro M1 Max, macOS 26.6.2, Python 3.14.7, Tesseract 5.5.4, `rapidfuzz` 3.14.6, `pillow` 13.0.0 (bài không dùng số đo thời gian). Phần cuối của `main.py` chạy cả bộ: 8 seed × 2 loại giấy tờ × 4 biến thể (sạch, mờ, nghiêng 3 đến 6 độ, tối kèm nhiễu) = 64 ảnh, 320 trường; đáp án là chính các chuỗi hàm sinh thẻ đã vẽ. Output (trích):

```text
[6] Cả bộ: 64 ảnh = 8 seed x 2 loại giấy tờ x 4 biến thể
  trường đúng (tên xét không dấu): chỉ nối từ OCR 252/320, sau hậu xử lý 266/320
  theo biến thể (chỉ nối từ -> hậu xử lý, trên tổng):
    clean    78 ->  78 / 80
    blur     59 ->  62 / 80
    rotate   48 ->  58 / 80
    dark     67 ->  68 / 80
  tên đúng NGUYÊN VĂN (có dấu): 6/64
  trường được hậu xử lý cứu (thô sai, sau đúng): 14 = {'bỏ dấu cách chèn thừa': 9, 'fuzzy với danh sách đóng': 3, 'đổi ký tự dễ nhầm': 2}; trường bị hậu xử lý làm hỏng: 0
  quét cả trang tìm số 12 chữ số khi mất nhãn: 13 lần, đúng 12 lần
  phân loại trường: đúng không cờ 259, đúng có cờ 7, sai có cờ/thiếu 45, sai KHÔNG cờ 9
  sai không cờ theo trường: {'full_name': 9}
```

Cột «chỉ nối từ OCR» vẫn dùng bước tìm nhãn và dòng giá trị, chỉ bỏ chuẩn hoá, sửa và kiểm tra. Bước quét cả trang tìm số 12 chữ số khi mất nhãn (`scan_for_number`, dòng cuối của khối trên) nằm ở cả hai cột nên không hiện ở bảng. Ảnh sạch gần như không cần hậu xử lý (hai trường sai đều là hạng bằng `Cc`, bị từ chối), ảnh nghiêng hưởng nhiều nhất (48 lên 58), và hậu xử lý không làm hỏng trường nào đã đúng. **Lỗi nguy hiểm nhất là lỗi không bị từ chối**: 45 trường sai đều có cờ hoặc bị bỏ trống, nhưng 9 trường sai không có cờ nào, toàn là tên:

```text
    cccd-1-dark    full_name  đúng 'LÊ VĂN C'     đọc 'A LE VAN C'     confidence 0.00
    cccd-2-dark    full_name  đúng 'TRẦN THỊ B'   đọc 'O TRAN THIB'    confidence 0.00
    cccd-3-rotate  full_name  đúng 'LÊ VĂN C'     đọc 'SẼ LỆ NL'       confidence 0.00
    gplx-1-blur    full_name  đúng 'TRẦN THỊ B'   đọc 'TRAN THIB'      confidence 0.89
    gplx-1-rotate  full_name  đúng 'TRẦN THỊ B'   đọc 'TRAN TH IB'     confidence 0.89
    gplx-2-dark    full_name  đúng 'TRẦN THỊ B'   đọc 'O TRAN THIB'    confidence 0.00
    gplx-4-rotate  full_name  đúng 'VŨ THỊ F'     đọc 'VŨ THIF'        confidence 0.91
    gplx-5-dark    full_name  đúng 'TRẦN THỊ B'   đọc 'TRAN THIB'      confidence 0.88
    gplx-8-dark    full_name  đúng 'LÊ VĂN C'     đọc 'TE EY'          confidence 0.12
  trong số đó confidence < 0,80: 5/9
```

Năm ca `confidence` dưới 0,80 sẽ bị chặn bởi ngưỡng ở bài sau. Bốn ca còn lại (`TRAN THIB`, `TRAN TH IB`, `VŨ THIF`) vẫn là tên hợp lệ về hình dạng với `confidence` 0,88 đến 0,91; regex không phân biệt được `THIB` với một tên thật. Bài 45 đo xem ngưỡng `confidence` chặn được bao nhiêu và lọt bao nhiêu. Kết thúc kiểm chứng:

```text
OK 40/40
```

`--verify` kiểm 40 điều: các ca Unicode, mười ca sửa theo ngữ cảnh (kể cả `VAN5` phải ra `digit_in_name`), cấu trúc số (đúng 54/108, chia 27 và 27), Luhn, các ca fuzzy, và các bất biến của cả bộ ảnh (hậu xử lý không làm tụt hay làm hỏng trường nào, số trường theo phân loại cộng đủ 320). Nó không kiểm số tuyệt đối của OCR (252, 266...), vì các số đó phụ thuộc phiên bản Tesseract, gói ngôn ngữ và font hệ thống.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `fullmatch` trượt dù chuỗi trông đúng | In `repr(text)`: có thể là NFD hoặc ký tự rộng bằng 0; gọi `normalize_text` |
| Tên đúng nhưng mất dấu | So bằng `fold`, giữ chữ OCR cho người duyệt; không «khôi phục» dấu khi không có danh sách |
| Thành phố bị gán nhầm | In điểm và điểm hạng nhì; xem ngưỡng, biên và độ đầy đủ của danh sách |
| Số đủ 12 chữ số mà vẫn sai | Kiểm cấu trúc không bắt được sáu số cuối; xem `word_confs`, chuyển người duyệt |
| Trường trống dù ảnh đọc được | `label_not_found` hay `value_not_found`: in các dòng thô, xem nhãn bị méo hoặc cột giá trị lệch |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Thẻ là ảnh vẽ bằng code với nhãn và giá trị cố định, nên «tìm nhãn rồi lấy dòng kế tiếp» đủ dùng; thẻ thật có chip, mã QR, hai mặt và bố cục đổi theo đợt phát hành. Danh sách 6 thành phố nhỏ hơn nhiều so với quê quán thật; mã tỉnh <code>099</code> và «GPLX bắt đầu 99» là quy ước dữ liệu giả. Các số 252, 266, 14, 9 là của 64 thẻ giả này với Tesseract 5.5.4, đừng suy ra tỉ lệ cho ảnh chụp thật. Kết quả chỉ là đầu vào cho người duyệt, không phải quyết định xác minh giấy tờ; số giấy tờ là dữ liệu cá nhân, không in nguyên văn vào log (bài 47 nói về mặt nạ PII).</p>
</div>

**Bài tập 1:** Hạ `FUZZY_CITY_MIN` xuống 60 và giữ biên 10 điểm. `match_city("Ha Nam")` trả gì? Sau đó đặt biên bằng 0 thì sao? Kết luận gì về hai tham số?

**Bài tập 2:** `TRAN THIB` với `confidence` 0,88 lọt qua mọi kiểm tra. Hồ sơ tài xế đã có tên người nhập. Đề xuất một kiểm tra rẻ dựa trên số từ, và nói nó bỏ sót ca nào trong danh sách chín tên sai ở trên.

<details><summary>Xem lời giải</summary><p>(1) Với min 60 và biên 10: <code>Ha Nam</code> được 66,7 điểm, hạng nhì (<code>Đà Nẵng</code>) 61,5, chênh 5,2 &lt; 10 nên <b>vẫn bị từ chối</b>. Đặt biên bằng 0 thì nhận <code>Hà Nội</code>, tức gán nhầm: biên chặn hai ứng viên sát nhau, ngưỡng chặn ứng viên còn quá xa. (2) Lệch số từ giữa tên OCR và tên đã nhập (sau <code>fold</code>, tách theo dấu cách) thì đưa vào hàng chờ: <code>TRAN THIB</code> và <code>VŨ THIF</code> có hai từ, tên thật ba từ. Cách này bắt ba trong bốn tên có <code>confidence</code> cao (gplx-1-blur, gplx-4-rotate, gplx-5-dark), <b>bỏ sót</b> <code>TRAN TH IB</code> vì chữ tách sai mà vẫn ba từ, và cần hồ sơ có sẵn tên nhập tay.</p></details>

Bài trước: [bài 43](/articles/python-tesseract-easyocr-paddleocr-so-sanh). Bài tiếp: [bài 45](/articles/python-danh-gia-ocr-cer-wer-threshold).

## Tài liệu gốc

- [Python: re, biểu thức chính quy](https://docs.python.org/3.14/library/re.html).
- [Python: unicodedata](https://docs.python.org/3.14/library/unicodedata.html) và [Unicode HOWTO](https://docs.python.org/3.14/howto/unicode.html).
- [Unicode Standard Annex 15: Normalization Forms](https://www.unicode.org/reports/tr15/).
- [RapidFuzz: fuzz](https://rapidfuzz.github.io/RapidFuzz/Usage/fuzz.html) và [process](https://rapidfuzz.github.io/RapidFuzz/Usage/process.html).
- [Tesseract: Command Line Usage (đầu ra TSV, psm)](https://tesseract-ocr.github.io/tessdoc/Command-Line-Usage.html).
