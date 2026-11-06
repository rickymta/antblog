---
slug: python-face-detection-landmarks-mediapipe-opencv
title: "Face detection và landmarks: tìm khuôn mặt trong ảnh giấy tờ và selfie"
summary: "Bài 48: so Haar, YuNet (OpenCV DNN) và MediaPipe trên ảnh thẻ, selfie tổng hợp; căn chỉnh theo 5 landmark về mẫu 112x112; đo mờ, tối; gom lý do chuyển người duyệt. Có số đo thật và lab chạy được."
category: programming
level: advanced
tags: [python, toi-noi, opencv, mediapipe, face-detection]
language: vi
---

Tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) nộp hồ sơ KYC gồm ảnh căn cước và một ảnh selfie chụp bằng điện thoại. Service `kyc-face` sẽ phải trả lời câu hỏi «người trong selfie có phải người trên thẻ không», nhưng trước đó có bốn câu nhỏ hơn mà nếu bỏ qua thì điểm so khớp chẳng có nghĩa gì: ảnh có mặt không, có đúng một mặt không, mặt có đủ rõ không, và mặt có được đặt vào khung giống như mặt mà mô hình so khớp đã học không. Bài này làm bốn việc đó, rồi quyết định ảnh nào đi tiếp và ảnh nào sang người duyệt.

Đây là bài 48 của lộ trình **Python nâng cao qua Tới Nơi**, bài đầu của phần 06 (so khớp khuôn mặt). Bạn cần quen mảng ảnh và OpenCV ở [bài 30](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh) và luồng KYC ở [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một selfie bị nghiêng đầu 30 độ trong mặt phẳng ảnh. Haar cascade có sẵn trong OpenCV và mạng nhỏ YuNet, bộ nào tìm được mặt? Và khi đã tìm được, đường nối hai mắt từ 5 landmark của YuNet có cho đúng góc 30 độ không?</p>
<details><summary>Xem đáp án</summary><p>Trên 40 ảnh ghép của lab: Haar tìm được <b>0/40</b>, YuNet <b>37/40</b>. Nhưng góc YuNet báo lệch trung vị 7,89 độ so với góc thật; bộ FaceLandmarker 478 điểm của MediaPipe lệch 0,62 độ. «Tìm thấy mặt» và «landmark đủ chính xác để căn chỉnh» là hai câu hỏi khác nhau.</p></details>
</div>

## Bốn câu hỏi trước khi so khớp

Hình 1 là chuỗi bước của lab `48-face-detection-landmarks`. Mỗi bước có thể dừng chuỗi: không có mặt thì không có gì để căn chỉnh, mặt mờ thì embedding ở bài 49 không đáng tin. Điểm cần chốt từ đầu là ảnh dừng lại **không bị từ chối**; nó được chuyển sang người duyệt kèm lý do, đúng tinh thần của loạt bài: máy chuẩn bị đầu vào, người quyết định.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 230" role="img" aria-label="Năm bước: ảnh vào, phát hiện hộp và 5 landmark, căn chỉnh về 112 x 112, đo chất lượng, rồi embedding ở bài 49; bước phát hiện và đo chất lượng có thể chuyển hồ sơ sang người duyệt kèm lý do."><defs><marker id="py48-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="py48-0-warn" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs><rect x="14" y="30" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="28" y="66" class="t-b">Ảnh vào</text><text x="28" y="92" class="t-small">thẻ CCCD, selfie</text><path d="M154 75 L192 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py48-0-arrow)"/><rect x="192" y="30" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="206" y="66" class="t-b">Phát hiện</text><text x="206" y="92" class="t-small">hộp + 5 landmark</text><path d="M332 75 L370 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py48-0-arrow)"/><rect x="370" y="30" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="384" y="66" class="t-b">Căn chỉnh</text><text x="384" y="92" class="t-small">đồng dạng, 112x112</text><path d="M510 75 L548 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py48-0-arrow)"/><rect x="548" y="30" width="140" height="90" rx="10" class="f-pk-soft s-pk"/><text x="562" y="66" class="t-b">Chất lượng</text><text x="562" y="92" class="t-small">mờ, sáng, cỡ mặt</text><path d="M688 75 L726 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py48-0-arrow)"/><rect x="726" y="30" width="140" height="90" rx="10" class="f-phys-soft s-phys"/><text x="740" y="66" class="t-b">Embedding</text><text x="740" y="92" class="t-small">bài 49</text><path d="M262 120 L262 166" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py48-0-warn)"/><path d="M618 120 L618 166" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py48-0-warn)"/><rect x="192" y="166" width="496" height="48" rx="10" class="f-warn-soft s-warn"/><text x="208" y="195" class="t-small">không mặt, nhiều mặt, nhỏ, cắt mép, nghiêng, mờ, tối: sang người duyệt</text></svg>
</div>
<figcaption>Hình 1. Chuỗi xử lý ảnh khuôn mặt trước khi so khớp. Hai bước có thể chuyển hồ sơ sang người duyệt; bước cuối là bài sau.</figcaption>
</figure>

Dữ liệu là điểm khó nhất của loạt bài này vì ta không được commit ảnh người. Lab dùng bộ **Olivetti faces** qua `sklearn.datasets.fetch_olivetti_faces`: 400 ảnh xám 64 × 64 của 40 người (10 ảnh mỗi người), chụp trong phòng thí nghiệm năm 1992 đến 1994. Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Lần đầu sklearn tải về khoảng 4,26 MB và lưu bản cache 1,4 MB (1.405.392 byte) trong `~/scikit_learn_data`, không nằm trong repository. Từ đó `synth.py` sinh hai loại ảnh. Ảnh **chân dung trên thẻ**: dán khuôn mặt vào ô ảnh của thẻ căn cước giả của bài 30. Ảnh **selfie**: dán khuôn mặt lên nền «phòng» 640 × 480 (dải màu, vài mảng sáng tối, nhiễu), có mặt nạ tròn mềm để không có cạnh vuông, với kích thước và góc xoay chọn được. Vì ta tự dán nên biết ô «đáp án» và biết góc xoay thêm vào. «Góc thật» trong bài là góc xoay ta thêm vào; ảnh Olivetti gốc có thể đã nghiêng nhẹ sẵn, nên ở roll 0 độ lệch đo được (0,85 độ với YuNet, 0,59 độ với FaceLandmarker) gồm cả độ nghiêng sẵn có lẫn sai số của bộ ước lượng, và ta không tách được hai phần. Cái giá là trung thực: đây không phải ảnh chụp bằng điện thoại, mà ảnh xám của một bộ dữ liệu cũ; phần giới hạn ở cuối bài nói tiếp.

Mô hình tải lần đầu vào `~/.cache/toinoi-python/models`, lab in URL và kích thước, rồi kiểm SHA-256. Haar thì có sẵn trong gói OpenCV nên không tải.

```text
  yunet_2023mar.onnx                 232,589 byte  https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx
  blaze_face_short_range.tflite      229,746 byte  https://storage.googleapis.com/mediapipe-models/face_detector/blaze_face_short_range/float16/1/blaze_face_short_range.tflite
  face_landmarker.task             3,758,596 byte  https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task
```

Về phiên bản, đã thử ngày 2026-11-06 trên Python 3.14.7: MediaPipe 1.2.2 và OpenCV 4.14.0 đều có wheel cho 3.14, nên lab không phải lùi về 3.13. Có một điểm vướng cần ghi lại. Bản chính `opencv-python-headless` 5.0.0.93 bỏ cả `cv2.CascadeClassifier` lẫn file `haarcascade_*.xml`; bản `opencv-contrib-python` 5.0.0.93 vẫn còn class và `cv2.data.haarcascades` nhưng thư mục đó không kèm file xml nào, nên Haar báo `RuntimeError` khi nạp. Mặt khác `mediapipe` kéo theo `opencv-contrib-python`, mà hai gói `opencv-*` cùng ghi đè thư mục `cv2` nên không cài chung được. Lab chọn `opencv-contrib-python>=4.14,<5`: một gói duy nhất, có đủ cả xml đi kèm.

## Ba bộ phát hiện: Haar, YuNet và BlazeFace

Mọi bộ phát hiện của lab (cùng với FaceLandmarker ở mục sau) cài theo một giao diện để so cho công bằng: nhận ảnh BGR, trả danh sách `Detection` gồm hộp, điểm tin cậy và landmark nếu có.

```python
# faces.py (trích)
class Detector(Protocol):
    name: str

    def detect(self, bgr: np.ndarray) -> list[Detection]: ...


class YuNetDetector:
    name = "yunet"

    def __init__(self, score_threshold: float = 0.7) -> None:
        model = ensure_model(YUNET, verbose=False)
        # Kích thước đầu vào đặt lại theo từng ảnh trước khi detect.
        self._net = cv2.FaceDetectorYN.create(str(model), "", (320, 320), score_threshold, 0.3, 5000)

    def detect(self, bgr: np.ndarray) -> list[Detection]:
        height, width = bgr.shape[:2]
        self._net.setInputSize((width, height))
        _, rows = self._net.detect(bgr)
        if rows is None:
            return []
        found = []
        for row in rows:
            x, y, w, h = (int(round(v)) for v in row[:4])
            found.append(Detection((x, y, w, h), float(row[14]), row[4:14].reshape(5, 2).astype(np.float64)))
        return found
```

`Protocol` là cách của bài 14 để bộ so sánh không cần biết lớp cụ thể. **Haar** là cascade các bộ lọc Haar huấn luyện từ lâu, chạy offline, chỉ cho hộp; lab gọi `detectMultiScale(gray, scaleFactor=1.1, minNeighbors=5, minSize=(40, 40))`, và các tham số này quyết định cả số mặt bắt được lẫn số mặt thừa (lab không dò lưới tham số, nên đừng đọc kết quả Haar dưới đây như «giới hạn của Haar»). **YuNet** là mạng nhỏ chạy qua OpenCV DNN (`cv2.FaceDetectorYN`), cho hộp và 5 landmark. **BlazeFace** (MediaPipe Tasks `FaceDetector`) cho hộp và 6 keypoint, có hai mô hình: short range (229.746 byte) và full range (1.083.786 byte). Lab đo hai câu hỏi tách bạch. «Có hộp» là bộ phát hiện trả ít nhất một hộp, tức đã tìm ra mặt. «Trúng» còn đòi hộp khớp ô đáp án (IoU ít nhất 0,4). Ô đáp án là ô vuông đã dán, và khi ảnh bị xoay thì là hộp bao của ô vuông đã xoay, rộng hơn mặt; vì thế ở các hàng nghiêng, «trúng» thấp hơn «có hộp» dù mặt đã được tìm ra. Kết quả trên 40 người, mỗi người một ảnh; mỗi ô là «trúng / có hộp / số mặt thừa»:

```text
điều kiện                        haar          yunet      blazeface blazeface-full  mp-landmarker   (trúng / có hộp / mặt thừa, trên 40 ảnh)
chân dung trên thẻ            38/38/2        40/40/0        39/39/0        38/38/0        39/39/0
selfie mặt 200 px             39/39/0        40/40/0        39/39/0        37/37/0        39/39/0
selfie mặt 80 px              37/37/0        40/40/0          4/8/0        18/18/0          4/4/0
selfie mặt 40 px              31/31/0        30/30/0          0/1/0          5/5/0          0/0/0
nghiêng 15 độ                 28/29/0        40/40/0        39/39/0        31/31/0        39/39/0
nghiêng 30 độ                   0/1/0        37/37/0        39/40/0        24/34/0        40/40/0
nghiêng 60 độ                   0/1/0         5/11/0        29/39/0          6/6/0        39/39/1
mờ Gauss sigma 3              38/38/0        40/40/0        38/38/0        38/38/0        38/38/0
tối gamma 3                   34/35/0        40/40/0        40/40/0        37/37/0        40/40/0
```

Đọc bảng theo từng cột. **Haar** bắt được mặt thẳng từ 80 px trở lên (37 đến 39/40) và còn 31/40 ở 40 px, nhưng chỉ có hộp ở 1/40 ảnh khi nghiêng 30 độ và yếu hơn khi ảnh tối (34/40); trên thẻ nó là bộ duy nhất báo mặt thừa (2 hộp). **YuNet** giữ được mặt 80 px (40/40) và nghiêng đến 30 độ (37/40); ở 60 độ nó có hộp ở 11/40 ảnh nhưng chỉ 5/40 trúng. **BlazeFace** short range chỉ có hộp ở 8/40 ảnh khi mặt 80 px trên khung 640 × 480 (trúng 4/40): mô hình này dành cho mặt chiếm phần lớn khung, như selfie cầm gần; đổi lại ở 60 độ nó có hộp ở 39/40 ảnh. Bản **full range** bắt mặt nhỏ tốt hơn (80 px: 18/40, 40 px: 5/40) nhưng kém ở mặt nghiêng (60 độ chỉ 6/40 có hộp) và hơi kém ở mặt lớn (200 px: 37/40 so với 39/40). **FaceLandmarker** cùng họ MediaPipe nên cùng điểm yếu cỡ mặt, nhưng có hộp ở 39/40 ảnh nghiêng 60 độ. Không bộ nào thắng mọi hàng. Lab dùng YuNet làm mặc định vì kết quả trên bộ ghép này: mặt nhỏ và nghiêng vừa phải đều giữ được, đúng kiểu ảnh thẻ chụp bằng điện thoại mà ta hình dung; ảnh điện thoại thật thì ta chưa đo, và lựa chọn đó phải kiểm lại bằng ảnh thật.

Hai con số kiểm khác. Trên 40 ảnh không có mặt (thẻ để trống ô ảnh, nền phòng thuần) Haar, YuNet, BlazeFace full range và FaceLandmarker không báo nhầm lần nào, BlazeFace short range báo nhầm 1 hộp. Trên 39 ảnh có hai người đứng cạnh nhau, YuNet báo đúng 2 mặt ở cả 39. Thời gian trung vị mỗi ảnh 640 × 480 trên CPU, một luồng gọi: Haar 11,2 ms, YuNet 16,4 ms, BlazeFace short range 2,6 ms, BlazeFace full range 6,5 ms, FaceLandmarker 12,7 ms. Con số này đo khi load average 1 phút của máy là khoảng 153 trên 10 lõi (máy đang bận việc khác), nên chỉ đọc thứ bậc lớn: BlazeFace short range nhanh nhất cỡ vài lần; đừng suy ra hơn kém giữa các bộ còn lại.

Về ô đáp án: hộp của bộ phát hiện không trùng khít ô đã dán, và mỗi bộ vẽ hộp theo một cách khác (lab không đo từng bộ rộng hơn hay hẹp hơn ô dán bao nhiêu). Vì vậy IoU không thể tiến sát 1 và ngưỡng trúng đặt ở 0,4 thay vì 0,5 hay cao hơn; mục đích là biết bộ nào tìm ra mặt, không phải bộ nào vẽ hộp «đúng hơn».

## Landmark và căn chỉnh

Mô hình embedding được huấn luyện trên mặt đã đặt vào cùng một khung: hai mắt nằm ngang ở cùng độ cao, mũi và hai khoé miệng ở vị trí cố định trong ảnh 112 × 112. Ta đưa ảnh về khung đó bằng **phép đồng dạng**: xoay, co giãn đều và tịnh tiến, không bóp méo theo chiều nào nên mặt không bị dẹt. YuNet cho 5 điểm theo thứ tự mắt phải, mắt trái, đỉnh mũi, khoé miệng phải, khoé miệng trái («phải» là của người trong ảnh, nên mắt phải nằm bên trái khung hình). Ta tìm phép đồng dạng biến 5 điểm đó thành 5 điểm của mẫu ArcFace bằng bình phương tối thiểu.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Bên trái: khuôn mặt nghiêng với năm điểm landmark và đường nối hai mắt nghiêng. Bên phải: mẫu ArcFace 112 x 112 vẽ gấp đôi, hai mắt nằm ngang. Mũi tên giữa là phép xoay, co và dịch tìm bằng bình phương tối thiểu."><defs><marker id="py48-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><g transform="rotate(25 200 160)"><rect x="110" y="70" width="180" height="180" rx="14" class="f-phys-soft s-phys"/><path d="M165 128 L235 128" class="f-none s-pk dash" stroke-width="2"/><circle cx="165" cy="128" r="6" class="f-warn"/><circle cx="235" cy="128" r="6" class="f-warn"/><circle cx="200" cy="170" r="6" class="f-warn"/><circle cx="172" cy="210" r="6" class="f-warn"/><circle cx="228" cy="210" r="6" class="f-warn"/></g><text x="40" y="300" class="t-small">ảnh gốc: đường nối hai mắt nghiêng</text><path d="M310 165 L540 165" class="f-none s-phys" stroke-width="2" marker-end="url(#py48-1-arrow)"/><text x="344" y="148" class="t-small">xoay + co + dịch</text><text x="330" y="190" class="t-small">5 điểm về 5 điểm</text><rect x="560" y="60" width="224" height="224" rx="4" class="f-phys-soft s-phys"/><path d="M637 163 L707 163" class="f-none s-pk dash" stroke-width="2"/><circle cx="637" cy="163" r="6" class="f-warn"/><circle cx="707" cy="163" r="6" class="f-warn"/><circle cx="672" cy="203" r="6" class="f-warn"/><circle cx="643" cy="245" r="6" class="f-warn"/><circle cx="702" cy="244" r="6" class="f-warn"/><text x="560" y="310" class="t-small">mẫu ArcFace 112x112, vẽ gấp đôi: mắt ngang</text></svg>
</div>
<figcaption>Hình 2. Căn chỉnh là tìm phép xoay, co và dịch đưa năm landmark của ảnh sang năm điểm cố định của mẫu. Sau phép biến đổi, đường nối hai mắt nằm ngang.</figcaption>
</figure>

```python
# faces.py (trích)
ARCFACE_112 = np.array([[38.2946, 51.6963], [73.5318, 51.5014], [56.0252, 71.7366],
                        [41.5493, 92.3655], [70.7299, 92.2041]])

def similarity_transform(src, dst):
    """Umeyama 1991: phép đồng dạng bình phương tối thiểu từ src sang dst, ma trận 2 x 3."""
    src_mean, dst_mean = src.mean(axis=0), dst.mean(axis=0)
    src_c, dst_c = src - src_mean, dst - dst_mean
    u, s, vt = np.linalg.svd(dst_c.T @ src_c / len(src))
    flip = np.eye(2)
    if np.linalg.det(u) * np.linalg.det(vt) < 0:     # loại phép phản chiếu
        flip[1, 1] = -1
    rotation = u @ flip @ vt
    scale = (s * np.diag(flip)).sum() / src_c.var(axis=0).sum()
    translation = dst_mean - scale * rotation @ src_mean
    return np.hstack([scale * rotation, translation[:, None]])

def align_face(bgr, landmarks, size=112):
    matrix = similarity_transform(landmarks, ARCFACE_112 * size / 112)
    return cv2.warpAffine(bgr, matrix, (size, size), flags=cv2.INTER_LINEAR,
                          borderMode=cv2.BORDER_REPLICATE)
```

Hàm toán đúng chưa là một câu hỏi kiểm được: lab xoay mẫu ArcFace 25 độ, co 0,7 lần, dịch rồi nhờ `similarity_transform` đưa về; sai số lớn nhất còn `2.8e-14` điểm ảnh. Câu hỏi khó hơn là landmark đầu vào có đúng không. Lab đo bằng cách xoay ảnh theo góc thật đã biết, ước lượng góc nghiêng từ đường nối hai mắt (YuNet 5 điểm, và FaceLandmarker 478 điểm rút 5 điểm: tâm mống mắt, đỉnh mũi, hai khoé miệng), rồi căn chỉnh và **đo lại** độ nghiêng còn lại trên crop bằng FaceLandmarker (crop phóng gấp đôi). Bảng là trung vị của |sai số|, đơn vị độ:

```text
 roll thật     yunet    mp 478     còn lại (YuNet)    còn lại (MP)   YuNet tìm thấy
       -30      8.11      0.76                6.21            0.75            39/40
       -15      3.88      0.58                3.81            0.76            40/40
         0      0.85      0.59                0.74            0.89            40/40
        15      2.67      0.65                2.20            0.67            40/40
        30      7.89      0.62                5.13            0.76            37/40
        40     18.25      0.61               14.82            0.78            19/40
```

Có ba điều trong bảng. Một, ở roll 0 cả hai chỉ lệch dưới 1 độ, nên «nền nhiễu» của phép đo khoảng 0,6 đến 0,9 độ. Hai, sai số của YuNet tăng nhanh theo góc: 2,67 độ ở 15, 7,89 ở 30 và 18,25 ở 40, kèm theo số ảnh còn tìm thấy rớt xuống 19/40; với ảnh nghiêng 40 độ, sai số trung vị 18,25 độ nghĩa là YuNet báo chừng 22 độ, và ca nghiêng 35 độ ở mục sau cũng chỉ ra 16,5 độ. Ước lượng của YuNet dường như bão hòa khi góc lớn; lab chưa đo nguyên nhân nên ta chỉ ghi nhận hiện tượng. Ba, căn chỉnh bằng landmark kém thì **mặt vẫn nghiêng sau khi căn chỉnh**: dùng YuNet ở roll 30, crop còn lệch 5,13 độ; dùng FaceLandmarker, còn 0,76 độ, chỉ bằng nền nhiễu. Cách phòng ngừa là đo lại sau căn chỉnh như bảng này, hoặc dùng bộ landmark dày hơn khi YuNet báo roll vượt vài độ. Đo lại bằng chính FaceLandmarker có thiên về nó; vì thế ta ghi cả nền nhiễu 0,6 đến 0,9 độ.

## Chất lượng ảnh: mờ, tối, cỡ mặt

Căn chỉnh xong, ta đo ba thứ trên crop 112 × 112: phương sai của ảnh Laplacian (càng nhỏ càng mờ, vì ít cạnh sắc), độ sáng trung bình và độ tương phản. Có một cái bẫy nhìn thấy được trong output của lab:

```text
ảnh sạch, selfie: trung vị phương sai Laplacian đo cả crop 147.5, đo vùng lõi 194.6; vùng lõi nhỏ nhất 72.6; độ sáng trong [109.6, 170.4]
ảnh sạch, thẻ: trung vị phương sai Laplacian đo cả crop 4248.6, đo vùng lõi 230.7; vùng lõi nhỏ nhất 78.3; độ sáng trong [108.9, 172.7]
```

Cùng một mặt sắc nét, đo trên cả crop cho 4.248,6 ở ảnh thẻ nhưng 147,5 ở selfie: ở thẻ, crop 112 × 112 còn chứa khung viền ô ảnh và chữ in cạnh đó, những cạnh cứng này làm phương sai tăng vọt dù chẳng liên quan đến độ nét của mặt. Vì thế `face_quality` đo trên **vùng lõi** (từ ngang lông mày đến dưới miệng, giữa hai má) và hai nguồn ảnh cho số cùng cỡ (194,6 và 230,7). Đây cũng là lý do ta đo trên crop căn chỉnh chứ không phải trên ảnh gốc: mặt chiếm 30% hay 5% khung ảnh thì phương sai Laplacian của ảnh gốc khác hẳn.

```python
# faces.py (trích)
FACE_CORE = (slice(36, 100), slice(26, 86))

def face_quality(aligned_bgr, *, core=True):
    gray = cv2.cvtColor(aligned_bgr, cv2.COLOR_BGR2GRAY)
    if core:
        gray = gray[FACE_CORE]
    return Quality(blur=float(cv2.Laplacian(gray, cv2.CV_64F).var()),
                   brightness=float(gray.mean()), contrast=float(gray.std()))
```

Ngưỡng chọn thế nào? Lab không lấy số trong sách mà đặt dưới mức nhỏ nhất của ảnh sạch: phương sai nhỏ nhất ở selfie là 72,6 và ở thẻ là 78,3, nên `MIN_BLUR_VAR = 60`; độ sáng ảnh sạch nằm trong [108,9; 172,7], nên cho phép 60 đến 200. Vì ngưỡng được đặt ngay trên tập sạch này nên ảnh sạch không bị gắn nhầm (0%) là hệ quả của cách chọn, chứ không phải bằng chứng ngưỡng đúng với ảnh mới; ta xem phần ảnh đã xuống cấp mà bị bắt:

```text
làm mờ Gauss      sigma    trung vị blur selfie   bị gắn mờ     trung vị blur thẻ   bị gắn mờ
                      0                   194.6          0%                 230.7          0%
                      1                   137.8          2%                 163.6          0%
                      2                    70.6         32%                  97.9         10%
                      3                    36.2         82%                  59.1         50%
                      4                    22.5        100%                  36.1         88%
đổi gamma         gamma    trung vị sáng selfie  gắn tối/cháy
                    0.5                   188.7      0% / 18%
                      1                   143.5       0% / 0%
                      2                    85.4      12% / 0%
                      3                    53.9      68% / 0%
                      4                    35.4      98% / 0%
```

Con số nói rõ một giới hạn: phương sai Laplacian phụ thuộc cả nội dung ảnh, nên một mặt có kính hay râu vốn nhiều cạnh hơn mặt trơn. Ở sigma 3 trung vị chỉ còn 36,2 nhưng vẫn có ảnh vượt 60 và đi qua (82% bị gắn, tức 18% lọt). Ngưỡng này bắt phần lớn ảnh mờ rõ, không bắt hết ảnh mờ vừa, và mờ nhẹ (sigma 1) đi qua gần như toàn bộ. Chỉ đặt ngưỡng cao hơn thì tăng cả số ảnh sạch bị gắn nhầm; chọn điểm nào là quyết định về chi phí của hàng đợi người duyệt, không phải chân lý.

## Gom lại: `check_portrait` và lý do chuyển người duyệt

`check_portrait(image, detector)` nối các bước và trả `FaceCheck(status, reasons, ...)`. Trạng thái dùng đúng tên trong vòng đời hồ sơ: `face_checked` nếu mọi bước qua, còn lại `needs_review` kèm một hoặc nhiều lý do (`no_face`, `multiple_faces`, `face_too_small`, `face_cut_off`, `tilted`, `blurry`, `too_dark`, `too_bright`).

```python
# faces.py (trích)
face = max(faces, key=lambda f: f.box[2] * f.box[3])   # mặt lớn nhất
x, y, w, h = face.box
if min(w, h) < MIN_FACE_PX:
    reasons.append("face_too_small")
if x < 0 or y < 0 or x + w > width or y + h > height:   # YuNet vẫn trả hộp thò ra ngoài khung
    reasons.append("face_cut_off")
roll = roll_degrees(face.landmarks)
if abs(roll) > MAX_ROLL_DEG:
    reasons.append("tilted")
quality = face_quality(align_face(bgr, face.landmarks))
```

Hàm không bao giờ trả «giả mạo» hay «sai người», và `to_bgr` từ chối ảnh rỗng, sai dtype hay số kênh lạ bằng `ValueError` thay vì để OpenCV lỗi khó đọc. Kết quả với mười ca dùng YuNet (cùng một thẻ hoặc một selfie) và một dòng thêm dùng FaceLandmarker:

```text
  thẻ sạch                face_checked  -                         roll -3.9
  thẻ mờ sigma 4          needs_review  blurry                    roll -2.2
  thẻ tối gamma 4         needs_review  too_dark                  roll -1.7
  thẻ không có ảnh        needs_review  no_face                   roll -
  hai người trong khung   needs_review  multiple_faces            roll -2.1
  selfie mặt 60 px        needs_review  face_too_small            roll 1.3
  selfie nghiêng 35 độ    needs_review  tilted                    roll 16.5
  ảnh đen hoàn toàn       needs_review  no_face                   roll -
  mặt bị cắt mép khung    needs_review  face_cut_off              roll 0.9
  ảnh xám 1 kênh          face_checked  -                         roll -3.8
  nghiêng 35 độ, dùng FaceLandmarker  needs_review  tilted (roll 35.4)
```

Hai dòng cuối nói lên điều của mục trước. Với ảnh nghiêng 35 độ YuNet báo 16,5 độ, thấp hơn thật hơn 18 độ; chỉ nhờ `MAX_ROLL_DEG = 15` đặt thấp mà ca này vẫn bị gắn `tilted`, trong khi FaceLandmarker báo 35,4 độ. Ngưỡng 15 thay vì 30 là hệ quả của đo đạc chứ không phải thói quen. Ca «mặt bị cắt mép» cho thấy một lỗi thật: ảnh mà khuôn mặt chỉ còn một nửa trong khung, YuNet vẫn trả hộp và đủ 5 landmark (hộp thò ra ngoài khung); lúc đầu lab để ca này đi qua, và chỉ khi có ca kiểm này mới thêm điều kiện hộp nằm trọn trong ảnh. Trên 40 thẻ sạch, cả 40 đạt `face_checked`. Ngưỡng cỡ mặt cũng có giá: `MIN_FACE_PX = 80` (cạnh ngắn của hộp) gắn `face_too_small` cho 31 trong 40 selfie mặt 80 px, dù YuNet tìm đủ 40/40 ảnh đó (lab in `{'face_too_small': 31, '(không lý do)': 9, 'blurry': 1}`); hộp YuNet thường hẹp hơn cạnh ô dán, nên một khuôn mặt «80 px» của ta thường có hộp dưới 80. Ngưỡng này phải chọn theo cỡ crop mà bước embedding cần, không phải theo con số tròn.

## Chạy và kiểm chứng

Từ thư mục gốc repository (lần đầu cần mạng để tải mô hình, bộ Olivetti và các gói Python):

```bash
uv run --project examples/toi-noi-python/48-face-detection-landmarks examples/toi-noi-python/48-face-detection-landmarks/main.py
uv run --project examples/toi-noi-python/48-face-detection-landmarks examples/toi-noi-python/48-face-detection-landmarks/main.py --verify
```

Lệnh thứ hai chạy lại toàn bộ rồi kiểm các bất biến đã in: số ảnh Olivetti, độ chính xác của phép đồng dạng, YuNet tìm đủ mặt trên thẻ và selfie, Haar mất ở 30 độ nghiêng còn YuNet thì không, BlazeFace short range mất ở mặt 80 px còn YuNet thì không, BlazeFace full range bắt mặt 80 px tốt hơn bản short range, sai số roll của FaceLandmarker dưới 3 độ, ngưỡng mờ nằm dưới mức nhỏ nhất của ảnh sạch, các ca `check_portrait` ở trên, ảnh đen, ảnh xám một kênh, và ba kiểu ảnh không hợp lệ bị từ chối. Dòng cuối của lần chạy mới nhất:

```text
OK 40/40
```

Không ca nào kiểm tốc độ tuyệt đối vì số đo thời gian phụ thuộc tải máy. Ảnh minh họa (`card_detections.png` vẽ hộp bốn bộ phát hiện và 5 landmark lên thẻ, `aligned_roll30.png` so hai crop căn chỉnh) nằm trong `out/`, đã gitignore.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| `AttributeError` ở `CascadeClassifier` hoặc `RuntimeError` khi nạp Haar | Bản OpenCV 5.x: bản chính bỏ class, bản contrib không kèm xml; lab chặn `opencv-contrib-python<5` |
| Haar không thấy mặt nghiêng | Đúng hành vi (0/40 ở 30 độ); dùng YuNet hoặc xoay ảnh thử nhiều góc |
| BlazeFace short range hay FaceLandmarker không thấy mặt nhỏ | Cỡ mặt trong khung (mặt 80 px: 8/40 và 4/40 có hộp); thử BlazeFace full range hoặc cắt vùng quan tâm rồi phóng lớn |
| Phương sai Laplacian của ảnh thẻ cao bất thường | Đang đo cả crop, có cạnh viền thẻ; đo vùng lõi |
| Crop căn chỉnh vẫn nghiêng | Landmark YuNet bão hòa ở góc lớn; đo lại roll sau căn chỉnh hoặc dùng landmark dày |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Olivetti là ảnh xám 64 × 64 của 40 người, hầu hết chụp thẳng trong phòng thí nghiệm; «thẻ» và «selfie» là ảnh ghép, nên các tỉ lệ trên đo hành vi của từng bước trên dữ liệu này, không phải tỉ lệ lỗi với ảnh điện thoại ngoài đời (ánh sáng ngược, khẩu trang, kính, nhiều màu da và tuổi). «Trúng» tính theo ô dán nên chỉ so sánh tương đối; YuNet là mặc định vì kết quả trên bộ ghép này, chưa kiểm bằng ảnh điện thoại thật. Số đo thời gian được đo khi máy rất bận và chỉ cho thứ bậc lớn. Ngưỡng 60, 60 đến 200, 15 độ và `MIN_FACE_PX` 80 lấy từ chính tập sạch này; dịch vụ thật phải chọn lại bằng ảnh của mình và theo dõi tỉ lệ ảnh bị chuyển người duyệt. Bài này chưa đánh giá chất lượng phát hiện theo nhóm người; việc đó thuộc bài 53.</p>
</div>

**Bài tập 1:** Hai mắt trên ảnh gốc cách nhau 160 px. Hệ số co của phép đồng dạng căn chỉnh về mẫu ArcFace 112 × 112 xấp xỉ bao nhiêu, và crop 112 × 112 phủ khoảng bao nhiêu px của ảnh gốc theo cạnh?

**Bài tập 2:** Hạ `MIN_BLUR_VAR` từ 60 xuống 30. Theo bảng sigma, tỉ lệ ảnh selfie bị gắn mờ ở sigma 3 thay đổi theo hướng nào, và điều đó đổi gì cho người duyệt?

<details><summary>Xem lời giải</summary><p><b>1.</b> Khoảng cách hai mắt của mẫu là √((73,5318 − 38,2946)² + (51,5014 − 51,6963)²) ≈ 35,24 px, nên hệ số co ≈ 35,24 / 160 ≈ <b>0,22</b>. Crop 112 px phủ 112 / 0,22 ≈ <b>509 px</b> của ảnh gốc theo mỗi cạnh: rộng hơn khuôn mặt nhiều, vì mẫu ArcFace chừa lề quanh mặt. <b>2.</b> Trung vị blur ở sigma 3 là 36,2, cao hơn 30, nên <b>dưới một nửa</b> ảnh còn bị gắn (so với 82% ở ngưỡng 60): hàng đợi người duyệt ngắn đi, nhưng nhiều ảnh mờ vừa lọt sang bước embedding hơn. Giảm ngưỡng đổi gánh nặng từ người duyệt sang độ tin cậy của điểm so khớp.</p></details>

Bài trước: [bài 47](/articles/python-luu-ket-qua-ocr-mongodb-pii-retention). Bài tiếp: [bài 49](/articles/python-face-embedding-cosine-threshold-far-frr).

## Tài liệu gốc

- [OpenCV Zoo: YuNet (face detection)](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet).
- [MediaPipe: Face Detector](https://ai.google.dev/edge/mediapipe/solutions/vision/face_detector).
- [MediaPipe: Face Landmarker](https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker).
- [scikit-learn: tập dữ liệu thực, mục Olivetti faces](https://scikit-learn.org/stable/datasets/real_world.html).
- [NumPy: numpy.linalg.svd](https://numpy.org/doc/stable/reference/generated/numpy.linalg.svd.html).
