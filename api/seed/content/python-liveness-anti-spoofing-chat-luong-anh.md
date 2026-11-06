---
slug: python-liveness-anti-spoofing-chat-luong-anh
title: "Liveness và chống giả mạo cơ bản: ảnh in, màn hình, mờ, tối"
summary: "Bài 50: kiểm chất lượng selfie (blur, sáng, tương phản, cỡ mặt) và dò mẫu lặp của màn hình, ảnh in bằng FFT. Nói rõ đây không phải liveness thật. Có lab chạy được và số đo thật."
category: programming
level: advanced
tags: [python, toi-noi, liveness, opencv, fft]
language: vi
---

Một tài xế mới của [Tới Nơi](/articles/du-an-mau-toi-noi) nộp ảnh selfie để đối chiếu với ảnh trên CCCD. Có ba kiểu ảnh làm bước so khớp khuôn mặt mất nghĩa: ảnh chụp trong phòng tối, ảnh nhòe vì tay run, và ảnh của người khác được chụp lại từ màn hình điện thoại hay một tờ giấy in. Hai kiểu đầu là lỗi của người dùng thật, chữa bằng cách yêu cầu chụp lại. Kiểu thứ ba là giả mạo và cần một người xem. Bài này đo cả hai bằng vài chục dòng NumPy và OpenCV, rồi nói rõ phần nó không làm được.

Đây là bài 50 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần biết ảnh là mảng và cách lọc ảnh (bài 30, [NumPy, Pillow và OpenCV](/articles/python-numpy-pillow-opencv-tien-xu-ly-anh)), và biết hộp bao khuôn mặt đến từ đâu ([bài 48](/articles/python-face-detection-landmarks-mediapipe-opencv)).

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Một người chụp lại ảnh của người khác từ màn hình điện thoại độ phân giải cao, ở nơi đủ sáng, ảnh nét. Bộ kiểm blur, độ sáng và tương phản của bài này sẽ cho qua hay chặn?</p>
<details><summary>Xem đáp án</summary><p><b>Cho qua.</b> Ảnh như vậy không có dấu hiệu nào mà các phép đo biết tìm. Trong lab, kiểu ảnh <code>screen_clean</code> chỉ bị gắn cờ 3,0%, thấp hơn cả mức gắn nhầm ảnh chụp thật (6,0%). Phần còn lại của bài giải thích vì sao, và nên làm gì với giới hạn đó.</p></details>
</div>

## Ba tầng khác nhau: chất lượng, dấu vết, liveness thật

Ba câu hỏi sau nghe giống nhau nhưng khác hẳn về độ khó.

- **Ảnh này có đủ tốt để so khớp không?** Mờ, tối, cháy sáng, mặt quá nhỏ. Đây là chất lượng ảnh, lỗi thường của người dùng thật, đo được bằng vài phép tính trên điểm ảnh. Câu trả lời dẫn tới «chụp lại».
- **Ảnh này có dấu vết của việc chụp lại không?** Lưới điểm ảnh của màn hình và hạt in của giấy tạo ra mẫu lặp mà ảnh chụp trực tiếp không có. Chỉ bắt được khi kẻ tấn công vô tình để lại dấu vết.
- **Có người thật đang đứng trước camera không?** Đây là *presentation attack detection* (PAD), bài toán của các nhà cung cấp chuyên: đo độ sâu, phản xạ, chuyển động, đưa ra thử thách như quay đầu, và dùng mô hình huấn luyện trên dữ liệu tấn công thật. Chúng được đánh giá theo chuẩn như ISO/IEC 30107-3 bằng hai tỉ lệ: APCER là tỉ lệ cuộc tấn công bị chấp nhận nhầm (tính riêng cho từng loại tấn công), BPCER là tỉ lệ yêu cầu hợp lệ bị từ chối nhầm.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 264" role="img" aria-label="Ba tầng: chất lượng ảnh và dấu vết giả mạo do bài này làm, liveness thật do nhà cung cấp chuyên làm; tấn công tiêm ảnh qua camera ảo nằm ngoài khả năng của xử lý ảnh."><defs><marker id="py50-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="24" width="270" height="156" rx="10" class="f-phys-soft s-phys"/><text x="36" y="56" class="t-b">Tầng 1 · Chất lượng ảnh</text><text x="36" y="86" class="t-small">blur, tối, cháy sáng, mặt quá nhỏ</text><text x="36" y="108" class="t-small">lỗi của người dùng thật</text><text x="36" y="130" class="t-small">kết luận: yêu cầu chụp lại</text><text x="36" y="160" class="t-small t-b">Bài này làm</text><path d="M290 102 L305 102" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-0-arrow)"/><rect x="305" y="24" width="270" height="156" rx="10" class="f-warn-soft s-warn"/><text x="321" y="56" class="t-b">Tầng 2 · Dấu vết giả mạo</text><text x="321" y="86" class="t-small">mẫu lặp của lưới điểm ảnh, hạt in</text><text x="321" y="108" class="t-small">chỉ thấy khi tấn công để lại dấu</text><text x="321" y="130" class="t-small">kết luận: đưa người duyệt</text><text x="321" y="160" class="t-small t-b">Bài này làm, rất yếu</text><path d="M575 102 L590 102" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-0-arrow)"/><rect x="590" y="24" width="270" height="156" rx="10" class="f-bg s-muted dash"/><text x="606" y="56" class="t-b">Tầng 3 · PAD thật</text><text x="606" y="86" class="t-small">độ sâu, phản xạ, chuyển động</text><text x="606" y="108" class="t-small">mô hình học từ dữ liệu tấn công</text><text x="606" y="130" class="t-small">đo bằng APCER và BPCER</text><text x="606" y="160" class="t-small t-b">Nhà cung cấp chuyên</text><rect x="20" y="200" width="840" height="44" rx="10" class="f-surface s-warn dash"/><text x="36" y="227" class="t-small">Tiêm ảnh qua camera ảo: pixel không còn dấu vết nào, phải chống ở thiết bị và ứng dụng, không phải ở ảnh.</text></svg>
</div>
<figcaption>Hình 1. Ba tầng với độ khó tăng dần. Lab chỉ làm hai tầng đầu, và tầng hai chỉ bắt được dấu vết do chính lab vẽ ra.</figcaption>
</figure>

Bài này làm tầng 1 và một phần rất nhỏ của tầng 2. Kết luận cao nhất mà code trả về là `ok`, nghĩa là «không phép đo nào báo», không bao giờ là «người thật». Vì vậy lab không có biến `is_live`, và kiểu `Verdict` chỉ có ba giá trị: `ok`, `recapture`, `review`. Với tấn công tiêm ảnh (virtual camera đẩy ảnh thẳng vào luồng), điểm ảnh không còn dấu vết nào; hướng chống nằm ở kiểm soát thiết bị và ứng dụng, không phải ở xử lý ảnh.

## Ảnh mô phỏng: lab dựng những gì

Lab không có camera, màn hình hay tờ giấy in thật. Mọi kiểu ảnh đều do `capture.py` sinh ra từ Olivetti, bộ 400 ảnh khuôn mặt xám 64 × 64 (40 người, 10 ảnh mỗi người) mà scikit-learn tải về lần đầu (tải về 4,26 MB, bản cache 1,4 MB, cần mạng, không commit). Mỗi khuôn mặt được phóng to 160 px, đặt giữa khung 320 × 320 có nền mềm, rồi đi qua «cảm biến». Bộ khuôn mặt Olivetti do AT&T Laboratories Cambridge thu thập; xem tài liệu scikit-learn. Hàm dưới đây (trích, bỏ chú thích kiểu) là toàn bộ đường đi của cảm biến:

```python
# capture.py
def to_camera(scene, rng, *, gain=1.0, blur=0.5):
    """Đường đi của cảm biến: gain phơi sáng, nhòe quang học, nhiễu cảm biến (sigma 2), rồi JPEG chất lượng 90."""
    img = cv2.GaussianBlur(scene * gain, (0, 0), blur)
    img = img + rng.normal(0, 2.0, img.shape)
    ok, buf = cv2.imencode(".jpg", np.clip(img, 0, 255).astype(np.uint8), [cv2.IMWRITE_JPEG_QUALITY, 90])
    assert ok
    return cv2.imdecode(buf, cv2.IMREAD_GRAYSCALE)
```

Chín kiểu ảnh khác nhau ở cách dựng cảnh và tham số đưa vào `to_camera`: `genuine` chỉ biến thiên phơi sáng và độ nhòe thường ngày; `blur` nhòe sigma 3 đến 4; `dark` nhân 0,3; `bright` nhân 2,0 rồi cắt ở 255; `small_face` đặt mặt 56 px; `screen_moire` và `screen_weak` nhân lưới điểm ảnh sâu 25 đến 40% hay 5 đến 12%; `print_halftone` thay ảnh bằng chấm in nghiêng 45 độ với mực không đen hẳn; `screen_clean` là «màn hình phân giải cao, không dấu vết», chỉ thêm gain 1,1 và nhòe nhẹ. Lưới màn hình là tích của hai dải tuần hoàn, lấy trung bình 3 × 3 mẫu con mỗi điểm ảnh (trích):

```python
# capture.py, screen_grid (trích)
u = (x * np.cos(angle) + y * np.sin(angle)) / pitch
v = (-x * np.sin(angle) + y * np.cos(angle)) / pitch
gap = (((u % 1) > 0.7) | ((v % 1) > 0.7)).astype(np.float32)
acc += 1 - depth * gap
```

Điều này quyết định cách đọc mọi con số phía sau. Ta đo chỉ số phản ứng với đúng những dấu vết mà ta đã vẽ vào, không đo khả năng chặn tấn công thật. Khuôn mặt phóng to từ 64 px vốn đã mềm, nên mức «nét» của ảnh genuine trong lab thấp hơn selfie thật.

## Chất lượng ảnh: sharpness, độ sáng, tương phản, cỡ mặt

Bốn chỉ số đầu đều tính trên hộp bao khuôn mặt, không phải cả khung, vì nền nét không cứu được khuôn mặt mờ. Trong lab, hộp là chỗ ta đặt mặt vào; ở hệ thống thật nó do bước detection của bài 48 trả về.

- **sharpness**: phương sai của Laplacian, thước đo mờ quen thuộc. Biên sắc làm Laplacian dao động mạnh, ảnh mờ thì phẳng.
- **brightness** là trung bình mức xám trong mặt; **contrast** là hiệu giữa phân vị 95 và 5.
- **face_ratio**: bề rộng mặt chia bề rộng khung, chặn mặt quá nhỏ để so khớp.

Bẫy nằm ở sharpness. Nhiễu cảm biến cũng làm Laplacian dao động, nên ảnh *mờ nhưng nhiều nhiễu* trông «nét». Lab đo trực tiếp (bước 1 của output) và hàm có tham số `smooth` để làm mượt Gauss nhẹ trước khi lấy Laplacian:

(Trích, bỏ chú thích kiểu và docstring.)

```python
# quality.py
def sharpness(img, box, smooth=1.0):
    crop = cv2.resize(face_crop(img, box).astype(np.float32), (128, 128), interpolation=cv2.INTER_AREA)
    if smooth > 0:
        crop = cv2.GaussianBlur(crop, (0, 0), smooth)
    return float(cv2.Laplacian(crop, cv2.CV_32F).var())
```

```text
== 1. Laplacian thuần có bị nhiễu đánh lừa không
  ảnh nét, không nhiễu : thuần   17.77 | làm mượt   6.64
  ảnh mờ, thêm nhiễu   : thuần   50.75 | làm mượt   1.08
```

Laplacian thuần cho ảnh mờ có nhiễu (50,75) gấp gần 3 lần ảnh nét sạch (17,77): kiểm ngược. Sau khi làm mượt, ảnh nét sạch (6,64) cao gấp hơn 6 lần ảnh mờ (1,08). Mọi phép đo trong bài dùng bản làm mượt.

**Ngưỡng từ đâu ra?** Không có ngưỡng «đúng» chung cho mọi camera, nên `calibrate` lấy chúng từ phân bố ảnh genuine: chặn dưới ở phân vị 1% cho sharpness, độ sáng và tương phản, chặn trên ở phân vị 99% cho độ sáng và mẫu lặp. Ta chủ động chấp nhận khoảng 1% mỗi phép đo gắn nhầm ảnh tốt. Riêng cỡ mặt tối thiểu 0,30 khung là chính sách cố định.

```python
# quality.py, calibrate (trích)
return Thresholds(
    sharpness_min=q([m.sharpness for m in genuine], quantile),
    brightness_min=q([m.brightness for m in genuine], quantile),
    brightness_max=q([m.brightness for m in genuine], 1 - quantile),
    contrast_min=q([m.contrast for m in genuine], quantile),
    face_ratio_min=min_face_ratio,
    moire_peak_max=q([m.moire_peak for m in genuine], 1 - quantile),
)
```

## Mẫu lặp trong phổ tần số

Ảnh chụp trực tiếp có phổ tần số trơn: năng lượng giảm dần khi tần số tăng, không có đỉnh nhọn riêng lẻ. Lưới điểm ảnh của màn hình và chấm in của halftone là mẫu lặp, nên phổ có những đỉnh nhọn ở tần số tương ứng, khi lệch bước với lưới điểm ảnh của camera còn sinh vân moiré. `moire_peak` đi năm bước (Hình 2): cắt một vùng vuông tối đa 128 px ở tâm mặt, *không thu nhỏ* vì thu nhỏ xóa mất mẫu mịn; nhân cửa sổ Hann để mép cắt không tạo vạch giả; FFT 2 chiều; chia công suất mỗi ô cho trung vị của vành tần số chứa nó rồi lấy đỉnh cao nhất ở dải 0,06 đến 0,5 chu kỳ trên điểm ảnh.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 236" role="img" aria-label="Năm bước đo mẫu lặp: cắt tâm mặt, cửa sổ Hann, FFT hai chiều, chia cho trung vị từng vành, lấy đỉnh cao nhất; ảnh chụp thường cho đỉnh thấp, màn hình và ảnh in cho đỉnh nhọn."><defs><marker id="py50-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="30" width="148" height="104" rx="10" class="f-phys-soft s-phys"/><text x="34" y="62" class="t-b">Cắt tâm mặt</text><text x="34" y="88" class="t-small">tối đa 128 x 128</text><text x="34" y="108" class="t-small">không thu nhỏ</text><path d="M168 82 L193 82" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-1-arrow)"/><rect x="193" y="30" width="148" height="104" rx="10" class="f-phys-soft s-phys"/><text x="207" y="62" class="t-b">Cửa sổ Hann</text><text x="207" y="88" class="t-small">làm mờ mép cắt</text><text x="207" y="108" class="t-small">tránh vạch giả</text><path d="M341 82 L366 82" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-1-arrow)"/><rect x="366" y="30" width="148" height="104" rx="10" class="f-pk-soft s-pk"/><text x="380" y="62" class="t-b">FFT 2 chiều</text><text x="380" y="88" class="t-small">công suất theo</text><text x="380" y="108" class="t-small">tần số (x, y)</text><path d="M514 82 L539 82" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-1-arrow)"/><rect x="539" y="30" width="148" height="104" rx="10" class="f-pk-soft s-pk"/><text x="553" y="62" class="t-b">Chia theo vành</text><text x="553" y="88" class="t-small">mỗi ô chia cho</text><text x="553" y="108" class="t-small">trung vị của vành</text><path d="M687 82 L712 82" class="f-none s-phys" stroke-width="2" marker-end="url(#py50-1-arrow)"/><rect x="712" y="30" width="148" height="104" rx="10" class="f-warn-soft s-warn"/><text x="726" y="62" class="t-b">Đỉnh cao nhất</text><text x="726" y="88" class="t-small">dải 0,06 đến 0,5</text><text x="726" y="108" class="t-small">lấy log10</text><rect x="20" y="162" width="410" height="52" rx="10" class="f-phys-soft s-phys"/><text x="36" y="184" class="t-small">Ảnh chụp trực tiếp: không có đỉnh nhọn</text><text x="36" y="203" class="t-small t-b">trung vị moire_peak 1,48</text><rect x="450" y="162" width="410" height="52" rx="10" class="f-warn-soft s-warn"/><text x="466" y="184" class="t-small">Lưới điểm ảnh, hạt in: đỉnh nhọn</text><text x="466" y="203" class="t-small t-b">trung vị moire_peak 3,23 đến 4,43</text></svg>
</div>
<figcaption>Hình 2. Đường đi của chỉ số mẫu lặp. Chia cho trung vị của từng vành giữ cho phổ ảnh tự nhiên, vốn giảm dần theo tần số, không bị tính nhầm là đỉnh.</figcaption>
</figure>

```python
# quality.py, moire_peak (trích)
radius = np.hypot(yy - side // 2, xx - side // 2) / side  # chu kỳ/điểm ảnh
ring = np.minimum((radius * side).astype(int), side // 2)
ratio = np.zeros_like(power)
for r in range(side // 2 + 1):
    sel = ring == r
    if sel.any():
        ratio[sel] = power[sel] / max(float(np.median(power[sel])), 1e-9)
band = (radius > 0.06) & (radius < 0.5)
return float(np.log10(ratio[band].max() + 1e-9))
```

Ảnh genuine của lab đã qua JPEG chất lượng 90 nên ngưỡng hiệu chỉnh đã gồm cả dấu vết nén. Số thật của từng kiểu nằm ở mục chạy bên dưới.

## Quyết định: ok, recapture hay review

`assess` gom năm chỉ số thành một trong ba kết luận. Mẫu lặp thắng chất lượng: nghi giả mạo thì người duyệt xem, bất kể ảnh còn mờ hay tối.

(Trích: ba nhánh chất lượng còn lại được lược.)

```python
# quality.py
def assess(m: Metrics, t: Thresholds) -> Assessment:
    quality = []
    if m.sharpness < t.sharpness_min:
        quality.append("blur")
    ...  # dark, bright, low_contrast, small_face tương tự
    spoof = ["moire_pattern"] if m.moire_peak > t.moire_peak_max else []
    if spoof:
        return Assessment(Verdict.REVIEW, tuple(spoof + quality))
    if quality:
        return Assessment(Verdict.RECAPTURE, tuple(quality))
    return Assessment(Verdict.OK, ())
```

Hai quy tắc vận hành đi kèm. Một: `reasons` chỉ ghi log nội bộ và hiển thị cho người duyệt. Với người dùng, `recapture` hiện thông báo chung («ảnh chưa đủ rõ, hãy chụp lại») và `review` không hiện lý do, vì mỗi cờ lộ ra là một gợi ý để kẻ tấn công né; mục chạy có số đo cho thấy việc né đơn giản đến mức nào. Hai: kết luận chuyển thành bước kế tiếp của hồ sơ KYC bằng `route`. `recapture` không được thử mãi: sau `max_attempts` lần vẫn bị gắn cờ, hồ sơ chuyển sang trạng thái `needs_review`; `review` chuyển thẳng. Người duyệt xem cùng kết quả so khớp và OCR trong luồng KYC ([bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric)); cờ là đầu vào cho họ, không phải quyết định từ chối.

```python
# quality.py
def route(verdict: Verdict, attempt: int, max_attempts: int = 3) -> Next:
    if verdict is Verdict.OK:
        return Next.PROCEED
    if verdict is Verdict.RECAPTURE and attempt < max_attempts:
        return Next.RETAKE
    return Next.NEEDS_REVIEW
```

## Chạy và kiểm chứng

Cần mạng ở lần chạy đầu để tải Olivetti, không cần Docker. Từ gốc worktree:

```bash
uv run --project examples/toi-noi-python/50-liveness-anti-spoofing examples/toi-noi-python/50-liveness-anti-spoofing/main.py --verify
```

Ngưỡng hiệu chỉnh trên ảnh 0 đến 4 của mỗi người (200 ảnh genuine), thử trên ảnh 5 đến 9 (200 ảnh mỗi kiểu). Output (trích; đủ cả chín kiểu ở bảng đầu):

```text
== 2. Ngưỡng hiệu chỉnh từ ảnh genuine (phân vị 1% / 99%)
  sharpness >= 3.21   brightness 96.0..180.8   contrast >= 71.0   face_ratio >= 0.30   moire_peak <= 1.96

== 3. Kết quả theo kiểu ảnh (200 ảnh mỗi kiểu, người đã gặp lúc hiệu chỉnh)
  kiểu              ok  recapture  review   gắn cờ   trung vị sharp / sáng / t.phản / moire
  genuine          188         10       2     6.0%    10.34  142.0  105.0  1.48
  blur               2        198       0    99.0%     1.52  137.6   92.0  1.37
  dark               0        200       0   100.0%     1.13   41.6   32.0  1.34
  bright             0        191       9   100.0%    19.30  238.7   82.0  1.51
  small_face         0        200       0   100.0%     5.40  135.7  100.1  1.32
  screen_moire       0          0     200   100.0%     8.85  114.7   93.0  3.93
  screen_weak        1          0     199    99.5%     9.16  131.0   99.0  3.23
  print_halftone     0          0     200   100.0%     9.13  139.2   95.0  4.43
  screen_clean     194          6       0     3.0%    11.09  153.5  116.0  1.45
  lý do gắn cờ (một ảnh có thể có nhiều lý do):
    genuine        {'bright': 4, 'moire_pattern': 2, 'low_contrast': 3, 'dark': 1, 'blur': 2}
    dark           {'blur': 199, 'dark': 200, 'low_contrast': 200}
    print_halftone {'moire_pattern': 200, 'low_contrast': 7}
    screen_clean   {'bright': 5, 'blur': 1}
```

Cách đọc. **Ảnh có lỗi chất lượng bị bắt gần hết**: `blur` 99,0%, `dark`, `bright`, `small_face` đều 100,0%. Điều này hiển nhiên: ngưỡng lấy từ ảnh genuine và các lỗi này được vẽ cách xa genuine. Ảnh tối còn bị gắn thêm cờ `blur` (199 trên 200): Laplacian là phép tuyến tính nên gain 0,3 làm phương sai co cỡ 0,09 lần (trung vị 1,13 so với 10,34 của genuine). Một ảnh có thể mang nhiều cờ, và cờ đầu tiên chưa chắc là nguyên nhân gốc. **Mẫu lặp do lab vẽ bị bắt hết**: `screen_moire`, `print_halftone` 100,0%, `screen_weak` 99,5%, trung vị `moire_peak` từ 3,23 đến 4,43 so với 1,48 của genuine. **Ảnh genuine bị gắn nhầm 6,0%** (12 trên 200): đó là giá của việc đặt các ngưỡng ở phân vị 1% hoặc 99%. **`screen_clean` chỉ 3,0%**: đúng như câu hỏi đầu bài, kiểu giả mạo không để lại dấu vết thì mọi phép đo ở đây mù.

Hai bước tiếp theo của output nói về độ vững của ngưỡng (bước 4 chạy lại với 400 ảnh genuine sinh mới nên không trùng 200 ảnh ở bảng trên):

```text
== 4. Ngưỡng chuyển sang người khác: ảnh genuine bị gắn cờ (400 ảnh, mỗi người 10 ảnh)
  chia theo thứ tự, hiệu chỉnh người 0-19, thử 20-39 :  28.0%
  chia theo thứ tự ngược, hiệu chỉnh 20-39, thử 0-19 :   8.5%
  200 cách chia ngẫu nhiên cùng người (5 ảnh hiệu chỉnh, 5 ảnh thử, đủ 40 người):
    trung vị   6.0%  p10   3.5%  p90  10.0%  lớn nhất  17.0%
  200 cách chia ngẫu nhiên rời người (20 người hiệu chỉnh, 20 người thử):
    trung vị   7.0%  p10   4.0%  p90  13.0%  lớn nhất  23.0%
  3 người có mặt tối nhất (độ sáng trung vị thấp nhất): [21, 33, 38] -> 3 trong 3 người thuộc nhóm 20-39

== 5. Chụp màn hình lưới rõ, tổng độ nhòe sigma tăng dần
  sigma 0.5: cờ moire 100.0%   gắn cờ bất kỳ lý do 100.0%
  sigma 1.0: cờ moire  93.0%   gắn cờ bất kỳ lý do  94.0%
  sigma 1.5: cờ moire   7.5%   gắn cờ bất kỳ lý do  33.5%
  sigma 2.0: cờ moire   0.5%   gắn cờ bất kỳ lý do  66.5%
  sigma 3.0: cờ moire   0.0%   gắn cờ bất kỳ lý do 100.0%
```

Bước 4: đừng suy kết luận từ một cách chia người. Cách chia theo thứ tự (hiệu chỉnh trên người 0 đến 19, thử trên 20 đến 39) cho 28,0% ảnh genuine bị gắn cờ, nhưng đảo chiều chỉ 8,5%. Qua 200 cách chia ngẫu nhiên, nếu hiệu chỉnh và thử đều có đủ 40 người (5 ảnh mỗi bên) thì trung vị 6,0%, p90 10,0%; nếu rời người (20 người hiệu chỉnh, 20 người thử) thì trung vị 7,0%, p90 13,0%, lớn nhất 23,0%. Vậy trường hợp điển hình là 6 đến 7%, còn 28,0% là ngoại lệ do cách chia: ba người có mặt tối nhất (21, 33 và 38) đều rơi vào nhóm 20 đến 39, nên nhiều khả năng ngưỡng học từ nhóm kia đặt độ sáng và độ nét tối thiểu quá cao cho họ (lab chưa tách riêng nguyên nhân). Bài học: hiệu chỉnh trên ít người thì vài người cực đoan kéo lệch ngưỡng, và ai bị gắn cờ nhiều hơn thì phải chụp lại nhiều hơn. Kiểm theo nhóm là chủ đề của bài 53 (trên dữ liệu công khai chỉ có nhóm giả định, nên bài đó không kết luận về nhóm người thật). Bước 5: cùng một ảnh màn hình lưới rõ, chỉ cần tổng độ nhòe sigma 1,5 là cờ mẫu lặp còn 7,5% và gắn cờ bất kỳ còn 33,5%; nhòe nhiều hơn thì cờ `blur` bắt lại (100,0% ở sigma 3,0). Có một khoảng nhòe vừa phải mà ảnh lọt: tại sigma 1,5 là 66,5% ảnh qua.

```text
== 6. Thời gian mỗi ảnh 320x320 (lượt nhanh nhất trong 7)
  sharpness 0.41 ms | moire_peak 3.56 ms | cả 5 chỉ số 4.74 ms; load average 136.30 160.72 188.48

OK 20/20
```

Thời gian đo trên máy macOS Apple Silicon đang rất bận (load average 136 lúc đo), nên chỉ đọc được bậc lớn: vài mili giây một ảnh, phần FFT là phần tốn nhất. `--verify` chỉ kiểm bất biến chứ không có ngưỡng tốc độ. Trong 20 ca có hai ca kiểm giới hạn: `screen_clean` phải gần như không bị bắt, và cách chia theo thứ tự phải cao gấp đôi trung vị các cách chia rời người.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| Ảnh mờ nhưng vẫn qua kiểm blur | Có dùng Laplacian thuần? Nhiễu cảm biến làm phương sai tăng, hãy làm mượt trước (`smooth`) |
| Ảnh genuine bị gắn cờ nhiều ở một nhóm người dùng | Ngưỡng hiệu chỉnh từ nhóm nhỏ hay thiên về một nhóm; hiệu chỉnh lại trên dữ liệu thật rộng hơn và đo gắn nhầm theo nhóm |
| `moire_peak` cao với mọi ảnh | Cắt ảnh có bị thu nhỏ trước FFT không, ảnh có nén JPEG khác mức đã hiệu chỉnh không |
| Ảnh màn hình sáng, nét vẫn qua | Đây là giới hạn của heuristic, không phải lỗi; cần PAD thật |
| Cờ mẫu lặp biến mất khi ảnh hơi nhòe | Mẫu mịn bị ống kính làm mất; đừng coi vắng cờ là bằng chứng ảnh thật |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Ảnh là mô phỏng của chính lab: màn hình và ảnh in là những mẫu tuần hoàn lý tưởng, khuôn mặt Olivetti xám 64 × 64 phóng to. Màn hình, giấy in và camera thật cho vân phức tạp hơn, và kẻ tấn công chọn được góc, khoảng cách, độ nhòe. Vì thế 100,0% ở trên chỉ chứng minh chỉ số phản ứng đúng với dấu vết ta vẽ, không phải tỉ lệ chặn tấn công thật. Một hệ thống tin được cần nhà cung cấp PAD đã được đánh giá độc lập (xem trang FATE PAD của NIST trong phần tài liệu), tín hiệu chủ động như thử thách quay đầu, và kiểm soát thiết bị chống camera ảo. Dữ liệu khuôn mặt của người dùng là dữ liệu cá nhân nhạy cảm: nhắc tới lưu trữ, đồng ý và quyền xóa ở bài 53. Lab dùng ảnh công khai và dữ liệu tổng hợp; không có ảnh người nào được commit.</p>
</div>

**Bài tập 1:** Trong `main.py`, gọi `calibrate(cal_metrics, quantile=0.05)` thay vì mặc định 0,01 và đo lại tỉ lệ gắn cờ trên ảnh genuine, `blur` và `screen_clean`. Có nên siết ngưỡng như vậy không?

**Bài tập 2:** Đặt `smooth=0` trong `sharpness`, hiệu chỉnh lại và đo tỉ lệ ảnh `blur` bị gắn cờ `blur`. Nó còn bắt được ảnh mờ không, và vì sao bước 1 của output vẫn cảnh báo?

<details><summary>Xem lời giải</summary><p><b>Bài tập 1.</b> Chạy trên đúng bộ ảnh của lab: ở mức 0,05, ảnh genuine bị gắn cờ tăng từ 6,0% lên 23,5%, <code>blur</code> từ 99,0% lên 100,0%, còn <code>screen_clean</code> từ 3,0% lên 15,0%. Siết ngưỡng chỉ làm người dùng thật phải chụp lại nhiều hơn; kiểu giả mạo không để lại dấu vết vẫn qua 85,0%. Ngưỡng chất lượng không thay được PAD. <b>Bài tập 2.</b> Với Laplacian thuần và ngưỡng hiệu chỉnh lại (17,05), 97,5% ảnh <code>blur</code> vẫn bị gắn cờ, nên trong lab nhiễu sigma 2 chưa đủ để phá thước đo. Bước 1 dùng nhiễu sigma 3 trên ảnh mờ sigma 3,5, và ở đó Laplacian thuần cho 50,75, cao hơn ảnh nét sạch 17,77. Camera thật ở nơi thiếu sáng thường nhiễu hơn lab, và đó là lý do mặc định là <code>smooth=1.0</code>.</p></details>

Bài trước: [bài 49](/articles/python-face-embedding-cosine-threshold-far-frr). Bài tiếp: [bài 51](/articles/python-vector-search-pgvector-faiss-trung-tai-xe).

## Tài liệu gốc

- [NumPy: Discrete Fourier Transform (numpy.fft)](https://numpy.org/doc/stable/reference/routines.fft.html).
- [NumPy: numpy.hanning](https://numpy.org/doc/stable/reference/generated/numpy.hanning.html).
- [scikit-learn: fetch_olivetti_faces](https://scikit-learn.org/stable/modules/generated/sklearn.datasets.fetch_olivetti_faces.html).
- [NIST: Face Analysis Technology Evaluation (FATE) PAD](https://pages.nist.gov/frvt/html/frvt_pad.html).
- [Python 3.14: enum.StrEnum](https://docs.python.org/3.14/library/enum.html).
