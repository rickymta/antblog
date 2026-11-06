---
slug: python-face-matching-danh-gia-bias-privacy-consent
title: "Đánh giá, thiên lệch, quyền riêng tư và sự đồng ý"
summary: "Bài 53: đo FAR/FRR theo nhóm kèm khoảng tin cậy và nói thẳng điều Olivetti không đo được; lưu embedding thay ảnh, đồng ý, retention, quyền xóa; hệ thống chỉ gợi ý, người quyết định. Lab chạy thật."
category: programming
level: advanced
tags: [python, toi-noi, face-matching, bias, privacy]
language: vi
---

Service so khớp khuôn mặt của [Tới Nơi](/articles/du-an-mau-toi-noi) chạy ổn trên bộ ảnh thử: cùng người thì điểm cao, khác người thì điểm thấp. Trước khi bật nó cho hồ sơ tài xế thật, có ba câu hỏi mà một con số «độ chính xác» không trả lời. Nó sai bao nhiêu, và sai nhiều hơn với ai? Ta giữ gì về khuôn mặt của tài xế, trong bao lâu, và họ rút lại được không? Và khi điểm nói «khớp», ai là người quyết định hồ sơ được duyệt? Đây là bài 53 của lộ trình **Python nâng cao qua Tới Nơi**, bài cuối của phần so khớp khuôn mặt. Bạn cần nắm FAR, FRR và ngưỡng ở [bài 49](/articles/python-face-embedding-cosine-threshold-far-frr) và service ở [bài 52](/articles/python-face-match-grpc-service-batching-model-loading).

Bài này không chứng minh hệ thống «công bằng» hay «không thiên lệch», vì với dữ liệu ta có, điều đó không làm được. Nó chỉ ra cách đo để một chênh lệch giữa các nhóm có nghĩa, và dựng các cơ chế (lưu tối thiểu, đồng ý, hết hạn, xóa, người quyết định) thành code có kiểm chứng. Phần pháp lý thuộc về pháp chế của Tới Nơi; bài chỉ nói ở mức nguyên tắc kỹ thuật, không phải tư vấn pháp lý.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta chia 40 người của bộ Olivetti thành hai nhóm 20 người, A và B, rồi đo ở cùng một ngưỡng. Tỉ lệ nhận nhầm (FAR) của nhóm A là 1,6%, của nhóm B là 0,2%. Có nên kết luận hệ thống thiên lệch bất lợi cho nhóm A không?</p>
<details><summary>Xem đáp án</summary><p><b>Chưa nên.</b> Khoảng tin cậy 95% của hai nhóm chồng lên nhau, và nếu chia 40 người ngẫu nhiên thành hai nửa thì cứ 10 lần có khoảng 1 lần chênh lệch FAR lớn bằng hoặc hơn chênh lệch ta thấy. Hơn nữa một cách chia chưa nói được gì: học ngưỡng ở nhóm A rồi đo ở nhóm B cho FAR 0,1%, còn học ở B rồi đo ở A cho 3,9%. Bài chạy đúng các phép thử này.</p></details>
</div>

## Đo trước, chia nhóm sau

Mọi so sánh theo nhóm đứng trên một phép đo chung. Lab dùng đúng đường ống của bài 49 và 52: 400 ảnh Olivetti (40 người, mỗi người 10 ảnh, xám 64×64, do AT&T Laboratories Cambridge thu thập, xem tài liệu scikit-learn), YuNet căn chỉnh về crop 112×112, SFace cho vector 128 chiều. Ảnh YuNet không lấy được mặt bị loại và đếm riêng; lần chạy này không ảnh nào bị loại, nhưng đó là kết quả chứ không phải điều bảo đảm. Mô hình là SFace (OpenCV Zoo ghi Apache 2.0) và YuNet (MIT); giấy phép của InsightFace và lý do bài không dùng nó nằm ở bài 49.

Ta lập mọi cặp ảnh: 1.800 cặp cùng người và 78.000 cặp khác người, điểm là cosine. Ta chấp nhận khi điểm không nhỏ hơn ngưỡng; **FAR** là tỉ lệ cặp khác người bị chấp nhận (nhận nhầm), **FRR** là tỉ lệ cặp cùng người bị từ chối (bỏ sót). Đo trên từng cặp ảnh nên đúng hơn là FMR/FNMR, tên bạn sẽ gặp ở báo cáo NIST; FAR/FRR của cả hệ thống còn tính ảnh không lấy được mặt. Chọn ngưỡng là chọn điểm đánh đổi giữa hai lỗi. Output của lab (trích):

```text
400 ảnh 64x64 của 40 người; YuNet lấy được mặt ở 400 ảnh, không lấy được ở 0 ảnh (bị loại và đếm riêng, không lùi âm thầm sang resize)
1800 cặp cùng người, 78000 cặp khác người; cosine trung bình: cùng người 0.813, khác người 0.241
mục tiêu FAR 1.0%: ngưỡng 0.479 -> FAR 1.0%, FRR 0.5%
mục tiêu FAR 0.1%: ngưỡng 0.567 -> FAR 0.1%, FRR 2.9%
```

Ngưỡng 0,567 và FRR 2,9% ở FAR 0,1% khớp bảng của bài 49 cho SFace trên ảnh căn chỉnh, vì cùng đường ống; con số chỉ so được khi cùng cách làm (bài 49 ghi cùng mô hình chỉ resize, không căn chỉnh, cho ngưỡng 0,750 và FRR 52,3%). Đây là số của riêng bộ dữ liệu này, không phải chất lượng trên ảnh selfie thật.

## Theo nhóm: đo được gì, không đo được gì

Câu hỏi «sai nhiều hơn với ai» cần nhãn nhóm. Olivetti **không có** nhãn nhân khẩu học (giới, tuổi, sắc tộc), và ta không suy ra chúng từ khuôn mặt: việc đoán thuộc tính nhạy cảm của một người từ ảnh vừa không đáng tin, vừa là một xử lý dữ liệu mới cần cơ sở riêng. Vì vậy các nhóm trong lab là **giả định**: nhóm A là người số 0 đến 19, nhóm B là người số 20 đến 39. Chúng chỉ cho ta thấy cơ chế đo, và qua đó cũng thấy giới hạn của việc đo.

Một cặp thuộc nhóm g khi cả hai ảnh thuộc g. Hai bẫy khi tính khoảng tin cậy. Thứ nhất, các cặp không độc lập: 19.000 cặp khác người trong một nhóm dùng lại chỉ 200 ảnh của 20 người, nên đơn vị độc lập là **người** và lab lấy lại mẫu (bootstrap) theo người, 2.000 lần. Thứ hai, một ngưỡng chung có thể cho FAR khác nhau giữa các nhóm chỉ vì may rủi của mẫu. Lab chia 40 người ngẫu nhiên thành hai nửa 20/20 hai nghìn lần để xem chênh lệch tự nhiên lớn cỡ nào, và học ngưỡng theo cả hai chiều. Output (trích):

```text
ngưỡng chung 0.479; khoảng trong ngoặc: 95%, lấy lại mẫu theo người có hoàn lại, 2000 lần
A (người 0-19)    19000 cặp khác người,  900 cặp cùng người | FAR   1.6% [0.2%, 4.0%] | FRR   0.8% [0.0%, 2.1%]
B (người 20-39)   19000 cặp khác người,  900 cặp cùng người | FAR   0.2% [0.0%, 0.4%] | FRR   0.2% [0.0%, 0.7%]
hiệu A - B: FAR +1.44 điểm %, FRR +0.6 điểm %
2000 lần chia NGẪU NHIÊN 20/20: hiệu FAR trong khoảng [-1.68, +1.68] điểm % (95%), hiệu FRR trong khoảng [-1.0, +1.0]
tỉ lệ lần chia ngẫu nhiên có |hiệu| lớn bằng hiệu quan sát: FAR 0.10, FRR 0.62
  học ở A (ngưỡng 0.498; đo lại ở A: FAR 1.0%, FRR 0.8%) -> đo ở B: FAR 0.1%, FRR 0.8%
  học ở B (ngưỡng 0.439; đo lại ở B: FAR 1.0%, FRR 0.0%) -> đo ở A: FAR 3.9%, FRR 0.1%
1000 cách chia ngẫu nhiên rời người, học ngưỡng FAR 1% ở một nửa, đo ở nửa kia: FAR đo được trung vị 1.1%, khoảng 95% [0.1%, 4.0%]; FRR trung vị 0.3%, khoảng 95% [0.0%, 1.6%]
```

Nhóm A có FAR 1,6% và nhóm B 0,2%. Hiệu +1,44 điểm nằm trong khoảng ±1,68 của các lần chia ngẫu nhiên nhưng không xa mép: cứ 10 lần chia ngẫu nhiên có 1 lần chênh lớn bằng hoặc hơn (với FRR là 62%). Kết luận trung thực: với 20 người mỗi nhóm, bộ đo này **không đủ mạnh** để phân biệt «có chênh lệch thật» với «nhiễu lấy mẫu», và điều đó không có nghĩa là không có thiên lệch. Hai chiều học ngưỡng cho thấy một cách chia không đủ: học ở A đo ở B ra FAR 0,1%, học ở B đo ở A ra 3,9%. Qua 1.000 cách chia rời người, FAR đo trên nửa chưa thấy có trung vị 1,1% nhưng khoảng 95% rộng từ 0,1% tới 4,0%: ngưỡng chọn rồi đo ngay trên tập đó cho FAR lạc quan.

Nhóm thứ hai đo được trên ảnh, không phải trên người: độ sáng trung bình của mỗi ảnh, chia ba phần bằng nhau (tối, vừa, sáng). Đây là một lát cắt chất lượng ảnh, loại lát cắt bạn dùng được khi chưa có nhãn nhân khẩu học.

```text
độ sáng trung bình mỗi ảnh (0..1): nhỏ nhất 0.382, lớn nhất 0.725; ranh giới ba phần 0.534 và 0.578; khoảng tin cậy: 1000 lần lấy lại mẫu theo người
tối (133 ảnh)      8314 cặp khác người,  464 cặp cùng người | FAR   0.3% [0.0%, 1.1%] | FRR   0.4% [0.0%, 1.3%]
vừa (133 ảnh)      8408 cặp khác người,  370 cặp cùng người | FAR   1.1% [0.1%, 3.0%] | FRR   0.0% [0.0%, 0.0%]
sáng (134 ảnh)     8499 cặp khác người,  412 cặp cùng người | FAR   1.8% [0.2%, 5.1%] | FRR   1.2% [0.0%, 3.8%]
```

Nhóm sáng có FAR 1,8%, cao nhất ba nhóm, nhưng khoảng [0,2%, 5,1%] rộng gấp nhiều lần chênh lệch. Bảng như vậy là lý do để **thu thêm dữ liệu** hoặc theo dõi lát cắt này khi vận hành, không phải bằng chứng để kết luận. Ngưỡng riêng theo nhóm là quyết định chính sách với đánh đổi riêng (đối xử khác nhau giữa người dùng), không phải thứ code tự quyết.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 300" role="img" aria-label="Ảnh selfie được giữ ngắn hạn và mã hóa tới khi có quyết định, mô hình biến nó thành 128 số, và bảng face_template giữ embedding cùng phiên bản mô hình và hạn lưu. Chỉ khi có đồng ý còn hiệu lực bảng mới được ghi. Từ bảng gốc, ba bản sao dẫn xuất gồm chỉ mục tìm kiếm, bản sao lưu, nhật ký và cache cũng phải bị xóa khi tài xế yêu cầu."><defs><marker id="py53-0-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><rect x="20" y="35" width="170" height="80" rx="10" class="f-warn-soft s-warn"/><text x="36" y="62" class="t-b">Ảnh selfie</text><text x="36" y="86" class="t-small">ngắn hạn, mã hóa</text><rect x="250" y="35" width="170" height="80" rx="10" class="f-phys-soft s-phys"/><text x="266" y="62" class="t-b">Mô hình embedding</text><text x="266" y="86" class="t-small">ra 128 số</text><rect x="490" y="35" width="210" height="80" rx="10" class="f-pk-soft s-pk"/><text x="506" y="58" class="t-b">face_template</text><text x="506" y="80" class="t-small">embedding, model_version</text><text x="506" y="100" class="t-small">hạn lưu</text><rect x="730" y="35" width="130" height="80" rx="10" class="f-warn-soft s-warn"/><text x="746" y="62" class="t-b">Đồng ý</text><text x="746" y="86" class="t-small">còn hiệu lực</text><path d="M190 75 L250 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-0-arrow)"/><path d="M420 75 L490 75" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-0-arrow)"/><path d="M730 75 L700 75" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py53-0-arrow)"/><rect x="250" y="190" width="170" height="70" rx="10" class="f-none s-warn dash"/><text x="266" y="218" class="t-b">Chỉ mục tìm kiếm</text><text x="266" y="240" class="t-small">bản sao dẫn xuất</text><rect x="455" y="190" width="170" height="70" rx="10" class="f-none s-warn dash"/><text x="471" y="218" class="t-b">Bản sao lưu</text><text x="471" y="240" class="t-small">chu kỳ riêng</text><rect x="660" y="190" width="200" height="70" rx="10" class="f-none s-warn dash"/><text x="676" y="218" class="t-b">Nhật ký, cache</text><text x="676" y="240" class="t-small">đừng ghi vector vào đây</text><path d="M560 115 L335 190" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py53-0-arrow)"/><path d="M595 115 L540 190" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py53-0-arrow)"/><path d="M640 115 L760 190" class="f-none s-warn dash" stroke-width="2" marker-end="url(#py53-0-arrow)"/><text x="20" y="288" class="t-small">Xóa theo yêu cầu phải tới cả ba bản sao dẫn xuất, không chỉ bảng gốc.</text></svg>
</div>
<figcaption>Hình 1. Ảnh chỉ được giữ ngắn hạn, thứ giữ lâu là vector kèm điều kiện (đồng ý, hạn lưu). Mọi bản sao dẫn xuất của vector là chỗ dữ liệu có thể còn sót.</figcaption>
</figure>

## Bao nhiêu phép thử thì đủ

Muốn nói «FAR dưới 0,01%» bạn cần số lần thử đủ lớn để nếu không thấy nhận nhầm nào, cận trên mới đủ thấp. Quy tắc ba cho cận trên một phía 95% xấp xỉ 3/n khi quan sát 0 lỗi trong n lần thử độc lập (Hanley và Lippman-Hand, JAMA 1983). Lab in con số cho hai nhóm dữ liệu của ta:

```text
nhóm 20 người: 19000 cặp khác người; không thấy nhận nhầm nào thì cận trên một phía 95% của FAR cũng chỉ xuống được khoảng 0.0158%, nếu các cặp độc lập
cả 40 người: 78000 cặp khác người; không thấy nhận nhầm nào thì cận trên một phía 95% của FAR cũng chỉ xuống được khoảng 0.0038%, nếu các cặp độc lập
nhưng các cặp dùng chung ảnh và chung người: đơn vị độc lập thực sự là NGƯỜI, ở đây 20 hoặc 40, nên con số trên là cận lạc quan
```

Dòng cuối là điểm chính: 78.000 cặp nghe nhiều nhưng chỉ có 40 người, và các cặp dùng chung ảnh nên không phải 78.000 quan sát độc lập. Một nhóm nhỏ ở dữ liệu thật (một thành phố, một độ tuổi) có thể chỉ vài chục người. Báo cáo NISTIR 8280 (2019) của NIST ghi rằng chênh lệch ở tỉ lệ nhận nhầm «exist broadly, across many, but not all, algorithms tested», thường khác nhau theo hệ số từ 10 tới hơn 100 giữa các nhóm: kết quả phụ thuộc thuật toán nên phải đo với đúng mô hình mình dùng, ở quy mô mà bộ 40 người không thay thế được.

## Lưu embedding thay ảnh, đồng ý, hết hạn và quyền xóa

Nguyên tắc tối thiểu hóa: không lưu thứ mình không cần. Để so khớp sau này ta cần vector, không cần ảnh. Ảnh selfie không nằm lại sau khi đã lấy vector, ngoại trừ phần người duyệt cần để xem hồ sơ `needs_review`: ảnh đó nên được giữ ngắn hạn, mã hóa, tới khi có quyết định thì xóa (lab không dựng kho ảnh này; nơi lưu kết quả OCR có ở [bài 47](/articles/python-luu-ket-qua-ocr-mongodb-pii-retention)). Thứ lưu dài hạn là vector. Bảng `face_template` không có cột ảnh, và `--verify` kiểm đúng danh sách cột. Nhưng vector **vẫn là dữ liệu sinh trắc học**: chính vector đó nhận ra một người từ ảnh khác. Output của lab:

```text
bảng face_template: ['driver_id', 'embedding', 'model_version', 'created_at', 'expires_at']
một mẫu: 512 byte (128 số float32); crop mặt 112x112x3 chưa nén: 37632 byte
lưu mẫu khi chưa có đồng ý: DR-001: chưa có đồng ý cho mục đích kyc_face_match
đưa một ảnh khác của DR-002 vào chỉ mục 4 mẫu, chỉ có embedding: top-1 DR-002 (0.883), top-2 DR-004 (0.402)
```

Kích thước không phải điểm chính; điểm chính là vector không đọc được bằng mắt nhưng vẫn nhận ra người, nên nó cần cùng mức bảo vệ với ảnh: kiểm soát truy cập, mã hóa lúc lưu, không ghi vào log. Đã có nghiên cứu dựng lại ảnh mặt gần giống từ vector trong một số điều kiện; lab không đo điều này nên bài không khẳng định mức rủi ro.

**Đồng ý** được mô hình hóa theo mục đích (`kyc_face_match`), có phiên bản nội dung đồng ý và thời điểm cấp, rút. Mỗi lần cấp và rút là một dòng lịch sử, không ghi đè. Hàm `enroll` ném `ConsentRequired` khi không có đồng ý còn hiệu lực. **Rút đồng ý phải xóa mẫu**: hết căn cứ xử lý thì không giữ vector tới hạn lưu. Một ràng buộc khác: đồng ý ở đây là cho so khớp 1:1 selfie với ảnh giấy tờ của chính tài xế. `FaceIndex` đóng vai chỉ mục tìm kiếm của [bài 51](/articles/python-vector-search-pgvector-faiss-trung-tai-xe), một nhiều (phát hiện một người nhiều tài khoản), là mục đích khác: cần căn cứ riêng, thời hạn riêng và do pháp chế của Tới Nơi quyết định; lab không coi đồng ý 1:1 là đủ cho 1:N.

**Hết hạn (retention)** là một hàm định kỳ xóa mẫu quá `expires_at` (cùng kiểu việc nền ở [bài 40](/articles/python-worker-arq-scheduling-retry-dead-letter); thời gian lấy từ Clock như [bài 14](/articles/python-abc-protocol-composition-dependency-injection), lab cố định giờ để output lặp lại). **Quyền xóa** là `erase`: rút đồng ý (kéo theo xóa mẫu) và ghi một dòng kiểm toán chỉ có biệt danh. Cái khó nằm ở bản sao dẫn xuất, như Hình 1:

```text
DR-004 rút đồng ý: xóa 1 mẫu; get -> None; lịch sử đồng ý 1 dòng, đã rút: True; cấp lại thì lịch sử có 2 dòng
retention 90 ngày: tại ngày 89 xóa 0 mẫu; tại ngày 90 xóa 3 mẫu
xóa DR-002 theo yêu cầu: bảng gốc xóa 1 mẫu; chỉ mục CHƯA dựng lại vẫn trả DR-002; sau khi dựng lại trả DR-003
nhật ký kiểm toán có 19 dòng, ví dụ ('template_erased', '39dd60707afa'); biệt danh của DR-002 là 39dd60707afa, mã gốc không xuất hiện
đổi phiên bản mô hình: mẫu lưu bằng sface_2021dec@opencv-zoo/rgb112, mô hình hiện tại sface_2021dec@opencv-zoo/rgb112-v2
```

Dòng thứ ba là lỗi ta cố tình dựng: bảng gốc đã xóa DR-002 nhưng chỉ mục vẫn trả về DR-002 cho tới khi dựng lại. Trong hệ thống thật, «xóa» phải đi qua mọi nơi giữ vector: chỉ mục, bản sao lưu (xóa theo chu kỳ của chúng, cần nói rõ với người dùng), log, cache, bản export. Nhật ký kiểm toán dùng biệt danh (băm có muối của mã tài xế), không phải ẩn danh: mã tài xế ít khả năng nên ai biết muối vẫn dò ngược được. Dòng cuối là một đánh đổi của «chỉ lưu vector»: khi nâng cấp mô hình, vector cũ không so được với vector mới (`ModelVersionMismatch`), và vì không giữ ảnh nên cách duy nhất là xin tài xế chụp lại. Đó là cái giá của tối thiểu hóa, và nên được chấp nhận có chủ ý.

## Hệ thống gợi ý, người quyết định

Điểm cosine là đầu vào cho người duyệt, không phải quyết định xác minh. Lab thể hiện điều này bằng kiểu dữ liệu, theo máy trạng thái của [bài 41](/articles/python-kyc-tai-xe-bai-toan-du-lieu-metric). `suggest` chia điểm thành ba dải bằng hai ngưỡng (review ở FAR 5%, strong ở FAR 0,1%), kèm phiên bản mô hình và phiên bản ngưỡng; ảnh không lấy được mặt là dải riêng `no_face`, không có điểm. Hệ thống chỉ được đẩy hồ sơ tới `face_checked` (rồi hồ sơ đi tiếp tới `risk_scored`) hoặc `needs_review`. `approved` và `rejected` chỉ đến từ `apply_decision`, hàm đòi hồ sơ đang ở `needs_review`, mã người duyệt (`RV-001`, quy ước của lab) và một lý do không rỗng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 260" role="img" aria-label="Điểm cosine từ Compare được chia thành ba dải strong, review, weak, và ảnh không lấy được mặt là dải riêng. Hệ thống đẩy hồ sơ tới face_checked hoặc needs_review rồi dừng tại một ranh giới. Chỉ người duyệt có mã và lý do mới đưa hồ sơ từ needs_review tới approved hoặc rejected."><defs><marker id="py53-1-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker></defs><text x="20" y="30" class="t-b">Hệ thống</text><text x="660" y="30" class="t-b">Người</text><rect x="20" y="50" width="150" height="80" rx="10" class="f-pk-soft s-pk"/><text x="36" y="80" class="t-b">Điểm cosine</text><text x="36" y="104" class="t-small">từ Compare (bài 52)</text><rect x="210" y="50" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="226" y="80" class="t-b">Dải điểm</text><text x="226" y="104" class="t-small">strong, review, weak</text><rect x="420" y="50" width="170" height="80" rx="10" class="f-pk-soft s-pk"/><text x="436" y="76" class="t-b">Hệ thống đẩy tới</text><text x="436" y="98" class="t-small">face_checked hoặc</text><text x="436" y="116" class="t-small">needs_review</text><path d="M625 40 L625 205" class="f-none s-warn dash" stroke-width="2"/><rect x="660" y="50" width="200" height="80" rx="10" class="f-phys-soft s-phys"/><text x="676" y="76" class="t-b">Người duyệt RV-001</text><text x="676" y="98" class="t-small">hồ sơ ở needs_review</text><text x="676" y="116" class="t-small">tên + lý do bắt buộc</text><rect x="660" y="165" width="200" height="60" rx="10" class="f-phys-soft s-phys"/><text x="676" y="200" class="t-b">approved / rejected</text><path d="M170 90 L210 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-1-arrow)"/><path d="M380 90 L420 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-1-arrow)"/><path d="M590 90 L660 90" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-1-arrow)"/><path d="M760 130 L760 165" class="f-none s-phys" stroke-width="2" marker-end="url(#py53-1-arrow)"/><text x="20" y="190" class="t-small">Dải chỉ quyết định hồ sơ nào lên đầu hàng và hiện bằng chứng gì.</text><text x="20" y="212" class="t-small">Không có đường tự động qua ranh giới dọc.</text></svg>
</div>
<figcaption>Hình 2. Hệ thống dừng ở ranh giới dọc. Hai trạng thái quyết định chỉ đạt được qua một hành động có người ký tên, từ hồ sơ đang chờ duyệt.</figcaption>
</figure>

Dải điểm cho biết khối lượng việc của người duyệt, và cả hai ngưỡng ở đây chọn trên chính dữ liệu đo nên lạc quan (xem phân bố ở mục trên). Trên 79.800 cặp ảnh:

```text
ngưỡng review 0.407 (FAR 5%), ngưỡng strong 0.567 (FAR 0,1%); cả hai chọn trên chính dữ liệu đo nên lạc quan, xem mục 3
cùng người  strong 1747 (97.1%), review 53 (2.9%), weak 0 (0.0%)
khác người  strong 78 (0.1%), review 3822 (4.9%), weak 74100 (95.0%)
```

Cặp khác người rơi vào `strong` chiếm 0,1% (78 cặp): ngay ở dải cao nhất vẫn có người lạ, nên dải `strong` cũng chỉ đẩy hồ sơ tới `face_checked` chứ không tự duyệt. Cặp cùng người rơi vào `weak` là 0% trên bộ sạch này, nhưng bài 49 cho thấy ảnh xuống cấp làm FRR tăng vọt; để dải `weak` tự động từ chối là từ chối người thật vì lỗi của mô hình, nên dải đó cũng đi tới người. Các ca kiểm còn lại: tác nhân `system:face-match` và thiếu lý do bị từ chối, hồ sơ ở `face_checked` hay `submitted` không nhận quyết định của người, ảnh không lấy được mặt đi tới `needs_review`.

## Danh sách kiểm trước khi bật

| Mục | Câu hỏi | Lab thể hiện ở |
| --- | --- | --- |
| Đo | FAR/FRR báo kèm khoảng tin cậy và số người, không chỉ số cặp; ngưỡng đo trên người chưa thấy | `bootstrap_ci`, `threshold_transfer`, mục 3 |
| Nhóm | Có nhãn nhóm thật, thu có đồng ý, đủ người mỗi nhóm; nếu không, nói rõ là chưa đo | mục 3, 4, 5 |
| Lưu | Chỉ lưu thứ cần; vector được bảo vệ như dữ liệu sinh trắc học | `face_template` không cột ảnh |
| Đồng ý | Theo mục đích (1:1 khác 1:N), có phiên bản, rút được, rút thì xóa mẫu | `ConsentRequired`, `withdraw_consent` |
| Hết hạn, xóa | Có hạn lưu; xóa tới cả bản sao dẫn xuất; có dấu vết kiểm toán | `purge_expired`, `erase`, `FaceIndex` |
| Phiên bản | Vector gắn phiên bản mô hình; có đường chụp lại khi nâng cấp | `ModelVersionMismatch` |
| Quyết định | Hồ sơ chỉ được duyệt hoặc từ chối bởi người có tên và lý do, từ `needs_review` | `apply_decision` |
| Theo dõi | Có lát cắt theo dõi sau khi bật; có kênh khiếu nại | ngoài lab |

## Chạy và kiểm chứng

Cần uv và Python 3.14; cần mạng lần chạy đầu để tải SFace (38,7 MB), YuNet (0,2 MB) và bộ Olivetti (tải 4,26 MB, bản cache 1,4 MB), đều vào thư mục cache, không commit. Từ thư mục gốc repository:

```bash
uv run --project examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy examples/toi-noi-python/53-face-matching-danh-gia-bias-privacy/main.py --verify
```

```text
OK 54/54
```

Lần chạy đầu tính embedding cho 400 ảnh (trích output của chế độ thường; các lần sau đọc từ `out/embeddings.npz`). Máy dùng chung đang rất bận nên thời gian chỉ để tham khảo, còn FAR/FRR không phụ thuộc tải máy và lặp lại y hệt giữa các lần chạy:

```text
tải máy lúc chạy (load average 1 phút): 126.8
embedding (400, 128), float32, mô hình sface_2021dec@opencv-zoo/rgb112; tính mới bằng ONNX Runtime, lưu vào out/embeddings.npz: 51.6 giây
```

`--verify` kiểm các bất biến: số cặp theo số ảnh mỗi người, FAR không vượt mục tiêu, tổng cặp trong nhóm cộng cặp chéo nhóm bằng tổng, cùng seed cho cùng khoảng tin cậy, khoảng 95% của hiệu giữa hai nửa ngẫu nhiên bao quanh 0, khoảng 95% của FAR trên người chưa thấy bao quanh mục tiêu 1%, bảng không có cột ảnh, rút đồng ý xóa mẫu và giữ lịch sử, hết hạn, xóa, cổng quyết định. Nó không kiểm một kết luận nào về thiên lệch.

| Hiện tượng | Kiểm tra trước |
| --- | --- |
| FAR theo nhóm lệch nhiều giữa hai nhóm | khoảng tin cậy theo người có chồng nhau không; chia ngẫu nhiên có cho chênh lệch tương đương không; đổi chiều học ngưỡng |
| Khoảng tin cậy rất rộng | số người mỗi nhóm; nhớ đơn vị độc lập là người, không phải cặp |
| Mẫu đã xóa vẫn xuất hiện ở kết quả tìm kiếm | chỉ mục, cache, bản sao lưu có được dựng lại hoặc xóa theo chưa |
| `ModelVersionMismatch` sau khi nâng mô hình | mẫu cũ không dùng được; cần quy trình chụp lại, không trộn phiên bản |
| Hồ sơ ở `approved` mà không có người ký | đường nào đã gọi trạng thái mà không qua `apply_decision` |

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Giới hạn của ví dụ so với production</span>
<p>Olivetti là 40 người chụp trong phòng thí nghiệm, ảnh xám 64×64, không có nhãn nhân khẩu học; các «nhóm» ở trên là giả định hoặc lát cắt ảnh, và mọi tỉ lệ chỉ đúng cho 40 người này, không suy ra cho tài xế của Tới Nơi. Bài không chứng minh hệ thống không thiên lệch. Đánh giá thật cần dữ liệu có nhãn nhóm thu hợp lệ, đủ lớn, ảnh selfie thật qua pipeline đầy đủ, và theo dõi sau khi bật. Kho mẫu ở đây là SQLite trong bộ nhớ; production dùng kho có mã hóa, kiểm soát truy cập, TTL và quy trình xóa tới bản sao lưu. Danh tính người duyệt chỉ kiểm tiền tố <code>RV-</code>; production lấy từ xác thực. Đồng ý, mục đích, số ngày lưu là giá trị minh họa. Bài không phải tư vấn pháp lý.</p></div>

**Bài tập:** Trong mục 3 của `main.py` (hàm `part_assumed_groups`), thêm phép thử cho nhóm giả định thứ ba: người chẵn so với người lẻ, cả hai chiều học ngưỡng. Bạn kỳ vọng hiệu FAR rơi vào đâu so với khoảng của các lần chia ngẫu nhiên, và vì sao? Rồi sửa `erase` để chỉ mục không còn trả DR-002 mà không cần gọi `rebuild` bằng tay.

<details><summary>Xem lời giải</summary><p>Chia chẵn/lẻ cũng chỉ là một cách chia 20/20 theo số thứ tự, không khác một lần chia ngẫu nhiên về bản chất, nên hiệu FAR của nó thường nằm trong khoảng 95% của các lần chia ngẫu nhiên; cái cần so là vị trí của nó trong phân bố, và hai chiều học ngưỡng có thể cho hai số rất khác nhau. Với <code>erase</code>: thêm danh sách <code>self._listeners</code> và <code>subscribe</code>; gọi từng listener với mã tài xế bị xóa trong <code>erase</code>, <code>withdraw_consent</code> và <code>purge_expired</code>; <code>FaceIndex</code> đăng ký <code>self.vectors.pop(driver_id, None)</code>. Lưu ý <code>purge_expired</code> hiện chỉ xóa mẫu, để lại bản ghi đồng ý còn hiệu lực và bản sao trong chỉ mục. Cách này chỉ phủ bản sao trong tiến trình; chỉ mục ở dịch vụ khác cần sự kiện xóa qua hàng đợi và xác nhận lại.</p></details>

Bài trước: [bài 52](/articles/python-face-match-grpc-service-batching-model-loading). Bài tiếp: [bài 54](/articles/python-fraud-toi-noi-bai-toan-nhan-du-lieu-metric).

## Tài liệu gốc

- [NIST: NISTIR 8280, Face Recognition Vendor Test Part 3: Demographic Effects (2019)](https://doi.org/10.6028/NIST.IR.8280).
- [OpenCV Zoo: SFace (face recognition)](https://github.com/opencv/opencv_zoo/tree/main/models/face_recognition_sface).
- [OpenCV Zoo: YuNet (face detection)](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet).
- [scikit-learn: fetch_olivetti_faces](https://scikit-learn.org/stable/modules/generated/sklearn.datasets.fetch_olivetti_faces.html).
- [Python: sqlite3](https://docs.python.org/3.14/library/sqlite3.html).
