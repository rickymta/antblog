---
slug: python-face-embedding-cosine-threshold-far-frr
title: "Embedding và so khớp: cosine, ngưỡng, FAR/FRR và ROC"
summary: "Bài 49: embedding khuôn mặt bằng SFace (ONNX), cosine bằng numpy, ROC, FAR/FRR, EER tự tính trên Olivetti; báo riêng ảnh không lấy được mặt, chọn ngưỡng theo chi phí và kiểm trên nhiều cách chia người. Có số đo thật."
category: python-advanced
level: advanced
tags: [python, toi-noi, face-embedding, onnx, far-frr]
language: vi
---

Hồ sơ KYC của một tài xế [Tới Nơi](/articles/du-an-mau-toi-noi) đã qua bước chuẩn bị ở bài 48: ảnh có đúng một mặt, đã căn chỉnh về khung 112 × 112. Giờ `kyc-face` so ảnh thẻ với selfie và ra một con số, giả sử 0,52. Khớp hay không? Nghĩa của con số đến từ **ngưỡng**, và ngưỡng đến từ dữ liệu: bao nhiêu cặp cùng người bị từ chối, bao nhiêu cặp khác người bị chấp nhận nhầm, mỗi loại nhầm tốn bao nhiêu, và bao nhiêu ảnh không lấy được mặt. Bài này đi từ embedding, cosine, FAR/FRR, ROC đến chọn ngưỡng, rồi kiểm ngưỡng với người chưa từng thấy.

Đây là bài 49 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần bài 48 ([face detection và landmarks](/articles/python-face-detection-landmarks-mediapipe-opencv)) vì ta dùng lại bước căn chỉnh của nó.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Nhiều ví dụ trên mạng đặt ngưỡng cosine cố định ở 0,3 hoặc 0,5. Với SFace trên Olivetti sạch, FAR ở hai ngưỡng đó xấp xỉ bao nhiêu?</p>
<details><summary>Xem đáp án</summary><p>Ở 0,3 FAR là <b>27,85%</b>: hơn một phần tư cặp khác người bị chấp nhận. Ở 0,5 FAR là 0,61%, cũng gấp sáu lần mục tiêu 0,1%. Ngưỡng đo được ở FAR 0,1% là 0,567. Ngưỡng phải đo trên dữ liệu và riêng cho từng mô hình.</p></details>
</div>

## Từ ảnh mặt đến vector

**Embedding** là vector số mà mô hình gán cho một khuôn mặt sao cho mặt của cùng một người nằm gần nhau, mặt người khác nằm xa nhau. Mặc định lab `49-face-embedding-cosine-threshold` dùng **SFace** (OpenCV Zoo, 128 chiều, chạy offline qua ONNX bằng `cv2.FaceRecognizerSF`) và **YuNet** để dò mặt. Cờ `--with-arcface` thêm **ArcFace R50** của InsightFace (512 chiều, onnxruntime) làm hàng so sánh; vì sao nó không phải mặc định thì đoạn giấy phép bên dưới nói rõ.

```python
# embedders.py (trích)
class SFace:
    name = "sface"
    dim = 128

    def __init__(self) -> None:
        path = ensure_model(SFACE, verbose=False)
        self._recognizer = cv2.FaceRecognizerSF.create(str(path), "")

    def embed(self, aligned_bgr):
        vectors = [self._recognizer.feature(img)[0] for img in aligned_bgr]  # mỗi lần một ảnh
        return l2_normalize(np.asarray(vectors, dtype=np.float32))
```

Vector ra luôn được **chuẩn hoá L2** (độ dài 1), nên cosine similarity chỉ còn là tích vô hướng: `a @ b.T` cho cả ma trận cosine của hai tập. Với ArcFace, lab nạp thẳng file ONNX bằng onnxruntime (`CPUExecutionProvider` cố định để số đo lặp lại; GPU hay CoreML đổi qua `providers`, bài 52) thay vì dùng gói `insightface`, vì gói đó (bản 2.1) kéo thêm một bản OpenCV đè lên thư mục `cv2`. Lab 49 dùng `opencv-python-headless` 5.0.0 và ngưỡng điểm YuNet 0,6, khác lab 48 (`opencv-contrib-python` 4.14 và 0,7).

**Giấy phép cần đọc trước khi chạy.** Mã nguồn InsightFace theo MIT, nhưng README của họ ghi các mô hình huấn luyện sẵn, trong đó có gói `buffalo_l` (file `w600k_r50.onnx`), chỉ dùng cho nghiên cứu phi thương mại; muốn dùng khác phải liên hệ InsightFace để xin giấy phép. Repository này không chứa file mô hình: cờ `--with-arcface` tải trực tiếp từ trang phát hành của họ về máy bạn, và bài chỉ dùng nó để so thang điểm giữa hai mô hình. Nếu bạn chạy trong công ty hoặc dự án có thu phí, hãy hỏi người phụ trách pháp lý trước, hoặc chỉ chạy SFace (OpenCV Zoo ghi Apache 2.0; YuNet ghi MIT). Giấy phép của file mô hình không tự trả lời câu hỏi về dữ liệu đã dùng để huấn luyện. Đoạn này không phải tư vấn pháp lý.

Dữ liệu là Olivetti: 400 ảnh xám 64 × 64, 40 người, mỗi người 10 ảnh. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Lần đầu sklearn tải khoảng 4,26 MB và lưu bản cache 1,4 MB. Từ đó có 40 × C(10, 2) = **1.800 cặp cùng người** (genuine) và C(400, 2) − 1.800 = **78.000 cặp khác người** (impostor). Lab in URL và kích thước mô hình khi tải (mặc định chỉ hai dòng đầu):

```text
  yunet_2023mar.onnx          232,589 byte  https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx
  sface_2021dec.onnx       38,696,353 byte  https://github.com/opencv/opencv_zoo/raw/main/models/face_recognition_sface/face_recognition_sface_2021dec.onnx
  (với --with-arcface thêm w600k_r50.onnx 174,383,860 byte, trong gói buffalo_l.zip 288,621,354 byte từ github.com/deepinsight/insightface/releases)
```

Mỗi ảnh 64 × 64 được phóng lên 192 px, đệm viền lặp 64 px mỗi phía, YuNet dò 5 landmark rồi phép đồng dạng của bài 48 đưa chúng về mẫu ArcFace 112 × 112. Vì Olivetti đã cắt sát mặt, bước này chủ yếu đưa mặt về **đúng tỉ lệ của mẫu**: nếu chỉ resize 64 lên 112 (không dò, không landmark), hai mắt cách nhau 52,8 px trong khi mẫu chỉ 35,2 px (mặt to hơn mẫu 1,50 lần); còn crop căn chỉnh có trung vị 49,6% điểm ảnh nằm ngoài ảnh gốc, tức viền lặp mà mẫu chừa lề.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 322" role="img" aria-label="Hộp phân vị điểm cosine của cặp khác người và cặp cùng người trên Olivetti sạch cho hai mô hình, kèm ngưỡng ở FAR bằng một phần nghìn. Với ArcFace hai nhóm tách nhau quanh 0,31; với SFace nhóm khác người trải tới 0,64 nên ngưỡng là 0,567."><defs></defs><text x="10" y="25" class="t-b">ArcFace R50 (512 chiều)</text><text x="411.1" y="25" class="t-small t-warn">ngưỡng 0,311 (FAR 0,1%)</text><text x="10" y="65" class="t-small">impostor</text><path d="M104.2 60 L455.1 60" class="f-none s-muted" stroke-width="2"/><rect x="186.8" y="48" width="143.7" height="24" rx="4" class="f-pk-soft s-pk"/><path d="M254.0 48 L254.0 72" class="f-none s-pk" stroke-width="3"/><text x="10" y="110" class="t-small">genuine</text><path d="M421.2 105 L816.4 105" class="f-none s-muted" stroke-width="2"/><rect x="569.8" y="93" width="217.7" height="24" rx="4" class="f-phys-soft s-phys"/><path d="M709.1 93 L709.1 117" class="f-none s-phys" stroke-width="3"/><path d="M405.1 37 L405.1 127" class="f-none s-warn dash" stroke-width="2"/><text x="10" y="155" class="t-b">SFace (128 chiều)</text><text x="569.0" y="155" class="t-small t-warn">ngưỡng 0,567 (FAR 0,1%)</text><text x="10" y="195" class="t-small">impostor</text><path d="M123.9 190 L606.1 190" class="f-none s-muted" stroke-width="2"/><rect x="259.0" y="178" width="205.3" height="24" rx="4" class="f-pk-soft s-pk"/><path d="M361.3 178 L361.3 202" class="f-none s-pk" stroke-width="3"/><text x="10" y="240" class="t-small">genuine</text><path d="M468.0 235 L817.7 235" class="f-none s-muted" stroke-width="2"/><rect x="592.0" y="223" width="202.9" height="24" rx="4" class="f-phys-soft s-phys"/><path d="M724.5 223 L724.5 247" class="f-none s-phys" stroke-width="3"/><path d="M563.0 167 L563.0 257" class="f-none s-warn dash" stroke-width="2"/><path d="M90.0 268 L830.0 268" class="f-none s-line" stroke-width="2"/><path d="M213.3 268 L213.3 274" class="f-none s-line" stroke-width="2"/><text x="207.3" y="290" class="t-small">0</text><path d="M521.7 268 L521.7 274" class="f-none s-line" stroke-width="2"/><text x="515.7" y="290" class="t-small">0,5</text><path d="M830.0 268 L830.0 274" class="f-none s-line" stroke-width="2"/><text x="824.0" y="290" class="t-small">1,0</text><text x="90" y="312" class="t-small">điểm cosine (hộp p05 đến p95, vạch đậm: trung vị, vạch mảnh: min đến max)</text></svg>
</div>
<figcaption>Hình 1. Phân bố điểm cosine của cặp khác người và cặp cùng người trên Olivetti sạch, vẽ từ các phân vị mà lab in ra. Đường đứt là ngưỡng ở FAR 0,1% của từng mô hình; hàng ArcFace lấy từ lần chạy có cờ --with-arcface.</figcaption>
</figure>

## Cosine, FAR, FRR và ROC bằng numpy

Quy ước: điểm càng cao càng giống, `score >= t` là «khớp». **FAR** (False Accept Rate) là tỉ lệ cặp khác người có điểm từ `t` trở lên: chấp nhận nhầm. **FRR** (False Reject Rate) là tỉ lệ cặp cùng người có điểm dưới `t`: từ chối nhầm. Lab đo trên từng cặp ảnh nên đúng hơn là FMR/FNMR, tên bạn sẽ gặp ở báo cáo NIST; FAR/FRR của cả hệ thống còn tính ảnh không lấy được mặt. **ROC** là đường **TAR = 1 − FRR** theo FAR khi quét `t` (FAR theo FRR là đường DET). **EER** là điểm FAR bằng FRR; **AUC** là xác suất một điểm cùng người ngẫu nhiên cao hơn một điểm khác người ngẫu nhiên. Lab tự tính chúng bằng numpy:

```python
# metrics.py (trích)
def split_scores(scores, probe_labels, ref_labels, *, symmetric, allow_missing=False):
    missing = np.isnan(scores)
    if missing.any() and not allow_missing:
        raise ValueError("... báo riêng ảnh không lấy được mặt rồi truyền allow_missing=True")
    same = probe_labels[:, None] == ref_labels[None, :]
    keep = np.triu(np.ones(scores.shape, dtype=bool), k=1) if symmetric else ~np.eye(*scores.shape, dtype=bool)
    keep &= ~missing
    return scores[same & keep].astype(np.float64), scores[~same & keep].astype(np.float64)

def far_frr(genuine, impostor, thresholds):
    g, i = np.sort(genuine), np.sort(impostor)
    far = (len(i) - np.searchsorted(i, thresholds, side="left")) / len(i)   # số impostor >= t
    frr = np.searchsorted(g, thresholds, side="left") / len(g)              # số genuine  <  t
    return far, frr

def threshold_at_far(impostor, target_far):
    desc = np.sort(impostor)[::-1]
    allowed = int(np.floor(target_far * len(desc)))   # số impostor được phép nằm trên ngưỡng
    return float(np.nextafter(desc[allowed], np.inf))
```

`split_scores` **bỏ cặp ảnh với chính nó** (điểm 1,0 sẽ làm genuine đẹp giả). Hàng NaN là ảnh không lấy được mặt, tức không có embedding: hàm từ chối chúng trừ khi người gọi báo riêng số ảnh đó rồi truyền `allow_missing=True`. `far_frr` sắp xếp một lần rồi dùng `searchsorted`; `side="left"` làm đúng quy ước `>=`. FAR được tính bằng cách đếm số nguyên rồi mới chia (bảng lỗi bên dưới nói vì sao). Có 15 test pytest cho phần toán này.

## Phân bố genuine và impostor trên Olivetti

Bảng phân vị trên Olivetti sạch (mọi cặp không thứ tự), rồi ROC với ngưỡng ở FAR không quá 0,1% trên chính các cặp impostor này:

```text
  mô hình     nhóm        số cặp     min     p05     p50     p95     max
  sface       genuine      1,800   0.413   0.614   0.829   0.943   0.980
  sface       impostor    78,000  -0.145   0.074   0.240   0.407   0.637

  mô hình     đầu vào           AUC     EER  TAR@FAR 1%  TAR@FAR 0.1%   ngưỡng       FAR
  sface       căn chỉnh     0.99982  0.0072      0.9950        0.9706    0.567   0.00100
  sface       chỉ resize    0.94497  0.1271      0.6294        0.4772    0.750   0.00100

  sface       ngưỡng 0.3: FAR 0.2785, FRR 0.0000; ngưỡng 0.5: FAR 0.0061, FRR 0.0083
  sface       78 cặp ảnh vượt ngưỡng 0.567, thuộc 9/780 cặp người
```

SFace tách tốt hai nhóm trên bộ này (AUC 0,99982) nhưng vẫn chồng nhau: cặp cùng người thấp nhất là 0,413 trong khi cặp khác người cao nhất là 0,637. Căn chỉnh có giá trị đo được: TAR ở FAR 0,1% từ 0,4772 (chỉ resize) lên 0,9706, EER từ 0,1271 xuống 0,0072; phần lớn mức cải thiện đến từ việc đưa mặt về đúng tỉ lệ mẫu. Hai điều cần dè chừng. **Một**, 78.000 cặp không phải 78.000 mẫu độc lập: 78 cặp ảnh vượt ngưỡng chỉ thuộc 9 trong 780 cặp người, vì các cặp ảnh của cùng hai người tương quan mạnh. Mức bất định của FAR vì thế lớn hơn nhiều so với việc đếm 78 trên 78.000; các mục sau đo nó. **Hai**, FAR 0,001% không đo nổi với 78.000 cặp, và «FRR 0» trên 1.800 cặp không phải «không bao giờ từ chối nhầm».

Với ArcFace (cờ `--with-arcface`, chỉ để so thang điểm), EER sạch là 0,0005 và ngưỡng ở FAR 0,1% là 0,311, so với 0,567 của SFace: ngưỡng của hai mô hình không dùng chung được. Sau khi bỏ 59 ảnh không lấy được mặt, ArcFace chịu ảnh xuống cấp tốt hơn SFace nhiều trên các ảnh còn lại, nhưng FRR hệ thống vẫn bị ảnh hụt kéo lên, và ngưỡng chọn trên một nhóm người vẫn vượt mục tiêu ở nhóm khác (trích từ lần chạy có cờ):

```text
  arcface_r50 nặng             59    3,069   132,990  0.99986  0.0061  0.9974    0.9746   0.329        0.1692
  arcface_r50 78 cặp ảnh vượt ngưỡng 0.311, thuộc 12/780 cặp người
  arcface_r50 300 cách chia: FAR nhóm chưa thấy p05 0.00012, trung vị 0.00091, p95 0.00303; vượt 0.001 ở 44% số lần
```

Bộ này dễ vì Olivetti là 40 người chụp trong cùng một phòng, nên mục sau mô phỏng ảnh thẻ in và quét kém.

## Thẻ với selfie: ảnh xuống cấp, ảnh không lấy được mặt

Trong KYC, hai ảnh so với nhau không cùng điều kiện: ảnh thẻ là ảnh in nhỏ rồi quét, selfie thì sạch hơn. `degrade.py` mô phỏng ảnh thẻ bằng mờ Gauss, đổi gamma (tối), nhiễu hạt và nén JPEG ở hai mức. **Probe** là ảnh xuống cấp, **reference** là ảnh sạch, so mọi cặp có thứ tự i ≠ j.

Bản đầu của lab xử lý sai một điểm: khi ảnh xuống cấp, YuNet đôi khi không lấy được mặt, và bản đầu lùi âm thầm sang «chỉ resize» rồi vẫn nhúng ảnh, làm số liệu trộn hai nhóm khác hẳn nhau. Lab giờ không lùi: ảnh không lấy được mặt **không được nhúng**, bị đếm riêng và chuyển `needs_review`, còn FAR/FRR/EER tính trên ảnh lấy được mặt. Số ảnh hụt không nhỏ:

```text
  mức sạch : YuNet không lấy được mặt ở 0/400 ảnh (0.0%); những ảnh này chuyển needs_review và không có embedding; 19.7 ms/ảnh (gồm dò mặt)
  mức nhẹ  : YuNet không lấy được mặt ở 8/400 ảnh (2.0%); ...; 14.9 ms/ảnh (gồm dò mặt)
  mức nặng : YuNet không lấy được mặt ở 59/400 ảnh (14.8%); ...; 13.0 ms/ảnh (gồm dò mặt)
```

(Trích.) Với mức nhẹ là Gauss sigma 1,0, gamma 1,5, nhiễu 6, JPEG 40 và mức nặng là sigma 1,5, gamma 2,0, nhiễu 8, JPEG 30:

```text
  mô hình     kịch bản    hụt mặt  genuine  impostor      AUC     EER  TAR@1%  TAR@0.1%  ngưỡng  FRR hệ thống
  sface       sạch              0    3,600   156,000  0.99982  0.0072  0.9950    0.9706   0.567        0.0294
  sface       nhẹ               8    3,528   152,880  0.99638  0.0218  0.9683    0.9235   0.555        0.0950
  sface       nặng             59    3,069   132,990  0.98165  0.0577  0.8579    0.6882   0.533        0.4133
  (AUC, EER, TAR, ngưỡng: chỉ trên ảnh lấy được mặt. FRR hệ thống = FRR ở FAR <= 1e-3 cộng thêm các ảnh không lấy được mặt)
```

Cột «FRR hệ thống» là con số đáng nhìn. Ở mức nặng, EER trên ảnh lấy được mặt là 0,0577, nhưng 14,8% ảnh không qua nổi bước dò mặt, nên tỉ lệ người thật không qua được hệ thống là 41,33% (FRR 31,18% ở ảnh lấy được mặt, cộng phần ảnh hụt). Chỉ báo EER trên ảnh lấy được mặt là giấu 59 ảnh khó nhất, những ảnh mà bài 48 phải bắt bằng chất lượng và chuyển người duyệt.

Nếu ta chọn ngưỡng trên dữ liệu sạch rồi dùng nguyên cho ảnh thẻ nặng thì sao (chỉ ảnh lấy được mặt):

```text
  sface       ngưỡng 0.567: FAR 0.00024, FRR 0.4236
```

FAR vẫn dưới mục tiêu nhưng FRR là 42,36%: ngưỡng chọn ở điều kiện này không mang nguyên sang điều kiện khác.

## Chọn ngưỡng: chi phí, ba vùng và người chưa thấy

Có ba cách chọn ngưỡng, cho ba con số khác nhau. **Theo mục tiêu FAR** (ví dụ 0,1%): đơn giản, nhưng bỏ qua cái giá của FRR. **Theo EER**: tiện để so mô hình, hiếm khi là điều vận hành muốn. **Theo chi phí**: chi phí kỳ vọng mỗi lượt là p × C_FA × FAR + (1 − p) × C_FR × FRR, với p là tỉ lệ lượt mạo danh; lấy ngưỡng làm nó nhỏ nhất. Lab dùng giả định minh hoạ p = 2%, nhầm nhận tốn 100 đơn vị, nhầm từ chối tốn 1; dịch vụ thật phải thay bằng số của vận hành.

```python
# metrics.py (trích)
def expected_cost(far, frr, *, c_fa, c_fr, p_impostor):
    return p_impostor * c_fa * far + (1.0 - p_impostor) * c_fr * frr

def best_threshold_by_cost(genuine, impostor, *, c_fa, c_fr, p_impostor):
    thresholds, far, frr = roc_points(genuine, impostor)
    cost = expected_cost(far, frr, c_fa=c_fa, c_fr=c_fr, p_impostor=p_impostor)
    best = int(len(cost) - 1 - np.argmin(cost[::-1]))   # hòa thì lấy ngưỡng cao hơn (thận trọng hơn)
    return float(thresholds[best]), float(cost[best])
```

```text
  mô hình     kịch bản  cách chọn           ngưỡng       FAR     FRR  chi phí/lượt
  sface       sạch      tối thiểu chi phí    0.520   0.00374  0.0106       0.01783
  sface       sạch      EER                  0.493   0.00722  0.0072       0.02151
  sface       sạch      FAR <= 1e-3          0.567   0.00100  0.0294       0.03086
  sface       nặng      tối thiểu chi phí    0.438   0.02103  0.0994       0.13946
  sface       nặng      EER                  0.394   0.05782  0.0577       0.17217
  sface       nặng      FAR <= 1e-3          0.533   0.00099  0.3118       0.30758
```

Trên ảnh nặng, ngưỡng tối thiểu chi phí là 0,438, **thấp hơn** ngưỡng ở FAR 0,1% (0,533): nó chấp nhận FAR 2,10% để đổi lấy FRR 9,94% thay vì 31,18%, vì với giả định này trọng số hiệu dụng của FAR là 0,02 × 100 = 2 còn của FRR là 0,98. Kết quả phụ thuộc mạnh vào giả định (nhầm nhận đắt hơn hoặc p cao hơn thì ngưỡng bị đẩy lên).

Vì máy chỉ **gợi ý** cho người duyệt, ta dùng hai ngưỡng: `t_high` (FAR ≤ 0,1%) và `t_low` (FRR ≤ 1%), chia điểm thành ba vùng (Hình 2, SFace trên ảnh thẻ nặng, ảnh lấy được mặt).

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Trục điểm cosine của SFace trên kịch bản thẻ nặng, chỉ tính ảnh lấy được mặt, chia ba vùng bởi hai ngưỡng 0,241 và 0,533. Cặp cùng người rơi 0,98 phần trăm vào vùng không khớp, 30,21 vào vùng chưa rõ, 68,82 vào vùng khớp; cặp khác người rơi 51,99, 47,92 và 0,10."><defs></defs><rect x="90.0" y="50" width="271.9" height="70" class="f-warn-soft s-warn"/><rect x="361.9" y="50" width="180.1" height="70" class="f-surface s-line dash"/><rect x="542.0" y="50" width="288.0" height="70" class="f-phys-soft s-phys"/><text x="98.0" y="74" class="t-mono">likely_mismatch</text><text x="369.9" y="74" class="t-mono">unclear</text><text x="550.0" y="74" class="t-mono">likely_match</text><text x="98.0" y="98" class="t-small">cùng người 0,98%</text><text x="369.9" y="98" class="t-small">cùng người 30,21%</text><text x="550.0" y="98" class="t-small">cùng người 68,82%</text><text x="98.0" y="114" class="t-small">khác người 51,99%</text><text x="369.9" y="114" class="t-small">khác người 47,92%</text><text x="550.0" y="114" class="t-small">khác người 0,10%</text><path d="M361.9 120 L361.9 150" class="f-none s-warn dash" stroke-width="2"/><path d="M542.0 120 L542.0 150" class="f-none s-warn dash" stroke-width="2"/><text x="321.9" y="170" class="t-small">t_low = 0,241</text><text x="512.0" y="170" class="t-small">t_high = 0,533</text><path d="M90.0 196 L830.0 196" class="f-none s-line" stroke-width="2"/><path d="M213.3 196 L213.3 202" class="f-none s-line" stroke-width="2"/><text x="207.3" y="220" class="t-small">0</text><path d="M521.7 196 L521.7 202" class="f-none s-line" stroke-width="2"/><text x="515.7" y="220" class="t-small">0,5</text><path d="M830.0 196 L830.0 202" class="f-none s-line" stroke-width="2"/><text x="824.0" y="220" class="t-small">1,0</text><text x="470" y="240" class="t-small">điểm cosine của SFace, thẻ nặng so với selfie sạch</text></svg>
</div>
<figcaption>Hình 2. Hai ngưỡng chia điểm thành ba vùng gợi ý. Cả ba vùng đều dẫn tới người duyệt; vùng chỉ quyết định thứ tự ưu tiên và mức nghi ngờ ghi kèm.</figcaption>
</figure>

```text
  Ba vùng gợi ý cho người duyệt (sface, kịch bản nặng, ảnh lấy được mặt): t_high = 0.533 (FAR <= 1e-3), t_low = 0.241 (FRR <= 1%)
  genuine   likely_match  68.82%   unclear  30.21%   likely_mismatch   0.98%
  impostor  likely_match   0.10%   unclear  47.92%   likely_mismatch  51.99%
  với 2% lượt mạo danh, vùng unclear chiếm 30.56% số ảnh lấy được mặt; cộng 59/400 ảnh không lấy được mặt thì hàng người duyệt chiếm khoảng 40.80% số lượt
```

Ở mức ảnh nặng, khoảng 40,80% lượt vào hàng người duyệt, gồm cả 59/400 ảnh không lấy được mặt. Không vùng nào tự duyệt hay tự từ chối hồ sơ.

Ngưỡng có tổng quát hoá sang người chưa thấy không? Một cách chia 20/20 theo chỉ số cho một con số, đảo chiều cho con số khác, nên lab in cả hai chiều, rồi 300 cách chia ngẫu nhiên rời người (chọn ngưỡng ở FAR 0,1% trên nhóm A, đo trên nhóm B), và bootstrap theo người (bốc lại 40 người có hoàn lại, 200 lần):

```text
  sface       người 0-19 -> 20-39: ngưỡng 0.533; nhóm chọn FAR 0.00098 FRR 0.2690 | nhóm chưa thấy FAR 0.00063 FRR 0.3615
  sface       người 20-39 -> 0-19: ngưỡng 0.523; nhóm chọn FAR 0.00100 FRR 0.3256 | nhóm chưa thấy FAR 0.00161 FRR 0.2313
  sface       300 cách chia: FAR nhóm chưa thấy p05 0.00012, trung vị 0.00106, p95 0.00370; vượt 0.001 ở 52% số lần; FRR p05 0.2062, trung vị 0.3046, p95 0.4095
  sface       bootstrap 200 lần theo người, khoảng [2.5%, trung vị, 97.5%]:
    chọn lại ngưỡng ở FAR <= 1e-3: FRR 0.2262 / 0.2933 / 0.3830; EER 0.0431 / 0.0575 / 0.0754
    giữ ngưỡng cố định 0.533: FAR 0.00026 / 0.00090 / 0.00206; FRR 0.2543 / 0.3097 / 0.3833
```

Hai chiều của cách chia đầu cho kết luận ngược nhau: chiều đi, FAR trên nhóm chưa thấy (0,063%) dưới mục tiêu; chiều về, 0,161% vượt mục tiêu; FRR cũng đảo (36,15% so với 23,13%). Không nên kết luận từ một lần chia. Qua 300 cách chia, FAR trên nhóm chưa thấy có trung vị 0,106%, vượt 0,1% ở 52% số lần, p95 là 0,370%: ngưỡng chọn ở FAR 0,1% trên một nhóm người **không bảo đảm** FAR 0,1% với nhóm khác. Bootstrap cho thấy tương tự: giữ nguyên ngưỡng thì FAR dao động từ 0,026% đến 0,206%. Khi chọn lại ngưỡng trong từng lần bốc, FAR luôn ở mục tiêu theo cách xây dựng, nên số đáng đọc là FRR (22,62% đến 38,30%). Chỉ có 40 người, nên các khoảng này là **ít nhất** mức bất định thật; bài 53 quay lại theo nhóm người.

## Chạy và kiểm chứng

Từ thư mục gốc repository (lần đầu cần mạng để tải YuNet, SFace và bộ Olivetti):

```bash
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold examples/toi-noi-python/49-face-embedding-cosine-threshold/main.py
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold examples/toi-noi-python/49-face-embedding-cosine-threshold/main.py --verify
uv run --project examples/toi-noi-python/49-face-embedding-cosine-threshold pytest examples/toi-noi-python/49-face-embedding-cosine-threshold -q
```

Thêm `--with-arcface` vào hai lệnh đầu để so với ArcFace R50: lab in cảnh báo giấy phép rồi tải gói 288.621.354 byte, và chạy lâu hơn nhiều trên CPU vì ArcFace phải qua 1.600 ảnh. `--verify` kiểm các bất biến đã in: số cặp (đúng 9 genuine và 390 impostor cho mỗi ảnh lấy được mặt), vector có độ dài 1, căn chỉnh tốt hơn chỉ resize, số ảnh hụt tăng theo mức xuống cấp, `split_scores` từ chối hàng NaN, FAR đo được không vượt mục tiêu, EER tăng đúng thứ tự, ngưỡng tối thiểu chi phí không đắt hơn hai cách chọn còn lại, mọi cách chia rời người, p95 FAR trên người chưa thấy vượt mục tiêu. Ca bất thường: ảnh so với chính nó bị loại khỏi genuine; ảnh đen và nhiễu trắng không lấy được mặt nên không có embedding.

Dòng cuối của lần chạy `--verify` mặc định mới nhất (SFace), kết quả pytest, và dòng cuối khi thêm `--with-arcface` (49 ca vì có thêm các ca của ArcFace):

```text
OK 32/32
15 passed in 0.88s
OK 49/49
```

Thời gian một ảnh SFace là 14,2 ms (trung vị 15 lần, load average 117,6; dò mặt và căn chỉnh YuNet là 19,7 ms mỗi ảnh sạch): chỉ đọc thứ bậc lớn vì máy rất bận. Embedding lưu ở `out/embeddings_sface.npy` và `labels.npy` là **dữ liệu sinh trắc học**: đã gitignore, và bài 53 bàn về thời hạn lưu, đồng ý và quyền xoá.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| EER đẹp nhưng người thật vẫn bị từ chối nhiều | Có ảnh không lấy được mặt không (cột «hụt mặt») và FRR hệ thống |
| Cùng một ngưỡng cho kết quả lạ khi đổi mô hình | Ngưỡng là của từng mô hình; đo lại bằng ROC |
| `ValueError: ... hàng có NaN` từ `split_scores` | Có ảnh không lấy được mặt: báo riêng rồi truyền `allow_missing=True` |
| FAR đo được là `0.0010000000000000009`, lớn hơn `1e-3` | Sai số của `1 - k/n`; đếm số nguyên rồi chia (`(n - k) / n`) |
| `Expected shape from model of {1,512} does not match actual shape of {32,512}` (chỉ với ArcFace) | Cảnh báo của ONNX Runtime vì đầu ra khai báo batch 1; lab đặt mức log chỉ còn lỗi, kết quả vẫn đúng |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Olivetti chỉ có 40 người chụp trong phòng thí nghiệm, ảnh xám 64 × 64; ảnh «thẻ» là ảnh xuống cấp bằng code, không phải ảnh in thật. Các số FAR, FRR, EER ở trên cho thấy cách đo, không phải mức lỗi ngoài đời, và số ảnh không lấy được mặt (14,8% ở mức nặng) cũng chỉ đúng cho kiểu xuống cấp này. Chi phí 100 và 1 và tỉ lệ mạo danh 2% là giả định minh hoạ. Ta chưa đánh giá theo nhóm tuổi, giới hay màu da và không được suy ra mô hình công bằng với mọi nhóm (bài 53). Điểm so khớp là đầu vào cho người duyệt hồ sơ (người duyệt `RV-001` trong từ điển miền), không phải quyết định xác minh.</p>
</div>

**Bài tập 1:** Với 78.000 cặp khác người, ngưỡng FAR ≤ 0,1% cho phép tối đa bao nhiêu cặp khác người nằm trên ngưỡng?

**Bài tập 2:** Một hồ sơ có điểm SFace 0,40 trên ảnh thẻ nặng. Theo hai ngưỡng của Hình 2, hồ sơ nằm vùng nào, và hệ thống làm gì với nó?

<details><summary>Xem lời giải</summary><p><b>1.</b> 0,1% × 78.000 = <b>78 cặp</b> (vị trí <code>allowed</code> trong <code>threshold_at_far</code>; bảng ở trên cũng đếm đúng 78 cặp ảnh vượt ngưỡng, thuộc 9 cặp người, nên 78 cặp này không độc lập). <b>2.</b> 0,40 nằm giữa <code>t_low</code> = 0,241 và <code>t_high</code> = 0,533, tức vùng <code>unclear</code>: hệ thống không duyệt, không từ chối, mà đưa hồ sơ vào hàng người duyệt với nhãn «chưa rõ».</p></details>

Bài trước: [bài 48](/articles/python-face-detection-landmarks-mediapipe-opencv). Bài tiếp: [bài 50](/articles/python-liveness-anti-spoofing-chat-luong-anh), liveness và chống giả mạo cơ bản.

## Tài liệu gốc

- [InsightFace: python-package (mô hình, giấy phép)](https://github.com/deepinsight/insightface/tree/master/python-package).
- [OpenCV Zoo: SFace (face recognition)](https://github.com/opencv/opencv_zoo/tree/main/models/face_recognition_sface).
- [ArcFace: Additive Angular Margin Loss for Deep Face Recognition](https://arxiv.org/abs/1801.07698).
- [ONNX Runtime: Python API](https://onnxruntime.ai/docs/api/python/api_summary.html).
- [scikit-learn: roc_curve](https://scikit-learn.org/stable/modules/generated/sklearn.metrics.roc_curve.html).
- [NIST: Face Recognition Technology Evaluation (FRTE) 1:1 Verification](https://pages.nist.gov/frvt/html/frvt11.html).
