---
slug: flutter-ci-github-actions-analyze-test-golden-cache
title: "CI với GitHub Actions: analyze, test, golden và cache pub"
summary: "Bài 33: đưa các lệnh của bài 27 đến 32 vào workflow GitHub Actions cho Tới Nơi: trigger theo path, quyền chỉ đọc, ghim Flutter 3.47.4 và action, cache pub đúng khoá, golden trên macOS, dart analyze --fatal-infos, coverage không ngưỡng; test Dart đọc chính workflow. Workflow chưa chạy trên GitHub."
category: programming
level: intermediate
tags: [flutter, dart, mobile, toi-noi, ci, github-actions, testing]
language: vi
---

Tối thứ sáu, Lan merge một PR sửa thẻ chuyến. Trên máy Lan mọi test xanh. Sáng thứ hai, Minh kéo `main` về thì golden đỏ, còn một lint của plugin mà `flutter analyze` không hề báo. Hai người chạy cùng một bộ lệnh, nhưng trên hai hệ điều hành và có thể hai phiên bản Flutter, nên không ai biết kết quả nào là kết quả thật.

CI cho đúng một câu trả lời: một máy sạch, một phiên bản Flutter ghim cứng, một bộ lệnh viết trong repository. Bài này đưa các lệnh của bài 27 đến 32 vào một workflow GitHub Actions. Đây là bài 33 của lộ trình **Flutter qua Tới Nơi**, bài kết Phần 4. Ta nên đọc [bài 32 về lint và analyzer plugin](/articles/flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code) trước, vì cổng `dart analyze --fatal-infos` đến từ đó, và [bài 29 về golden test](/articles/flutter-golden-test-semantics-test-giao-dien-on-dinh), vì golden quyết định cách chia runner.

Workflow này chạy cả lab 27 đến 33: bước cuối của job `verify` gọi `verify-27-33.mjs`, cùng khuôn với các script verify trước. Lab 29 chạy phần không phải golden ở Ubuntu cùng cả nhóm, còn golden chạy riêng ở macOS.

Nói thẳng từ đầu: nhánh của lab chưa được push, nên workflow này **chưa từng chạy trên GitHub**. Ta kiểm được workflow có những tính chất gì (bằng test Dart) và từng lệnh trong đó xanh trên máy tác giả; cache, golden trên runner macOS và artifact thì chưa kiểm được.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Các lab của loạt bài không commit <code>pubspec.lock</code>. Workflow bật <code>cache: true</code> của <code>subosito/flutter-action</code> với khoá mặc định. Ta thêm một dependency mới vào <code>pubspec.yaml</code> của một lab. Khoá cache pub có đổi không?</p>
<details><summary>Xem đáp án</summary><p>Không. Khoá mặc định của action ghép thêm <code>hashFiles('**/pubspec.lock')</code>, và theo tài liệu GitHub, khi không file nào khớp thì <code>hashFiles</code> trả chuỗi rỗng. Không có lockfile nào được commit nên khoá không bao giờ đổi theo dependency. Cache lại bất biến, nên gói mới không được lưu vào cache cũ và phải tải lại ở mỗi lần chạy. Workflow của bài băm các <code>pubspec.yaml</code> thay vào đó.</p></details>
</div>

## Khi nào workflow chạy, và với quyền gì

Phần đầu file quyết định ba việc: sự kiện nào kích hoạt, token được làm gì, và run cũ có bị huỷ không.

### toi-noi-flutter.yml · .github/workflows/toi-noi-flutter.yml (trích)

```yaml
name: Toi Noi Flutter labs
on:
  workflow_dispatch:
  push:
    # Nhánh có PR đã chạy qua pull_request; push chỉ cần cho main.
    branches: [main]
    paths:
      - 'examples/toi-noi-flutter/**'
      - '.github/workflows/toi-noi-flutter.yml'
  pull_request:
    paths:
      - 'examples/toi-noi-flutter/**'
      - '.github/workflows/toi-noi-flutter.yml'
permissions:
  contents: read
# Nhánh khác: run mới huỷ run cũ cùng ref. Main: mỗi commit một group (theo
# SHA) nên không run nào bị huỷ hay bị thay thế khi đang chờ.
concurrency:
  group: ${{ github.workflow }}-${{ github.ref == 'refs/heads/main' && github.sha || github.ref }}
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```

**Trigger theo path.** Theo tài liệu, workflow chạy khi ít nhất một file đổi khớp một pattern trong `paths`. Trong pattern, `*` khớp mọi ký tự trừ `/`, còn `**` khớp mọi ký tự kể cả `/`. Vì thế `examples/toi-noi-flutter/**` bắt cả `29-golden-semantics/lib/trip_card.dart`, còn `examples/toi-noi-flutter/*` thì không. Sửa một bài viết trong `api/seed/content/` không đốt phút CI; sửa chính workflow thì có, nên đường dẫn của nó nằm trong `paths`.

**Push chỉ cho `main`.** Nếu `push` không lọc nhánh, mỗi lần đẩy commit vào nhánh đã có PR sinh hai run (một `push`, một `pull_request`), và job macOS tốn gấp đôi. Vì vậy `push` có `branches: [main]`; nhánh PR chạy qua `pull_request`.

Cái giá của `paths`: khi bộ lọc bỏ qua workflow, check của nó ở trạng thái "Pending", và PR bắt buộc check đó có thể không merge được.

**Quyền tối thiểu.** `permissions: contents: read` chỉ cho token đọc mã; theo tài liệu, quyền nào không liệt kê là `none`. Workflow chỉ checkout, build và upload artifact, nên không cần ghi gì; tài liệu bảo mật của GitHub khuyên mặc định chỉ đọc rồi nâng riêng từng job.

**Huỷ run cũ.** Theo tài liệu, `concurrency` cho phép tối đa một run trong mỗi `group`: run đang chờ cùng nhóm bị huỷ khi có run mới, còn `cancel-in-progress: true` huỷ cả run đang chạy. Chỉ đặt `cancel-in-progress: false` cho `main` là chưa đủ, run chờ vẫn bị thay. Ta đưa SHA vào tên nhóm của `main`: mỗi commit một nhóm riêng nên không run nào bị huỷ; nhánh khác nhóm theo ref.

## Ghim phiên bản: Flutter, action và Node

CI đổi trạng thái mà không ai đổi code thường do một phiên bản trôi. Có bốn chỗ cần ghim: Flutter, action, Node, và dependency pub. Ba chỗ đầu nằm trong workflow; chỗ cuối ta xem sau bảng.

| Thứ ghim | Trước bài này | Sau |
|---|---|---|
| Flutter | `flutter-version: '3.47.4'`, `channel: stable` | giữ nguyên |
| `subosito/flutter-action` | `@v2` (tag trôi) | `@v2.23.0` |
| `actions/setup-node` | `@v6` | `@v7.1.0` |
| `actions/checkout` | `@v7.0.1` | giữ nguyên |
| `actions/upload-artifact` | chưa dùng | `@v7.0.2` |

Số hiệu lấy từ API release của GitHub ngày 06/11/2026; kiểm lại khi đọc. Flutter 3.47.4 có gói Linux x64 và hai gói macOS (x64, arm64), nên action tìm được SDK trên cả hai runner.

Tag ba số vẫn chưa bất biến: tài liệu bảo mật của GitHub nói tag có thể bị di chuyển nếu kẻ xấu chiếm được repository của action, và khuyên ghim SHA đầy đủ. Lab chọn tag ba số cho dễ đọc, test chấp nhận cả hai dạng; với SHA, lấy từ repository của chính action, không từ fork.

### workflow_audit.dart · 33-ci-github-actions/lib/workflow_audit.dart (trích)

```dart
/// `uses:` có ref cố định: tag đủ ba số (`v7.0.1`) hoặc SHA 40 ký tự hex.
/// Action cục bộ (`./...`) không có ref nên không xét.
final _exactRef = RegExp(r'^(v\d+\.\d+\.\d+|[0-9a-f]{40})$');

  List<Finding> checkActionRefs() {
    final found = <Finding>[];
    for (final (job, step) in steps) {
      final uses = step['uses'] as String?;
      if (uses == null || uses.startsWith('./')) continue;
      final at = uses.lastIndexOf('@');
      final ref = at < 0 ? '' : uses.substring(at + 1);
      if (!_exactRef.hasMatch(ref)) {
        found.add(
          Finding(
            'action-ref',
            'job $job: "$uses" không ghim tag đủ ba số hay SHA 40 ký tự',
          ),
        );
      }
    }
    return found;
  }
```

`v7` hay `main` không khớp nên bị báo; `v7.0.1` và SHA 40 ký tự thì qua. Test "ca sai" thử cả `master`, `latest`, `HEAD`, thiếu `@` và SHA 7 ký tự.

**Dependency pub là chỗ ghim thứ tư, và lab chưa ghim.** Lab không commit `pubspec.lock`, nên CI có thể resolve bản mới hơn máy dev. Điều này đã xảy ra: `flutter_riverpod` 3.5.0 đánh dấu `container.exists` là deprecated và làm `flutter analyze` thoát 1 ở nơi resolve bản mới. Dự án thật commit `pubspec.lock` và chạy `dart pub get --enforce-lockfile`, lệnh hỏng khi lockfile không khớp `pubspec.yaml`. Lab là ví dụ nên ta chấp nhận lệch này và coi CI là nơi báo trôi.

## Cache pub: khoá mặc định không thấy dependency đổi

Lệnh `flutter pub get` tải gói về `~/.pub-cache`; cache thư mục này giúp lần chạy sau khỏi tải lại. Với `cache: true`, `flutter-action` cache cả SDK lẫn pub. Theo `action.yaml` của bản v2.23.0, khoá pub là `flutter-pub-:os:-:channel:-:version:-:arch:-:hash:` nối thêm `-` và `hashFiles('**/pubspec.lock')`.

Chỗ vướng: lab không commit `pubspec.lock` (quy ước của loạt, lockfile chỉ giữ từ `toi-noi-app`), nên phần băm rỗng. Theo tài liệu cache của GitHub, cache không sửa được sau khi tạo, muốn đổi nội dung phải tạo khoá mới. Khoá không đổi thì ta cứ trúng cache cũ, gói thêm sau không vào cache.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 1. Hai cách tạo khoá cache pub. Hàng trên, khoá mặc định: băm của file pubspec.lock cho kết quả rỗng vì không có lockfile nào được commit, nên khoá không đổi khi thêm dependency, nên gói mới tải lại ở mỗi lần chạy. Hàng dưới, khoá của workflow: băm các pubspec.yaml đổi khi dependency đổi, nên khoá mới bị miss, flutter pub get tải gói, và cache mới được lưu cho các lần sau.">
<defs><marker id="fl33-1-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker><marker id="fl33-1-p" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-pk"/></marker></defs>
<text x="20" y="28" class="t-b">Khoá mặc định của action</text>
<rect x="20" y="44" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="34" y="74" class="t-b">Băm lockfile</text><text x="34" y="100" class="t-small">hashFiles của</text><text x="34" y="120" class="t-small">**/pubspec.lock</text>
<rect x="315" y="44" width="250" height="110" rx="10" class="f-surface s-muted"/><text x="329" y="74" class="t-b">Kết quả rỗng</text><text x="329" y="100" class="t-small">không có lockfile nào</text><text x="329" y="120" class="t-small">được commit</text>
<rect x="610" y="44" width="250" height="110" rx="10" class="f-warn-soft s-warn"/><text x="624" y="74" class="t-b">Khoá không đổi</text><text x="624" y="100" class="t-small">luôn trúng cache cũ,</text><text x="624" y="120" class="t-small">gói mới tải lại mãi</text>
<path d="M272 99 L313 99" class="f-none s-warn" stroke-width="2" marker-end="url(#fl33-1-w)"/>
<path d="M567 99 L608 99" class="f-none s-warn" stroke-width="2" marker-end="url(#fl33-1-w)"/>
<text x="20" y="208" class="t-b">Khoá của workflow</text>
<rect x="20" y="224" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="34" y="254" class="t-b">Băm pubspec.yaml</text><text x="34" y="280" class="t-small">mọi pubspec.yaml</text><text x="34" y="300" class="t-small">dưới examples/</text>
<rect x="315" y="224" width="250" height="110" rx="10" class="f-surface s-muted"/><text x="329" y="254" class="t-b">Đổi khi pubspec đổi</text><text x="329" y="280" class="t-small">thêm dependency thì</text><text x="329" y="300" class="t-small">khoá mới, miss lần đầu</text>
<rect x="610" y="224" width="250" height="110" rx="10" class="f-pk-soft s-pk"/><text x="624" y="254" class="t-b">Lưu cache mới</text><text x="624" y="280" class="t-small">pub get tải gói, cache</text><text x="624" y="300" class="t-small">mới phục vụ lần sau</text>
<path d="M272 279 L313 279" class="f-none s-pk" stroke-width="2" marker-end="url(#fl33-1-p)"/>
<path d="M567 279 L608 279" class="f-none s-pk" stroke-width="2" marker-end="url(#fl33-1-p)"/>
<text x="20" y="378" class="t-small">Phần bên phải của hàng dưới chưa được kiểm trên GitHub: nhánh chưa push.</text>
</svg>
</div>
<figcaption>Hình 1. Cùng một cache, hai cách đặt khoá. Chỉ khoá băm pubspec.yaml đổi khi dependency đổi.</figcaption>
</figure>

Tham số `pub-cache-key` cho ta khoá riêng; phần băm lockfile vẫn được nối vào nhưng rỗng nên vô hại. Ta băm mọi `pubspec.yaml` dưới `examples/toi-noi-flutter`:

### toi-noi-flutter.yml · .github/workflows/toi-noi-flutter.yml (trích)

```yaml
      - uses: subosito/flutter-action@v2.23.0
        with:
          flutter-version: '3.47.4'
          channel: stable
          cache: true
          # Lab không commit pubspec.lock nên khoá mặc định của action không
          # đổi khi dependency đổi; băm các pubspec.yaml thay vào đó.
          pub-cache-key: flutter-pub-:os:-:channel:-:version:-:arch:-${{ hashFiles('examples/toi-noi-flutter/**/pubspec.yaml') }}
```

Theo tài liệu GitHub (kiểm tại 11/2026), một run chỉ khôi phục cache của nhánh hiện tại, nhánh mặc định và với PR là nhánh gốc của PR, không phải nhánh anh em; cache không truy cập quá 7 ngày bị xoá, mặc định mỗi repository có 10 GB. Test của lab kiểm khoá có băm `pubspec.yaml` và `.gitignore` của lab 28, 29 vẫn liệt kê `pubspec.lock`. Cache pub cũng không phải mọi thứ: lab 22 phụ thuộc `sqlite3`, có build hook tải binary SQLite khi build, mà cache chỉ giữ `~/.pub-cache`. Lab này chưa chạy trên Linux nên chưa biết mỗi job có tải lại không.

## Hai runner: golden trên macOS, phần còn lại trên Linux

Ảnh chuẩn của lab 29 sinh trên macOS; trên hệ điều hành khác, các test chạm vào ảnh chuẩn tự `skip` kèm lý do. Chỉ có runner Linux thì golden luôn "xanh" theo kiểu bị bỏ qua. Vì vậy workflow có một job riêng cho golden, chạy trên macOS, nơi ảnh được sinh.

### toi-noi-flutter.yml · .github/workflows/toi-noi-flutter.yml (trích)

```yaml
  golden-macos:
    name: Golden lab 29 (macOS)
    runs-on: macos-latest
    timeout-minutes: 20
    steps:
      - uses: actions/checkout@v7.0.1
      # ... flutter-action với pub-cache-key như trên ...
      - name: Golden tests lab 29
        working-directory: examples/toi-noi-flutter/29-golden-semantics
        run: |
          flutter pub get
          flutter test --tags golden
      - name: Upload golden failures
        if: failure()
        uses: actions/upload-artifact@v7.0.2
        with:
          name: golden-failures-lab29
          path: examples/toi-noi-flutter/29-golden-semantics/test/failures/
          if-no-files-found: ignore
          retention-days: 7
```

Khi golden trượt, `flutter_test` ghi ảnh diff vào thư mục `failures` cạnh file test (đọc trong `_goldens_io.dart`); lab 29 ghi bốn ảnh mỗi lần trượt. Bước upload cần `if: failure()` vì mặc định một step chỉ chạy khi các step trước thành công. `if-no-files-found: ignore` giữ nó yên lặng khi lỗi không phải golden; `retention-days: 7` là lựa chọn của ta.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 2. Hai job chạy song song. Sự kiện push hoặc pull_request có đổi file khớp paths, hoặc chạy tay bằng workflow_dispatch, kích hoạt cả hai job. Job golden-macos chạy trên macOS với lệnh flutter test --tags golden và, khi đỏ, upload thư mục test/failures. Job verify chạy trên ubuntu với dart analyze --fatal-infos cho lab 32, flutter test --coverage cho lab 28 rồi các script verify; nó upload lcov.info.">
<defs><marker id="fl33-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="115" width="200" height="130" rx="10" class="f-surface s-muted"/><text x="34" y="145" class="t-b">Sự kiện</text><text x="34" y="171" class="t-small">push, pull_request</text><text x="34" y="191" class="t-small">có file khớp paths</text><text x="34" y="211" class="t-small">hoặc chạy tay</text>
<rect x="290" y="20" width="330" height="140" rx="10" class="f-pk-soft s-pk"/><text x="306" y="50" class="t-b">golden-macos</text><text x="306" y="76" class="t-small">macos-latest, timeout 20</text><text x="306" y="100" class="t-mono">flutter test --tags golden</text><text x="306" y="124" class="t-small">lab 29, nơi ảnh chuẩn sinh ra</text>
<rect x="290" y="190" width="330" height="170" rx="10" class="f-phys-soft s-phys"/><text x="306" y="220" class="t-b">verify</text><text x="306" y="246" class="t-small">ubuntu-latest, timeout 120</text><text x="306" y="270" class="t-mono">dart analyze --fatal-infos</text><text x="306" y="294" class="t-mono">flutter test --coverage</text><text x="306" y="318" class="t-small">rồi script verify-*.mjs</text><text x="306" y="342" class="t-small">analyze + test từng nhóm lab</text>
<rect x="690" y="20" width="170" height="140" rx="10" class="f-surface s-line"/><text x="704" y="50" class="t-b">Artifact (đỏ)</text><text x="704" y="76" class="t-small">test/failures</text><text x="704" y="96" class="t-small">giữ 7 ngày</text>
<rect x="690" y="190" width="170" height="170" rx="10" class="f-surface s-line"/><text x="704" y="220" class="t-b">Artifact</text><text x="704" y="246" class="t-small">coverage/lcov.info</text><text x="704" y="266" class="t-small">giữ 14 ngày</text>
<path d="M222 150 C256 150 256 90 288 90" class="f-none s-muted" stroke-width="2" marker-end="url(#fl33-2-a)"/>
<path d="M222 210 C256 210 256 275 288 275" class="f-none s-muted" stroke-width="2" marker-end="url(#fl33-2-a)"/>
<path d="M622 90 L688 90" class="f-none s-muted" stroke-width="2" marker-end="url(#fl33-2-a)"/>
<path d="M622 275 L688 275" class="f-none s-muted" stroke-width="2" marker-end="url(#fl33-2-a)"/>
</svg>
</div>
<figcaption>Hình 2. Hai job độc lập, không có needs. Golden ở macOS vì ảnh chuẩn sinh ở đó; mọi thứ còn lại ở Linux vì rẻ hơn.</figcaption>
</figure>

**Chi phí phút.** Giá theo phút của runner chuẩn trên trang billing của GitHub, **kiểm tại 11/2026**. Repository công khai chạy runner chuẩn miễn phí, nên so sánh này dành cho repository riêng.

| Runner | USD mỗi phút | So với Linux |
|---|---|---|
| Linux 2 nhân (x64) | 0,006 | 1 |
| Windows 2 nhân (x64) | 0,010 | khoảng 1,7 |
| macOS 3 hoặc 4 nhân | 0,062 | khoảng 10,3 |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 250" role="img" aria-label="Hình 3. Giá theo phút của runner chuẩn, kiểm tại 11/2026. Cột Linux 0,006 USD ngắn nhất, Windows 0,010 USD dài hơn một chút, macOS 0,062 USD dài gấp khoảng mười lần Linux.">
<rect x="20" y="30" width="150" height="44" rx="8" class="f-surface s-muted"/><text x="34" y="58" class="t-b">Linux 2 nhân</text>
<rect x="190" y="30" width="50" height="44" rx="6" class="f-phys-soft s-phys"/><text x="252" y="58" class="t-small">0,006 USD/phút</text>
<rect x="20" y="95" width="150" height="44" rx="8" class="f-surface s-muted"/><text x="34" y="123" class="t-b">Windows 2 nhân</text>
<rect x="190" y="95" width="84" height="44" rx="6" class="f-surface s-line"/><text x="286" y="123" class="t-small">0,010 USD/phút</text>
<rect x="20" y="160" width="150" height="44" rx="8" class="f-surface s-muted"/><text x="34" y="188" class="t-b">macOS</text>
<rect x="190" y="160" width="520" height="44" rx="6" class="f-warn-soft s-warn"/><text x="722" y="188" class="t-small">0,062 USD/phút</text>
<text x="20" y="236" class="t-small">Độ dài cột tỉ lệ với giá. Nguồn: docs.github.com, kiểm tại 11/2026.</text>
</svg>
</div>
<figcaption>Hình 3. Mỗi phút macOS bằng khoảng mười phút Linux, nên chỉ việc cần macOS mới nên chạy ở đó.</figcaption>
</figure>

Minh hoạ: job golden 6 phút, 100 lần chạy mỗi tháng (số do ta đặt, chưa đo) là 600 phút, khoảng 37,2 USD ở macOS và 3,6 USD ở Linux. Gói Free có 12 USD mỗi tháng (kiểm tại 11/2026). Vì vậy chỉ lệnh golden lên macOS, và `paths` cắt những lần chạy không cần.

Ta chưa biết ảnh sinh trên macOS của máy tác giả có khớp runner `macos-latest` không. Nếu lần đầu golden đỏ, hãy xem `test/failures` và có thể phải sinh lại ảnh chuẩn trên đúng loại máy đó.

## Analyze, coverage và artifact trong job verify

Job `verify` chạy trên Ubuntu. Hai bước mới nằm trước các bước `Analyze and test labs`:

### toi-noi-flutter.yml · .github/workflows/toi-noi-flutter.yml (trích)

```yaml
      # flutter analyze không in chẩn đoán của analyzer plugin; dart analyze có.
      - name: Analyzer plugins gate lab 32
        working-directory: examples/toi-noi-flutter/32-lint-analyzer
        run: |
          flutter pub get
          dart analyze --fatal-infos
      # Báo cáo, không phải cổng: không có ngưỡng coverage tối thiểu.
      - name: Coverage lab 28
        working-directory: examples/toi-noi-flutter/28-widget-test
        run: |
          flutter pub get
          flutter test --coverage
          awk -F: '/^LF:/{f+=$2} /^LH:/{h+=$2} END{printf "Lab 28 line coverage: %d/%d (%.1f%%)\n", h, f, 100*h/f}' coverage/lcov.info | tee -a "$GITHUB_STEP_SUMMARY"
      - name: Upload coverage lab 28
        uses: actions/upload-artifact@v7.0.2
        with:
          name: lcov-lab28
          path: examples/toi-noi-flutter/28-widget-test/coverage/lcov.info
          retention-days: 14
```

**Timeout 120 phút.** Ở máy tác giả (Mac 10 nhân, có cache), lab 01 đến 26 chạy khoảng 20 phút. Runner chậm hơn 2 đến 3 lần là 40 đến 60 phút, và nhóm 27 đến 33 cùng các lab sau còn cộng thêm. Mặc định của GitHub là 360 phút, quá rộng cho một job treo; 45 thì sát. Ta đặt 120, số minh hoạ chưa đo trên runner. Khi vượt, hãy chia `verify` thành vài job theo nhóm bài hoặc dùng `matrix`, đừng nâng mãi timeout.

**Vì sao `dart analyze`.** Bài 32 đã thử: với Flutter 3.47.4, `flutter analyze` không in chẩn đoán của analyzer plugin, còn `dart analyze` có. Các script `verify-*.mjs` hiện có chạy `flutter analyze`, nên lint tự viết của lab 32 sẽ lọt qua nếu chỉ dựa vào chúng. Cờ `--fatal-infos` đổi mã thoát thành 1 khi chỉ còn info: ngân sách cảnh báo bằng 0. Lần đầu cần mạng để tải `riverpod_lint` và biên dịch plugin; ở máy tác giả lệnh mất từ khoảng 30 giây (máy rảnh) tới 72 giây (máy tải nặng, build sạch).

**Coverage là báo cáo, không phải cổng.** `flutter test --coverage` ghi `coverage/lcov.info`, trong đó `LF` là số dòng có thể chạy và `LH` là số dòng đã chạy. Ở lab 28, ta đo được 137 trên 158 dòng, khoảng 86,7% (số của máy tác giả). Workflow in con số vào trang tóm tắt của run và upload `lcov.info`, nhưng **không đặt ngưỡng cứng**. Ngưỡng dễ khiến người ta viết test chỉ để chạm dòng, mà coverage chỉ đếm dòng được chạy, không đếm test có khẳng định đúng hành vi không. Nếu muốn ngưỡng sau này, nên áp cho code mới trong diff. Bước này chạy lại test lab 28 mà script verify cũng chạy, một chi phí nhỏ đổi lấy `lcov.info`.

## Kiểm chính workflow bằng test

Lab 33 đọc workflow thật bằng package `yaml` và khẳng định các tính chất ở trên. Package này đọc khoá `on` là chuỗi `on`, không thành `true` như YAML 1.1, và một test khẳng định điều đó. Để mỗi kiểm tra thật sự đỏ khi cần, các "ca sai" sửa workflow thật đúng một chỗ:

### support.dart · 33-ci-github-actions/test/support.dart (trích)

```dart
WorkflowAudit mutated(String from, String to, {bool all = false}) {
  final original = loadRealWorkflow().text;
  if (!original.contains(from)) {
    throw StateError('Workflow thật không chứa "$from"; sửa test.');
  }
  final text = all
      ? original.replaceAll(from, to)
      : original.replaceFirst(from, to);
  return WorkflowAudit(text, repoRoot: repoRoot);
}
```

Nếu đoạn `from` biến mất, test ném lỗi thay vì lặng lẽ thành no-op. Ví dụ: đổi `**` thành `*` trong `paths` phải làm kiểm tra đỏ.

### audit_rules_test.dart · 33-ci-github-actions/test/audit_rules_test.dart (trích)

```dart
    test('đổi ** thành * làm mất lab nằm sâu một cấp', () {
      final audit = mutated(
        "'examples/toi-noi-flutter/**'",
        "'examples/toi-noi-flutter/*'",
        all: true,
      );
      expect(audit.checkPathsTrigger(projectDirs()), isNotEmpty);
    });
```

Các nhóm test còn lại: Flutter ghim đúng 3.47.4; có `permissions` chỉ đọc; mọi `working-directory` và script `node` tồn tại; không có chuỗi giống bí mật (giá trị giả trong test ghép từ mảnh); và các job đúng như bài mô tả. Test không chứng minh workflow đúng schema của GitHub; việc đó thuộc về `actionlint`, mà máy kiểm không có.

## Khép Phần 4: mục tiêu và đối chiếu

Mục tiêu của Phần 4 là: mỗi lớp của app có loại test phù hợp, chạy xanh trên CI, và quy tắc lint giữ code đồng nhất. Đối chiếu với những gì ta có:

| Mục tiêu | Đã có | Còn thiếu |
|---|---|---|
| Mỗi lớp một loại test | Bài 27 đến 31: domain, widget, golden, Riverpod, integration | Bài 31 cần thiết bị, chưa chạy ở máy kiểm |
| Xanh trên CI | Workflow analyze, test, golden riêng trên macOS | Chưa chạy trên GitHub; integration test chưa nằm trong CI |
| Lint đồng nhất | Cổng `dart analyze --fatal-infos` ở CI, pre-commit ở bài 32 | Pre-commit bỏ qua được; CI là cổng cuối |

Phần 4 đã dựng đủ các mảnh và test từng mảnh trên máy, nhưng "xanh trên CI" mới là khẳng định về file, chưa phải về một lần chạy thật. Lần push đầu tiên là phép thử.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi dựng CI cho Flutter</span>
<ul>
<li><strong>CI Linux xanh nhưng không ai kiểm golden.</strong> Dấu hiệu: giao diện đổi mà CI vẫn xanh. Nguyên nhân: trên Linux, golden tự bỏ qua. Cách sửa: chạy golden ở job macOS với <code>--tags golden</code>.</li>
<li><strong>PR kẹt "Pending".</strong> Dấu hiệu: PR không merge được dù không có lỗi. Nguyên nhân: <code>paths</code> bỏ qua workflow mà nhánh bảo vệ bắt buộc check của nó. Cách sửa: đừng bắt buộc check của workflow có <code>paths</code>.</li>
<li><strong>Cache pub không bao giờ làm mới.</strong> Dấu hiệu: thêm dependency mà <code>pub get</code> vẫn tải lại mỗi lần. Nguyên nhân: khoá chỉ băm <code>pubspec.lock</code> mà lab không commit lockfile. Cách sửa: <code>pub-cache-key</code> băm <code>pubspec.yaml</code>.</li>
<li><strong>Golden đỏ mà không có ảnh để xem.</strong> Dấu hiệu: log chỉ có "Pixel test failed". Nguyên nhân: bước upload thiếu <code>if: failure()</code> hoặc <code>path</code> sai. Cách sửa: thêm <code>if: failure()</code> và trỏ <code>test/failures/</code>.</li>
<li><strong>Không ghim Flutter.</strong> Dấu hiệu: CI đổi trạng thái mà không ai đổi code. Nguyên nhân: phiên bản do action chọn lúc chạy. Cách sửa: <code>flutter-version: '3.47.4'</code> cùng <code>channel: stable</code>; test của lab bắt thiếu ghim.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/33-ci-github-actions
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3 trên macOS Apple silicon: `dart analyze --fatal-infos` báo `No issues found!`, `dart test` chạy 55 test xanh (24 trên workflow thật, 31 ca sai). Mức: **Kiểm chứng tự động**, không cần thiết bị. Test không dùng mạng, cần repository đầy đủ vì đọc workflow và vài file lab khác. Ta cũng chạy tay từng lệnh của workflow trong lab tương ứng, và cả ba đều xanh: `flutter test --tags golden` (lab 29), `dart analyze --fatal-infos` (lab 32), `flutter test --coverage` (lab 28).

**Chưa làm**:

- Workflow chưa chạy trên GitHub vì nhánh chưa push: chưa kiểm cache có trúng, golden có xanh trên runner macOS, artifact có tải được, thời gian chạy thực.
- Chưa chạy `actionlint`: máy không có, ta không cài mới. Nếu có, chạy `actionlint .github/workflows/toi-noi-flutter.yml`.
- Chưa ghim dependency pub bằng lockfile, chưa chia `verify` thành nhiều job, chưa ghim action bằng SHA, chưa đặt `persist-credentials: false` cho `actions/checkout`, chưa có ngưỡng coverage.
- Số lần chạy và thời gian trong bài là minh hoạ.

## Bài tập

1. Một PR chỉ sửa `docs/flutter-master-curriculum.md`, một PR khác sửa `examples/toi-noi-flutter/packages/toi_noi_core/lib/src/money.dart`. Workflow chạy cho PR nào?
2. PR đổi `29-golden-semantics/lib/trip_card.dart`: job `verify` xanh, `golden-macos` đỏ. Tải artifact nào, trong đó có gì, và hai hướng xử lý là gì?
3. Ta thêm package `uuid` vào `pubspec.yaml` của lab 22. Với khoá cache của workflow, lần chạy đầu sau đó diễn ra thế nào? Còn với khoá mặc định của action?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Chỉ PR thứ hai. <code>docs/</code> không khớp pattern nào trong <code>paths</code>, còn <code>examples/toi-noi-flutter/**</code> khớp mọi file ở độ sâu bất kỳ vì <code>**</code> vượt qua dấu <code>/</code>.</p>
<p><strong>2.</strong> Artifact <code>golden-failures-lab29</code>, chứa <code>test/failures/</code> với bốn ảnh cho mỗi golden trượt: <code>masterImage</code>, <code>testImage</code>, <code>isolatedDiff</code>, <code>maskedDiff</code>. Mở <code>isolatedDiff</code> trước. Nếu chủ ý, sinh lại ảnh chuẩn bằng <code>flutter test --update-goldens --tags golden</code> trên macOS và commit cùng PR; nếu không, sửa code.</p>
<p><strong>3.</strong> Với khoá của workflow, băm các <code>pubspec.yaml</code> đổi nên khoá mới trượt cache, <code>pub get</code> tải gói rồi cache mới được lưu cho lần sau. Với khoá mặc định, khoá không đổi (không có lockfile), nên vẫn trúng cache cũ không có gói mới và gói đó phải tải lại ở mọi lần chạy.</p>
</details>

Bài trước: [Lint, analyzer, dart format và analyzer plugin: chất lượng code có kiểm chứng](/articles/flutter-lint-analyzer-format-analyzer-plugin-chat-luong-code). Bài tiếp theo mở đầu Phần 5 với bản đồ flutter_map.

## Tài liệu gốc

- [Workflow syntax for GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax): `on.push.paths`, `concurrency`, `permissions`, `timeout-minutes` (docs.github.com).
- [Dependency caching reference](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching): khoá, phạm vi theo nhánh, giới hạn dung lượng (docs.github.com).
- [Evaluate expressions in workflows and actions](https://docs.github.com/en/actions/reference/workflows-and-actions/expressions): `hashFiles` (docs.github.com).
- [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use): ghim action, quyền tối thiểu cho `GITHUB_TOKEN` (docs.github.com).
- [About billing for GitHub Actions](https://docs.github.com/en/billing/concepts/product-billing/github-actions): giá theo phút của runner (docs.github.com).
- [subosito/flutter-action](https://github.com/subosito/flutter-action) và [actions/upload-artifact](https://github.com/actions/upload-artifact): đầu vào của action.
- [matchesGoldenFile](https://api.flutter.dev/flutter/flutter_test/matchesGoldenFile.html) (api.flutter.dev) và [Continuous delivery with Flutter](https://docs.flutter.dev/deployment/cd) (docs.flutter.dev).
