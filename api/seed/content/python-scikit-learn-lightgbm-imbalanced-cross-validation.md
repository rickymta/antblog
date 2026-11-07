---
slug: python-scikit-learn-lightgbm-imbalanced-cross-validation
title: "scikit-learn và LightGBM: huấn luyện trên dữ liệu mất cân bằng"
summary: "Bài 57: huấn luyện bộ chấm điểm gian lận bằng scikit-learn Pipeline và LightGBM: cross-validation theo thời gian, class weight, calibration, ngưỡng theo chi phí, lưu mô hình bằng joblib kèm metadata. Đo lặp trên 3 seed."
category: programming
level: advanced
tags: [python, toi-noi, fraud, scikit-learn, lightgbm]
language: vi
---

Rule engine ở [bài 56](/articles/python-rule-engine-scoring-strategy-explainability) cho điểm đọc được lý do, nhưng bỏ sót nhiều chuyến mà dấu vết nằm ở tổ hợp nhiều tín hiệu yếu. Mô hình học trên 30 đặc trưng của bài 55 là bước tiếp theo, và «chạy được trên notebook» chưa phải thứ [Tới Nơi](/articles/du-an-mau-toi-noi) đưa lên production. Service fraud-scoring cần huấn luyện lại định kỳ, lưu thành file, nạp lại ở nơi phục vụ, và trả ra một con số mà người duyệt và người vận hành đọc được. Trên dữ liệu 98 % chuyến sạch, ba câu hỏi nảy ra ngay: chia dữ liệu thế nào để số đo đáng tin, có cần «cân bằng» hai lớp không, và con số mô hình trả về có phải xác suất không.

Đây là bài 57 của lộ trình **Python nâng cao qua Tới Nơi**. Bạn cần quen luật chấm điểm ở bài 56, 30 đặc trưng ở [bài 55](/articles/python-feature-engineering-tu-event-kafka) và PR-AUC ở [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric). Dữ liệu là tổng hợp nên số đo chỉ nói về bộ này; điểm rủi ro là đầu vào cho người duyệt, không phải quyết định khóa tài khoản. LightGBM có wheel cho Python 3.14 (kiểm ngày 06/11/2026, bản 4.7.0); riêng macOS cần thêm `brew install libomp`. Mọi bước dùng nhãn **cuối cùng**, trừ một bước nói rõ; kết luận về thứ bậc được lặp trên ba seed của bộ sinh.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ở train, số chuyến sạch gấp 48,7 lần số chuyến gian lận. Ta đặt <code>scale_pos_weight</code> bằng 48,7 để «cân bằng» hai lớp. So với không đặt trọng số, PR-AUC trên valid tăng, giảm hay gần như không đổi? Và xác suất trung bình mô hình trả về so với tỉ lệ gian lận thật 1,71 % của valid ra sao?</p>
<details><summary>Xem đáp án</summary><p><b>PR-AUC gần như không đổi</b> (0,885 ở cả hai), nhưng <b>xác suất bị đẩy lên</b>: trung bình 0,0143 khi không trọng số, 0,0339 khi có, gần gấp đôi tỉ lệ thật. Số chuyến vượt ngưỡng 0,5 tăng từ 848 lên 1.800 trong khi precision ở 0,5 tụt từ 0,927 xuống 0,566. Trọng số lớp không làm mô hình xếp hạng giỏi hơn; nó dịch xác suất, và mục «Trọng số lớp» đo từng bước.</p></details>
</div>

## Pipeline: chọn đặc trưng theo tên, rồi mô hình

Mô hình huấn luyện xong phải đi cùng bước chuẩn bị đầu vào của nó, nếu không nơi phục vụ sẽ đưa nó dữ liệu khác với lúc học. `Pipeline` của scikit-learn gói hai thứ vào một đối tượng, và `ColumnTransformer` ở đầu pipeline chọn **đúng 30 cột theo tên**, bỏ cột khác (`model.py`, trích):

```python
def make_pipeline(estimator, *, scale: bool = False, features: Sequence[str] = FEATURES) -> Pipeline:
    numeric = (
        Pipeline([("log1p", FunctionTransformer(np.log1p)), ("scale", StandardScaler())]) if scale else "passthrough"
    )
    select = ColumnTransformer([("features", numeric, list(features))], remainder="drop", verbose_feature_names_out=False)
    return Pipeline([("select", select), ("model", estimator)])
```

`scale=True` thêm `log1p` và chuẩn hóa cho mô hình tuyến tính; cây quyết định không cần vì `log1p` đơn điệu. Hồi quy logistic qua pipeline này là mốc so sánh của bài, và cũng là lúc kiểm pipeline giữ lời hứa «chọn theo tên» (trích):

```text
hồi quy logistic (log1p + StandardScaler, 30 đặc trưng, 0.4 giây): PR-AUC 0.836, ROC-AUC 0.991 trên valid
đưa nguyên bảng 35 cột (có trip_id, nhãn): kết quả giống hệt = True; đảo thứ tự cột đặc trưng: giống hệt = True
bỏ cột speed_kmh: ValueError: columns are missing: {'speed_kmh'}
```

Đưa cả bảng 35 cột hay đảo thứ tự cột đều cho dự đoán giống hệt, và thiếu một đặc trưng thì lỗi to chứ không đoán. Hai tính chất đó chặn lỗi khó thấy nhất của phục vụ mô hình: cột lệch vị trí vì ai đó đổi thứ tự trong truy vấn, hay nhãn lọt vào đầu vào.

## Cross-validation theo thời gian

Cross-validation tách train thành nhiều phần, huấn luyện trên phần này, đo trên phần kia, để có nhiều số đo thay vì một. Với dữ liệu có thời gian, phần đo phải nằm sau phần huấn luyện. `time_folds` tạo các fold mở rộng dần, bỏ hai ngày sát cửa sổ test:

```python
def time_folds(day, *, n_splits=4, test_days=5, gap_days=2, end_day=30):
    for i in range(n_splits):
        start = end_day - (n_splits - i) * test_days
        yield np.flatnonzero(day < start - gap_days), np.flatnonzero((day >= start) & (day < start + test_days))
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Trục 60 ngày. Hàng trên: mô hình gốc huấn luyện ngày 0 đến 22, bộ hiệu chỉnh ngày 22 đến 30, ngày 30 đến 40 bỏ trống chờ nhãn, valid ngày 40 đến 60 chỉ để đo. Bốn hàng dưới: mỗi fold của cross-validation theo thời gian có cửa sổ test 5 ngày, bỏ 2 ngày sát test và train mọi ngày trước đó."><defs><marker id="py57-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="40" y="40" width="293" height="44" rx="6" class="f-phys-soft s-phys"/><text x="50" y="66" class="t-small">gốc: huấn luyện, ngày 0 đến 22</text><rect x="333" y="40" width="107" height="44" rx="6" class="f-warn-soft s-warn"/><text x="343" y="58" class="t-small">hiệu chỉnh</text><text x="343" y="74" class="t-small">ngày 22-30</text><rect x="440" y="40" width="133" height="44" rx="6" class="f-none s-warn dash"/><text x="450" y="66" class="t-small">chờ nhãn, bỏ trống</text><rect x="573" y="40" width="267" height="44" rx="6" class="f-pk-soft s-pk"/><text x="583" y="66" class="t-small">valid: chỉ đo, ngày 40 đến 60</text><text x="34" y="102" class="t-small">0</text><text x="327" y="102" class="t-small">22</text><text x="434" y="102" class="t-small">30</text><text x="567" y="102" class="t-small">40</text><text x="834" y="102" class="t-small">60</text><text x="40" y="26" class="t-b">Mô hình cuối cùng</text><text x="40" y="136" class="t-b">Bốn fold cross-validation trên train (trước khi chạm valid)</text><rect x="40" y="150" width="107" height="26" rx="4" class="f-phys-soft s-phys"/><text x="48" y="167" class="t-small">train 0-8</text><rect x="147" y="150" width="27" height="26" rx="4" class="f-none s-warn dash"/><rect x="173" y="150" width="67" height="26" rx="4" class="f-pk-soft s-pk"/><text x="181" y="167" class="t-small">test</text><text x="250" y="167" class="t-small">26.693 train, 16.542 test</text><rect x="40" y="186" width="173" height="26" rx="4" class="f-phys-soft s-phys"/><text x="48" y="203" class="t-small">train 0-13</text><rect x="213" y="186" width="27" height="26" rx="4" class="f-none s-warn dash"/><rect x="240" y="186" width="67" height="26" rx="4" class="f-pk-soft s-pk"/><text x="248" y="203" class="t-small">test</text><text x="317" y="203" class="t-small">43.295 train, 16.740 test</text><rect x="40" y="222" width="240" height="26" rx="4" class="f-phys-soft s-phys"/><text x="48" y="239" class="t-small">train 0-18</text><rect x="280" y="222" width="27" height="26" rx="4" class="f-none s-warn dash"/><rect x="307" y="222" width="67" height="26" rx="4" class="f-pk-soft s-pk"/><text x="315" y="239" class="t-small">test</text><text x="383" y="239" class="t-small">59.976 train, 16.591 test</text><rect x="40" y="258" width="307" height="26" rx="4" class="f-phys-soft s-phys"/><text x="48" y="275" class="t-small">train 0-23</text><rect x="347" y="258" width="27" height="26" rx="4" class="f-none s-warn dash"/><rect x="373" y="258" width="67" height="26" rx="4" class="f-pk-soft s-pk"/><text x="381" y="275" class="t-small">test</text><text x="450" y="275" class="t-small">76.528 train, 16.818 test</text><text x="40" y="318" class="t-small">Nét đứt: hai ngày bỏ trống sát test. Mỗi fold chỉ train trên quá khứ; nhãn trong lab là nhãn cuối cùng.</text></svg>
</div>
<figcaption>Hình 1. Bố trí theo thời gian của bài: mô hình gốc, bộ hiệu chỉnh, khoảng bỏ trống và valid chỉ để đo; bốn fold cross-validation chỉ train trên quá khứ và bỏ hai ngày sát test.</figcaption>
</figure>

Hai ngày bỏ trống **không** làm nhãn chín: chúng nhỏ hơn mọi trung vị độ trễ nhãn của bài 54 (3, 6, 8 và 16 ngày), và lab dùng nhãn cuối cùng ở mọi fold. Nếu mỗi fold chỉ được biết nhãn có sẵn lúc bắt đầu cửa sổ test, ta chỉ biết một phần gian lận của phần train (trích):

```text
  test từ ngày 10: biết 179 trong 401 chuyến gian lận (45%)
  test từ ngày 15: biết 439 trong 717 chuyến gian lận (61%)
  test từ ngày 20: biết 723 trong 1,148 chuyến gian lận (63%)
  test từ ngày 25: biết 1,019 trong 1,449 chuyến gian lận (70%)
```

Khoảng bỏ trống chỉ tránh việc đo sát ngay sau phần train. Mục «Calibration» đo hậu quả của việc dùng nhãn biết sớm thay vì nhãn cuối cùng.

Lab dùng bốn fold để chọn số vòng cây. Ta huấn luyện **một lần** 400 vòng rồi chấm PR-AUC ở từng số vòng bằng `num_iteration`:

```python
pipe = make_pipeline(make_lgbm(n_estimators=400)).fit(xtr[tr_i], ytr[tr_i])
z = pipe[:-1].transform(xtr[te_i])          # chỉ bước chọn cột
scores = [pr(ytr[te_i], pipe[-1].predict_proba(z, num_iteration=k)[:, 1]) for k in TREE_GRID]
```

```text
 test (ngày)    train    test  gian lận     25     50    100    200    300    400
   10 đến 15   26,693  16,542     2.01%  0.801  0.807  0.807  0.805  0.801  0.800
   15 đến 20   43,295  16,740     2.19%  0.801  0.817  0.822  0.826  0.826  0.824
   20 đến 25   59,976  16,591     2.09%  0.796  0.811  0.818  0.818  0.815  0.815
   25 đến 30   76,528  16,818     2.54%  0.854  0.860  0.864  0.862  0.859  0.854
  trung bình                             0.813  0.824  0.828  0.828  0.825  0.823
số vòng chọn theo trung bình PR-AUC các fold: 100; từ 100 đến 400 vòng trung bình chỉ chênh 0.005, nên không đáng tinh chỉnh thêm
```

Đường cong phẳng từ 100 vòng, nên lab chọn 100 (giá trị mặc định của `make_lgbm()`); chọn số vòng bằng valid thì valid không còn là mẫu chưa từng thấy. PR-AUC giữa các fold dao động từ 0,807 đến 0,864 dù mô hình không đổi. Để biết lượng train đóng góp bao nhiêu, ta đổi **một** biến: giữ train cố định ở fold đầu (26.693 chuyến) và chấm cả bốn cửa sổ:

```text
                             ngày 10   ngày 15   ngày 20   ngày 25
train cố định                  0.807     0.758     0.731     0.763
train mở rộng (như trên)       0.807     0.822     0.818     0.864
```

Train mở rộng cải thiện dần (0,807 lên 0,864) còn train cố định tụt xuống 0,731 đến 0,763, nên lượng train là yếu tố chính của khoảng dao động. Vì sao mô hình cố định tụt theo thời gian thì lab chưa đo nguyên nhân. Một lần chia là ít: hai mô hình chênh vài phần nghìn trên một lần chia chưa chắc khác nhau.

Cross-validation ngẫu nhiên (`StratifiedKFold`, 5 fold) cho trung bình 0,831, so với 0,828 của bốn fold theo thời gian (hai dòng trong đầu ra của lab). Hai số này khác nhau ở **hai** biến, cỡ train (80.017 so với 26.693 đến 76.528 chuyến) và cách chia, nên không so thẳng để kết luận về cách chia; và cũng không so thẳng được với PR-AUC 0,885 trên valid ở mục sau, vì fold train nhỏ hơn.

## Trọng số lớp: đổi một biến

Câu hỏi đầu bài đo ba giá trị của `scale_pos_weight` (1, 7 và 48,7), huấn luyện trên toàn bộ train, giữ nguyên mọi thứ khác, rồi chấm trên valid:

```text
 scale_pos_weight  PR-AUC  ROC-AUC  logloss   Brier  p trung bình      cờ ở 0.5 precision  recall
              1.0   0.885    0.995   0.0195  0.0050        0.0143       848     0.927   0.689
              7.0   0.887    0.994   0.0196  0.0052        0.0213     1,093     0.815   0.781
             48.7   0.885    0.994   0.0333  0.0095        0.0339     1,800     0.566   0.893
```

PR-AUC đi ngang ở 0,885 đến 0,887: trọng số lớp không làm mô hình xếp hạng chuyến tốt hơn. Cái đổi là **thang xác suất**: `p` trung bình tăng từ 0,0143 lên 0,0339 khi tỉ lệ thật là 0,0171, logloss xấu đi từ 0,0195 lên 0,0333 và Brier từ 0,0050 lên 0,0095. Ở ngưỡng 0,5 số chuyến bị gắn cờ tăng từ 848 lên 1.800, recall từ 0,689 lên 0,893 và precision tụt từ 0,927 xuống 0,566. Lab kiểm xem hiệu ứng đó có phải chỉ là hạ ngưỡng không: lấy từ mô hình **không** trọng số đúng chừng ấy chuyến điểm cao nhất (trích):

```text
  scale_pos_weight=  7.0 gắn cờ 1,093 chuyến (ngưỡng 0.5): precision 0.815, recall 0.781; không trọng số, 1,093 chuyến điểm cao nhất: precision 0.818, recall 0.784
  scale_pos_weight= 48.7 gắn cờ 1,800 chuyến (ngưỡng 0.5): precision 0.566, recall 0.893; không trọng số, 1,800 chuyến điểm cao nhất: precision 0.568, recall 0.896
```

Hai cách cho precision và recall khác nhau không quá 0,01 trong lần đo này. Muốn bắt nhiều hơn, hạ ngưỡng trên mô hình không trọng số, và giữ xác suất trung thực. Lab chỉ đo ba giá trị, một seed, trên bộ dữ liệu này; trọng số có thể có ích khi lớp hiếm quá ít để cây học, và `is_unbalance` hay `class_weight` là hai cách khai báo khác mà lab không đo.

## Calibration: p có đáng gọi là xác suất không

Ngưỡng theo chi phí (mục sau) chỉ có nghĩa khi «p = 0,2» nghĩa là khoảng 20 % số chuyến như thế gian lận thật. **Calibration** là bước đo và sửa điều đó. Lab chia train làm hai theo thời gian: mô hình gốc học trên ngày dưới 22, bộ hiệu chỉnh là ngày 22 đến 30. `fit_calibrated` (`model.py`) làm cả hai bước; `FrozenEstimator` bảo `CalibratedClassifierCV` không huấn luyện lại mô hình gốc:

```python
def fit_calibrated(x, y, day, *, cal_start_day=22, method="sigmoid", **lgbm_overrides):
    a, b = np.flatnonzero(day < cal_start_day), np.flatnonzero(day >= cal_start_day)
    base = make_pipeline(make_lgbm(**lgbm_overrides)).fit(x[a], y[a])
    cal = CalibratedClassifierCV(FrozenEstimator(base), method=method).fit(x[b], y[b])
    return base, cal
```

Hai phần liền nhau, không có khoảng bỏ, nên chuyến cuối của phần gốc có nhãn chưa chín và bộ hiệu chỉnh thừa hưởng điều đó. Tỉ lệ gian lận cũng khác nhau: 1,87 % ở phần gốc, 2,39 % ở bộ hiệu chỉnh (639 trên 26.737 chuyến) và 1,71 % ở valid. Phương pháp **sigmoid** được chọn trước khi đọc valid: bộ hiệu chỉnh chỉ có 639 chuyến gian lận, hàm hai tham số đơn điệu ít rủi ro hơn bậc thang của isotonic. Bảng dưới in cả hai để thấy cái giá của isotonic:

```text
mô hình                             PR-AUC  logloss   Brier  p trung bình     ECE  số mức điểm
scale_pos_weight=1.0, chưa hiệu chỉnh   0.884   0.0214  0.0054        0.0128  0.0043       66,250
scale_pos_weight=1.0, sigmoid        0.884   0.0186  0.0048        0.0172  0.0019       66,250
scale_pos_weight=1.0, isotonic       0.874   0.0183  0.0048        0.0171  0.0008           93
scale_pos_weight=48.7, chưa hiệu chỉnh   0.877   0.0245  0.0069        0.0258  0.0087       66,702
scale_pos_weight=48.7, sigmoid       0.877   0.0188  0.0049        0.0169  0.0011       66,702
scale_pos_weight=48.7, isotonic      0.864   0.0203  0.0049        0.0170  0.0009           84
```

ECE là sai lệch trung bình có trọng số giữa `p` và tỉ lệ thật trong năm khoảng xác suất. Sigmoid (hai tham số) kéo `p` trung bình về 0,0172, gần tỉ lệ thật 0,0171, giảm logloss từ 0,0214 xuống 0,0186 mà PR-AUC giữ nguyên 0,884, vì sigmoid là hàm đơn điệu của điểm. Isotonic hiệu chỉnh sát hơn (ECE 0,0008) nhưng chỉ còn 93 mức điểm trên 66.704 chuyến: nhiều chuyến bằng điểm nhau và PR-AUC tụt xuống 0,874. Sigmoid cũng sửa được mô hình có trọng số (ECE từ 0,0087 xuống 0,0011), tức xác suất lấy lại được dù đã có trọng số; vì trọng số không cải thiện PR-AUC, lab không dùng nó.

Bảng độ tin cậy trên valid cho mô hình không trọng số, mỗi dòng là một khoảng `p` (số chuyến, `p` trung bình, tỉ lệ gian lận thật):

```text
        khoảng p  chưa hiệu chỉnh:      p    thật      sigmoid:      p    thật
  [0.000, 0.005)            63,373 0.0004  0.0005        59,434 0.0011  0.0002
  [0.005, 0.020)             1,355 0.0105  0.0539         3,962 0.0090  0.0053
  [0.020, 0.100)               899 0.0416  0.1657         1,691 0.0481  0.0716
  [0.100, 0.500)               307 0.2254  0.4658           734 0.2126  0.2371
  [0.500, 1.000)               770 0.9241  0.9636           883 0.9211  0.9196
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 340" role="img" aria-label="Độ tin cậy trên valid, trục log. Mỗi điểm là một khoảng xác suất dự đoán: hoành độ là p dự đoán trung bình, tung độ là tỉ lệ gian lận thật. Mô hình không trọng số chưa hiệu chỉnh nằm trên đường chéo, tức dự đoán thấp hơn thật. Mô hình trọng số 48,7 chưa hiệu chỉnh nằm dưới đường chéo, tức dự đoán cao hơn thật. Mô hình không trọng số sau sigmoid nằm sát đường chéo."><defs></defs><rect x="90" y="30" width="270" height="270" class="f-bg s-line"/><path d="M90 300 L90 30" class="f-none s-line"/><text x="80" y="316" class="t-small">0,001</text><path d="M90 300 L360 300" class="f-none s-line"/><text x="46" y="304" class="t-small">0,001</text><path d="M180 300 L180 30" class="f-none s-line"/><text x="170" y="316" class="t-small">0,01</text><path d="M90 210 L360 210" class="f-none s-line"/><text x="46" y="214" class="t-small">0,01</text><path d="M270 300 L270 30" class="f-none s-line"/><text x="260" y="316" class="t-small">0,1</text><path d="M90 120 L360 120" class="f-none s-line"/><text x="46" y="124" class="t-small">0,1</text><path d="M360 300 L360 30" class="f-none s-line"/><text x="350" y="316" class="t-small">1</text><path d="M90 30 L360 30" class="f-none s-line"/><text x="46" y="34" class="t-small">1</text><path d="M90 300 L360 30" class="f-none s-muted dash" stroke-width="1.5"/><text x="155" y="336" class="t-small">p dự đoán trung bình trong khoảng</text><text x="20" y="22" class="t-small">tỉ lệ gian lận thật</text><path d="M181.9 144.2 L235.7 100.3 L301.8 59.9 L356.9 31.4" class="f-none s-phys" stroke-width="2"/><circle cx="181.9" cy="144.2" r="4" class="f-phys s-phys"/><circle cx="235.7" cy="100.3" r="4" class="f-phys s-phys"/><circle cx="301.8" cy="59.9" r="4" class="f-phys s-phys"/><circle cx="356.9" cy="31.4" r="4" class="f-phys s-phys"/><path d="M180.0 272.9 L237.0 196.8 L306.1 119.6 L354.3 44.9" class="f-none s-warn" stroke-width="2"/><circle cx="180.0" cy="272.9" r="4" class="f-warn s-warn"/><circle cx="237.0" cy="196.8" r="4" class="f-warn s-warn"/><circle cx="306.1" cy="119.6" r="4" class="f-warn s-warn"/><circle cx="354.3" cy="44.9" r="4" class="f-warn s-warn"/><path d="M175.9 234.8 L241.4 133.1 L299.5 86.3 L356.8 33.3" class="f-none s-pk" stroke-width="2"/><circle cx="175.9" cy="234.8" r="4" class="f-pk s-pk"/><circle cx="241.4" cy="133.1" r="4" class="f-pk s-pk"/><circle cx="299.5" cy="86.3" r="4" class="f-pk s-pk"/><circle cx="356.8" cy="33.3" r="4" class="f-pk s-pk"/><circle cx="440" cy="80" r="5" class="f-phys s-phys"/><text x="456" y="84" class="t-b">không trọng số, chưa hiệu chỉnh</text><text x="456" y="102" class="t-small">thật cao hơn dự đoán: dự đoán quá thấp</text><circle cx="440" cy="144" r="5" class="f-warn s-warn"/><text x="456" y="148" class="t-b">scale_pos_weight 48,7, chưa hiệu chỉnh</text><text x="456" y="166" class="t-small">thật thấp hơn dự đoán: dự đoán quá cao</text><circle cx="440" cy="208" r="5" class="f-pk s-pk"/><text x="456" y="212" class="t-b">không trọng số, sau sigmoid</text><text x="456" y="230" class="t-small">gần đường chéo, lệch nhất ở khoảng p thấp</text><text x="440" y="270" class="t-small">Đường nét đứt: dự đoán bằng thật.</text><text x="440" y="288" class="t-small">Khoảng p dưới 0,005 không vẽ.</text></svg>
</div>
<figcaption>Hình 2. Độ tin cậy trên valid (trục log). Mô hình không trọng số, chưa hiệu chỉnh, dự đoán quá thấp ở các khoảng giữa; trọng số 48,7 làm dự đoán quá cao; sigmoid đưa điểm về gần đường chéo.</figcaption>
</figure>

Mô hình chưa hiệu chỉnh sai theo hướng **thấp**: ở khoảng 0,1 đến 0,5, `p` trung bình 0,2254 nhưng 46,6 % số chuyến gian lận thật. Mô hình có trọng số sai theo hướng ngược lại: ở khoảng 0,5 đến 1, `p` trung bình 0,864 mà chỉ 68,3 % gian lận thật (lab in thêm một dòng cho mô hình này, bài lược bớt). Không có quy tắc «mô hình luôn quá tự tin».

Toàn bộ các bước trên dùng nhãn cuối cùng. Bước tiếp theo của lab dùng **nhãn biết tại ngày 40** (chuyến gian lận có `label_known_at` chưa tới thì coi là sạch) cho cả huấn luyện lẫn hiệu chỉnh, vẫn đo trên valid bằng nhãn cuối:

```text
gian lận biết được vào ngày 40: phần gốc 1,308/1,372 (95%), phần hiệu chỉnh 522/639 (82%)
nhãn để huấn luyện + hiệu chỉnh     PR-AUC  p trung bình     ECE  gắn cờ  chi phí kỳ vọng (VND)  collusion PR-AUC
nhãn cuối cùng                       0.884        0.0172  0.0019   1,203             18,585,730             0.592
nhãn biết ở ngày 40                  0.876        0.0143  0.0052     959             20,049,819             0.478
```

Với nhãn biết sớm, `p` trung bình xuống 0,0143 (gian lận chưa có nhãn bị dạy là sạch), ECE từ 0,0019 lên 0,0052, chi phí lên 20,05 triệu VND (so với 18,59 triệu) và PR-AUC của `collusion`, loại có độ trễ nhãn dài nhất, tụt từ 0,592 xuống 0,478. Đây là một seed; hướng thì khớp với bài 54.

## Ngưỡng theo chi phí, và mô hình bắt được gì

Mô hình chi phí của lab (từ bài 54, đã sửa cho đúng): **chi phí = 8.000 VND × số chuyến gắn cờ + (tiền chuyến + 20.000 VND) × số chuyến gian lận bỏ sót**. Phí duyệt tính cho mọi chuyến gắn cờ, kể cả gian lận thật. Hai giả định: gắn cờ chặn hết thiệt hại (người duyệt không sai), và `amount_vnd` đại diện cho thiệt hại (với `promo_abuse`, thiệt hại thật chỉ là phần giảm giá). Gắn cờ một chuyến tốn 8.000 chắc chắn và tránh được `p × (tiền + 20.000)` kỳ vọng, nên với `p` đã hiệu chỉnh quyết định có công thức:

```python
def expected_cost_flag(p, amount_vnd):
    """Gắn cờ khi p * (tiền chuyến + 20.000) > 8.000. Chỉ có nghĩa khi p đã hiệu chỉnh."""
    return np.asarray(p) * (np.asarray(amount_vnd) + HANDLING_COST_VND) > REVIEW_COST_VND
```

Ngưỡng này thay đổi theo giá chuyến và không cần tinh chỉnh trên dữ liệu. Lab chọn thêm một ngưỡng chung trên **bộ hiệu chỉnh** (ngày 22 đến 30, không đụng valid), rồi báo cáo mọi chính sách trên toàn bộ valid (66.704 chuyến, 1.141 gian lận):

```text
chính sách                                            gắn cờ    TP    FP    FN   chi phí (VND)
không gắn cờ gì                                            0     0     0  1141      54,626,442
ngưỡng 0.5                                               883   812    71   329      21,522,235
ngưỡng chọn trên bộ hiệu chỉnh (0.149)                 1,329   946   383   195      19,004,109
chi phí kỳ vọng, p đã hiệu chỉnh                       1,203   918   285   223      18,585,730
chi phí kỳ vọng, p CHƯA hiệu chỉnh                       927   836    91   305      20,173,490
người duyệt chỉ xem 667 chuyến (1 %)                     667   656    11   485      27,719,126
hồi quy logistic, ngưỡng trên bộ hiệu chỉnh (0.069)    1,188   878   310   263      21,134,366
```

Quy tắc chi phí kỳ vọng (18,59 triệu VND) và ngưỡng chọn bằng dữ liệu (19,00 triệu) cách nhau 2 %: đừng đọc thành thứ bậc. Cùng quy tắc trên `p` **chưa** hiệu chỉnh đắt hơn 8,5 % ở seed 42, nhưng con số đó không vững. Lab chạy lại cả chuỗi (cùng siêu tham số, cùng cách chia) trên ba seed của bộ sinh:

```text
 seed   gian lận train/hiệu chỉnh/valid  PR toàn train  PR gốc<22  ECE chưa  ECE sigmoid  chi phí sigmoid  chi phí chưa   chênh
   42             2.01% / 2.39% / 1.71%          0.885      0.884    0.0043       0.0019       18,585,730    20,173,490    8.5%
    7             2.00% / 2.19% / 1.93%          0.882      0.870    0.0040       0.0021       21,674,654    22,214,297    2.5%
  123             1.96% / 2.45% / 1.91%          0.892      0.885    0.0039       0.0021       20,988,993    22,123,359    5.4%
```

Ở cả ba seed sigmoid giảm ECE (khoảng 0,004 xuống 0,002) và `p` chưa hiệu chỉnh cho chi phí cao hơn, nhưng mức chênh dao động từ 2,5 đến 8,5 %: hiệu chỉnh thường có ích chứ không hứa một con số. Cột «PR gốc<22» còn cho biết giữ tám ngày cho hiệu chỉnh tốn bao nhiêu so với học trên toàn bộ train: 0,001, 0,012 và 0,007 PR-AUC, tức vài phần nghìn đến hơn một phần trăm. Công suất cố định 1 % (667 chuyến) tốn 27,72 triệu vì còn 485 chuyến gian lận lọt; xem 1 % chuyến là quá ít khi valid có 1,71 % gian lận.

Tách theo loại, LightGBM đã hiệu chỉnh so với hồi quy logistic (cùng train ngày dưới 22), trên ba seed:

```text
loại                 seed 42          seed 7        seed 123
ghost_trip     0.872 / 0.838   0.828 / 0.813   0.857 / 0.834
gps_spoof      0.770 / 0.623   0.756 / 0.641   0.795 / 0.504
promo_abuse    0.731 / 0.696   0.744 / 0.732   0.751 / 0.727
collusion      0.592 / 0.401   0.379 / 0.227   0.613 / 0.213
```

Mô hình hơn rõ ở `gps_spoof` và `collusion` ở cả ba seed; ở `ghost_trip` và `promo_abuse` chênh nhỏ (seed 7, `promo_abuse`: 0,744 so với 0,732). `collusion` cũng dao động mạnh giữa các seed (0,379 đến 0,613), nên một con số đơn lẻ về loại này không đáng tin. Tám đặc trưng có tổng gain lớn nhất là `dev_customers_7d` (16,9 %), `drv_jump_speed_kmh` (14,6 %), `route_ratio`, `start_pickup_gap_km`, `duration_s`, `dev_newuser_7d`, `pair_trips_7d`, `discount_ratio`. Gain là mức mô hình dùng đặc trưng khi huấn luyện, không phải lý do của từng dự đoán; đó là việc của SHAP, bài 61.

## Lưu mô hình: joblib, metadata và những gì có thể sai

`joblib.dump` ghi mô hình đã hiệu chỉnh ra file, nhưng người nạp còn cần biết nó học từ đâu, dùng đặc trưng nào, đạt bao nhiêu và chạy được trên phiên bản thư viện nào. `save_model` ghi `model.joblib` và `model_meta.json`, mỗi file qua file tạm rồi `os.replace` nên người đọc không thấy file dở. Hai file vẫn là hai thao tác: nếu tiến trình chết giữa chừng, `load_model` thấy sha256 lệch. Lab thử cả ghi và nạp:

```text
đã ghi out/model.joblib (345 KiB) và out/model_meta.json; model_version = fraud-lgbm-1796fba0
khóa của metadata: artifact, created_at, data, decision, feature_list, libraries, metrics_valid, metrics_valid_by_type, model, model_version, reference_distribution
nạp lại: 66,704 dự đoán giống hệt lúc huấn luyện = True; score_one(chuyến đầu valid) = 0.001522 (hàng loạt: 0.001522); ngưỡng trong meta 0.149
```

Metadata có `model_version` (`fraud-lgbm-` cộng tám ký tự đầu của sha256), `feature_list` theo đúng thứ tự, khoảng ngày của từng phần dữ liệu và loại nhãn, siêu tham số, ngưỡng và giả định chi phí, `metrics_valid` và `metrics_valid_by_type`, phiên bản thư viện, và `reference_distribution`: với mỗi đặc trưng, các mốc phân vị và tỉ lệ dòng mỗi khoảng trên dữ liệu huấn luyện, để bài 60 tính PSI mà không cần giữ lại dữ liệu. `load_model` trả `ScoringModel`:

- `predict_proba(df)` chọn cột theo `meta["feature_list"]` chứ không theo `FEATURES` hiện tại, nên mô hình cũ với bộ đặc trưng khác vẫn nạp được; cột thừa bị bỏ.
- `score_one(dict)` và `predict_proba` từ chối `None`, `NaN`, vô cực, chuỗi và thiếu khóa bằng `ModelInputError`, thay vì để LightGBM âm thầm coi `NaN` là «thiếu».
- `threshold` (ngưỡng chọn trên bộ hiệu chỉnh) và `flag(p, amount)` (quy tắc chi phí kỳ vọng).
- `booster` và `preprocess` cho SHAP ở bài 61; `contributions(df)` cho tổng mỗi dòng khớp điểm thô (log-odds) **trước hiệu chỉnh**, nên SHAP giải thích điểm thô chứ không phải `p`.

```text
đóng góp từng đặc trưng (pred_contrib): tổng mỗi dòng khớp điểm thô trước hiệu chỉnh, lệch lớn nhất 1.0e-13 trên 200 chuyến (đây là log-odds của LightGBM, không phải p)
score_one từ chối đầu vào xấu (ModelInputError):
  None: đặc trưng drv_jump_km không phải số: None
  NaN: đặc trưng drv_jump_km là NaN hoặc vô cực
  chuỗi: đặc trưng drv_jump_km không phải số: 'abc'
  thiếu khóa: thiếu đặc trưng: speed_kmh
```

Mọi lỗi nạp đều là `ModelLoadError`, kể cả JSON hỏng, meta thiếu khóa và joblib cắt cụt; phiên bản scikit-learn và LightGBM phải **cùng minor** (lệch patch chỉ cảnh báo):

```text
nạp file bị sửa (mọi lỗi đều là ModelLoadError):
  đổi 1 bit trong model.joblib: sha256 của model.joblib không khớp meta: file hỏng hoặc nhầm file
  bỏ 1 đặc trưng khỏi meta: danh sách đặc trưng trong meta khác danh sách pipeline đã học
  meta ghi lightgbm 3.3.5: lightgbm lúc lưu 3.3.5 khác lúc nạp 4.7.0 (khác minor): pickle không bảo đảm tương thích
  meta là JSON hỏng: model_meta.json không đọc được: JSONDecodeError
  meta thiếu khóa artifact: model_meta.json thiếu khóa: ['artifact']
  model.joblib cắt cụt, sha256 sửa cho khớp: model.joblib không nạp được: EOFError:
```

Một cảnh báo không thể bỏ qua: **joblib dùng pickle, và nạp pickle của người lạ là chạy mã của họ**. sha256 trong metadata chỉ phát hiện file hỏng hoặc nhầm file; metadata nằm cạnh file nên kẻ ghi được file cũng ghi được metadata. Chỉ nạp mô hình do chính pipeline của mình tạo, lấy từ nơi bạn kiểm soát quyền ghi. Nếu phải nhận file từ nguồn kém tin cậy hơn, lưu thêm `booster.save_model(...)` (định dạng văn bản của LightGBM) hoặc dùng skops thay vì pickle. Bài 60 làm registry phiên bản mô hình; kiểm soát ai được ghi vào kho là việc của hạ tầng, không nằm trong code lab.

## Chạy và kiểm chứng

Không cần Docker. Cần uv và Python 3.14; macOS cần `brew install libomp`; lần đầu cần mạng để tải thư viện và dựng đặc trưng từ event cho ba seed (khoảng nửa phút mỗi seed, lưu `out/table*.parquet`). Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced examples/toi-noi-python/57-sklearn-lightgbm-imbalanced/main.py
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced examples/toi-noi-python/57-sklearn-lightgbm-imbalanced/main.py --verify
uv run --project examples/toi-noi-python/57-sklearn-lightgbm-imbalanced pytest examples/toi-noi-python/57-sklearn-lightgbm-imbalanced -q
```

Dòng cuối của `--verify` (bước cross-validation mất 342 giây khi load average 1 phút là 429,34 trên 10 lõi) và của `pytest`:

```text
Kiểm chứng: OK 45/45
25 passed in 27.25s
```

`--verify` kiểm 45 bất biến: pipeline bỏ cột thừa và không phụ thuộc thứ tự cột; fold theo thời gian không train trên tương lai; train cố định tụt theo thời gian còn train mở rộng thì không; trọng số lớp gần như không đổi PR-AUC nhưng đẩy xác suất trung bình lên; sigmoid giảm logloss và ECE ở cả ba seed; nhãn biết ở ngày 40 làm chi phí cao hơn; ngưỡng chi phí rẻ hơn không làm gì; mô hình lưu nạp lại cho dự đoán giống hệt; đầu vào xấu và file sửa bị từ chối. Có ca sai thì lab in tên ca, `FAIL n/m` (n là số ca đạt) và thoát với mã 1, kể cả khi không có `--verify`. Nó không ghim số tuyệt đối hay tốc độ.

| Hiện tượng | Kiểm tra trước |
|---|---|
| `Library not loaded: @rpath/libomp.dylib` khi import lightgbm trên macOS | Thiếu OpenMP: `brew install libomp` |
| Số chuyến vượt 0,5 tăng vọt sau khi thêm `scale_pos_weight` | Trọng số lớp dịch xác suất; hạ ngưỡng trên mô hình không trọng số hoặc hiệu chỉnh lại |
| PR-AUC cross-validation thấp hơn valid | Cỡ train của fold nhỏ hơn; so cùng cỡ train |
| `ModelLoadError` về phiên bản thư viện | Huấn luyện lại bằng đúng phiên bản của nơi phục vụ; đừng bật `strict=False` ở production |
| `ModelInputError: ... None` hoặc `NaN` khi dự đoán | Nguồn đặc trưng trả giá trị thiếu; xử lý ở nguồn (fallback ở bài 59), không điền 0 |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Dữ liệu là tổng hợp (200.000 chuyến, 60 ngày) nên PR-AUC 0,885 không đại diện cho dữ liệu thật; ta tự sinh dữ liệu nên mô hình học tốt không chứng minh gì ngoài bộ này. Bộ sinh có hai tạo tác ảnh hưởng mọi valid của phần 07: chuyến sạch dùng <code>NEWUSER50</code> giảm từ 7,8 % xuống 0,9 % theo thời gian vì quần thể khách gần như cố định, và tài khoản gian lận không dùng mã nào ngoài <code>NEWUSER50</code> của <code>promo_abuse</code>, nên <code>cust_promo_share_30d</code> lớn hơn 0 là dấu hiệu «sạch». Kết luận thứ bậc được đo trên ba seed nhưng với một cách chia và siêu tham số gần như mặc định (31 lá, tốc độ học 0,1). Mô hình chi phí giả định gắn cờ chặn hết thiệt hại và dùng <code>amount_vnd</code> làm thiệt hại. Nhãn là nhãn cuối cùng, trừ bước so sánh với nhãn biết ở ngày 40. Lab chưa đo độ trễ dự đoán (bài 59), chưa có registry và drift (bài 60), chưa giải thích từng dự đoán (bài 61). Đã chạy với Python 3.14.7, lightgbm 4.7.0, scikit-learn 1.9.1, numpy 2.5.4, polars 2.1.0, joblib 1.6.0.</p>
</div>

**Bài tập:** (1) Theo quy tắc chi phí kỳ vọng, ngưỡng `p` để gắn cờ một chuyến 30.000 VND là bao nhiêu? Một chuyến 120.000 VND thì sao, và vì sao ngưỡng đổi theo giá chuyến? (2) Bộ hiệu chỉnh có 2,39 % gian lận. Nếu ngoài đời tỉ lệ đó tăng gấp đôi, thành 4,78 %, một chuyến mô hình cho `p = 0,1` thực sự đáng nghi bao nhiêu, giả sử mô hình vẫn xếp hạng như cũ và chỉ tỉ lệ nền đổi?

<details><summary>Xem lời giải</summary><p>(1) Gắn cờ tốn 8.000 chắc chắn, bỏ sót tốn tiền chuyến cộng 20.000, nên gắn cờ khi p × (tiền + 20.000) &gt; 8.000, tức p &gt; 8.000 / (tiền + 20.000). Chuyến 30.000 VND: 8.000 / 50.000 = <b>0,16</b>. Chuyến 120.000 VND: 8.000 / 140.000 = <b>0,057</b>. Chuyến đắt thì bỏ sót tốn hơn nên chấp nhận gắn cờ ở xác suất thấp hơn. Ngưỡng chung chọn trên bộ hiệu chỉnh (0,149 trong lab) là trung bình hóa của việc đó. (2) Odds ban đầu: 0,0239 / 0,9761 = 0,0245; odds mới: 0,0478 / 0,9522 = 0,0502, gấp <b>2,05</b> lần. Với <code>p = 0,1</code> (odds 0,111), odds mới là 0,111 × 2,05 = 0,228, tức p ≈ <b>0,19</b>. Mô hình đã hiệu chỉnh trên tỉ lệ cũ sẽ đánh giá thấp mọi chuyến; sửa bằng hiệu chỉnh lại trên dữ liệu gần, hoặc nhân odds theo tỉ lệ nền mới, và theo dõi tỉ lệ nền để biết lúc nào phải làm (bài 60).</p></details>

Bài trước: [bài 56](/articles/python-rule-engine-scoring-strategy-explainability). Bài tiếp: [bài 58](/articles/python-networkx-graph-cau-ket-tai-xe-khach).

## Tài liệu gốc

- [scikit-learn: ColumnTransformer](https://scikit-learn.org/stable/modules/generated/sklearn.compose.ColumnTransformer.html).
- [scikit-learn: Probability calibration](https://scikit-learn.org/stable/modules/calibration.html).
- [scikit-learn: FrozenEstimator](https://scikit-learn.org/stable/modules/generated/sklearn.frozen.FrozenEstimator.html).
- [scikit-learn: Model persistence](https://scikit-learn.org/stable/modules/model_persistence.html).
- [LightGBM: Parameters](https://lightgbm.readthedocs.io/en/stable/Parameters.html).
- [LightGBM: LGBMClassifier](https://lightgbm.readthedocs.io/en/stable/pythonapi/lightgbm.LGBMClassifier.html).
