---
slug: flutter-firebase-app-distribution-phan-phoi-noi-bo-tester
title: "Firebase App Distribution: phân phối nội bộ cho tester iOS và Android của Tới Nơi"
summary: "Bài 55: đưa APK và IPA ad-hoc từ CI tới nhóm tester qa-noi-bo bằng firebase CLI và service account (không dùng token đã deprecated), release notes tiếng Việt sinh từ commit, vòng UDID cho iOS, feedback, crash và so với TestFlight. Test chạy script thật với lệnh giả. Chưa chạy trên Firebase."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, android, release, firebase, app-distribution, ci]
language: vi
---

Chị Mai là QA của Tới Nơi. Sáng nay chị cần bản staging mới của app khách trên một iPhone và một Pixel. Workflow bài 54 đã dựng xong, nhưng artifact nằm trên GitHub: ai tải cũng cần quyền đọc repository, hết hạn sau 14 ngày, và IPA không cài được vì iPhone của chị chưa có trong provisioning profile.

Thứ còn thiếu là một "bưu điện" giữa CI và máy tester. Firebase App Distribution làm việc đó cho cả iOS lẫn Android. Bài này nối nó vào workflow bài 54, sinh release notes từ commit, xử lý vòng đăng ký UDID của iOS và nói rõ chỗ nó thua TestFlight.

Đây là bài 55 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 54 về CI/CD build](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets) trước, vì bài này phân phối đúng các artifact ở đó. IPA ad-hoc ký theo [bài 52](/articles/flutter-ky-ung-dung-ios-certificate-provisioning-profile-capabilities); lane fastlane nối vào [bài 53](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build).

Nói thẳng: repository này không có tài khoản Firebase, nên **chưa có lần phân phối thật nào**. Ta kiểm file cấu hình, kiểm release notes bằng test, và chạy thật từng script `run` bằng bash với `dart`, `firebase` giả.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Ta phân phối IPA ad-hoc build 42 cho nhóm QA. Chị Mai nhận email, bấm cài, nhưng iPhone báo không cài được. Nguyên nhân có thể là gì, và cần làm gì để chị cài được?</p>
<details><summary>Xem đáp án</summary><p>IPA ad-hoc chỉ cài được trên máy có UDID nằm trong provisioning profile đã ký nó, và máy chị Mai chưa có trong profile của build 42. Khi chị chấp nhận lời mời, App Distribution xin phép lấy UDID và báo cho ta. Ta thêm UDID vào Apple Developer và profile ad-hoc, build lại IPA rồi phân phối lại. Gửi lại build cũ vô ích: profile nằm trong IPA.</p></details>
</div>

## Project, app và nhóm tester

Khung chương trình chia flavor theo kênh: `dev` chạy trên máy lập trình viên, `prod` đi qua cửa hàng, chỉ `staging` qua App Distribution. Firebase project có mỗi flavor một app Android và một app iOS (bài 50), nên ở đây chỉ app `vn.toinoi.customer.staging` trên hai nền tảng, cộng cặp của app tài xế, nhận bản build.

Người nhận được gom vào **nhóm** với một alias, lab dùng `qa-noi-bo`. Theo tài liệu (kiểm tại 11/2026), một project có tối đa 500 tester, một nhóm tối đa 200, có thể xin tăng không mất phí; lời mời hết hạn sau 30 ngày.

Workflow và fastlane cùng đọc một file định danh. App ID không phải bí mật, nhưng bản ở đây là giá trị giả, trùng với file `.example` của bài 50:

### app-distribution.example.json · 55-app-distribution/native-config/app-distribution.example.json (trích)

```json
{
  "firebaseProjectId": "toi-noi-example",
  "testerGroups": {
    "internal": { "alias": "qa-noi-bo", "displayName": "QA nội bộ" }
  },
  "apps": {
    "customer": {
      "staging": {
        "android": {
          "appId": "1:000000000000:android:FAKE-STAGING-REPLACE-ME",
          "packageName": "vn.toinoi.customer.staging"
        },
        "ios": {
          "appId": "1:000000000000:ios:FAKE-STAGING-REPLACE-ME",
          "bundleId": "vn.toinoi.customer.staging"
        }
      }
    },
```

`DistributionConfig` đọc file này và từ chối mọi thứ không phải `staging`: không có đường nào để lệnh phân phối trỏ tới `prod`.

### distribution_config.dart · 55-app-distribution/lib/src/distribution_config.dart (trích)

```dart
    if (flavor != Flavor.staging) {
      throw DistributionException(
        'flavor ${flavor.name} không đi qua App Distribution: dev chạy trên '
        'máy lập trình viên, prod đi qua cửa hàng',
      );
    }
```

Hàm đọc còn kiểm dạng App ID và Application ID (qua `applicationId` của lab 53). Test chứng minh ca `dev` và `prod`:

### distribution_config_test.dart · 55-app-distribution/test/distribution_config_test.dart (trích)

```dart
    test('target() từ chối dev và prod, nói rõ vì sao', () {
      for (final flavor in [Flavor.dev, Flavor.prod]) {
        expect(
          () =>
              config.target(AppKind.customer, flavor, DistributionPlatform.ios),
          throwsA(
            isA<DistributionException>().having(
              (e) => e.message,
              'message',
              contains(flavor.name),
            ),
          ),
          reason: flavor.name,
        );
      }
    });
```

Giới hạn khác (kiểm tại 11/2026): file tối đa 2.048 MiB, tối đa 1.000 release mỗi app, mỗi release có mặt 150 ngày.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 410" role="img" aria-label="Hình 1. Đường đi của một bản staging trong cùng một workflow. Các job android và ios của bài 54 cho ra APK và IPA ad-hoc, giữ 14 ngày. Job notes không có secret sinh release notes. Job distribute chạy trên ubuntu, đợi android, ios và notes, tải artifact của chính run, gọi firebase CLI bằng khoá service account rồi xoá khoá, và không chạy mã của repository. Firebase App Distribution nhận bản build cho nhóm qa-noi-bo. Từ đó APK cài thẳng lên máy Android, còn IPA chỉ cài được trên iPhone có UDID nằm trong profile. Chỉ flavor staging đi qua kênh này.">
<defs><marker id="fl55-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="100" width="170" height="130" rx="10" class="f-surface s-muted"/><text x="34" y="130" class="t-b">Job android, ios</text><text x="34" y="156" class="t-small">APK + IPA ad-hoc</text><text x="34" y="176" class="t-small">artifact 14 ngày</text><text x="34" y="196" class="t-small">flavor staging</text>
<rect x="225" y="70" width="225" height="190" rx="10" class="f-pk-soft s-pk"/><text x="239" y="100" class="t-b">Job distribute</text><text x="239" y="126" class="t-small">ubuntu-24.04, secret</text><text x="239" y="146" class="t-small">tải artifact cùng run</text><text x="239" y="166" class="t-small">firebase CLI 15.35.0</text><text x="239" y="186" class="t-small">khoá service account</text><text x="239" y="206" class="t-small">không chạy mã repo</text><text x="239" y="226" class="t-small">xoá khoá: if always()</text>
<rect x="225" y="290" width="225" height="70" rx="10" class="f-surface s-line"/><text x="239" y="318" class="t-b">Job notes</text><text x="239" y="340" class="t-small">không secret, dart run</text>
<rect x="490" y="70" width="190" height="190" rx="10" class="f-phys-soft s-phys"/><text x="504" y="100" class="t-b">App Distribution</text><text x="504" y="126" class="t-small">project Firebase</text><text x="504" y="146" class="t-small">nhóm qa-noi-bo</text><text x="504" y="166" class="t-small">200 tester mỗi nhóm</text><text x="504" y="186" class="t-small">release giữ 150 ngày</text><text x="504" y="206" class="t-small">thư mời, trạng thái</text>
<rect x="715" y="40" width="145" height="100" rx="10" class="f-surface s-line"/><text x="729" y="70" class="t-b">Android</text><text x="729" y="96" class="t-small">APK cài thẳng</text>
<rect x="715" y="180" width="145" height="120" rx="10" class="f-warn-soft s-warn"/><text x="729" y="210" class="t-b">iOS</text><text x="729" y="236" class="t-small">IPA ad-hoc</text><text x="729" y="256" class="t-small">UDID phải có</text><text x="729" y="276" class="t-small">trong profile</text>
<path d="M192 165 L223 165" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-1-a)"/>
<path d="M337 288 L337 262" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-1-a)"/>
<path d="M452 165 L488 165" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-1-a)"/>
<path d="M682 150 C700 150 700 90 713 90" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-1-a)"/>
<path d="M682 190 C700 190 700 240 713 240" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-1-a)"/>
<rect x="20" y="374" width="840" height="30" rx="8" class="f-warn-soft s-warn"/><text x="34" y="394" class="t-small">Chỉ flavor staging đi qua kênh này: dev ở máy lập trình viên, prod đi qua cửa hàng</text>
</svg>
</div>
<figcaption>Hình 1. Từ artifact của bài 54 tới máy tester trong một workflow; riêng iOS có thêm vòng đăng ký thiết bị (Hình 3).</figcaption>
</figure>

## Phân phối từ CI bằng firebase CLI

Android trước. App Distribution nhận cả APK lẫn AAB, nhưng AAB đòi liên kết Firebase với Google Play: app phải có trên Play Console cùng package name và đã xuất bản trên một track (Internal testing, không phải Draft). Play còn ký lại APK bằng một chứng chỉ thử riêng, nên dịch vụ gắn với chữ ký (đăng nhập Google, Maps) phải đăng ký thêm vân tay đó. Với staging ta chọn APK. Bài 54 chỉ dựng AAB cho Play (bài 57), nên ta dán thêm bước dựng APK vào job `android` của nó:

### build-apk-steps.example.yml · 55-app-distribution/native-config/.github/workflows/build-apk-steps.example.yml (trích)

```yaml
- name: Build APK
  run: |
    flutter build apk --release --flavor staging \
      --dart-define-from-file=config/staging.json \
      --build-number="$BUILD_NUMBER" \
      --obfuscate --split-debug-info=build/symbols/android-apk
- name: Upload APK
  uses: actions/upload-artifact@v7.0.2
  with:
    name: toi-noi-customer-staging-apk-${{ needs.prepare.outputs.build_number }}
    path: apps/customer/build/app/outputs/flutter-apk/app-staging-release.apk
    if-no-files-found: error
    retention-days: 14
```

Test đối chiếu từng cờ với bước AAB của bài 54. Đường dẫn APK suy ra từ mã nguồn công cụ Flutter 3.47.4, chưa build thật.

Phần phân phối là hai job dán vào cuối chính workflow của bài 54 (đoạn mẫu nằm trong lab, ta không sửa bài 54), không phải một workflow riêng chạy sau bằng `workflow_run`. Theo tài liệu GitHub, `workflow_run` chạy trên nhánh mặc định và có secret dù workflow gốc không có. Ref đó không khớp luật tag `staging-v*` của environment `staging` (bài 54) nên bị từ chối, và muốn dùng đúng mã đã dựng thì phải checkout `head_sha` rồi chạy mã ấy cùng job với khoá. Đặt chung workflow thì job `distribute` đợi `[prepare, android, ios, notes]`, tải artifact của chính run theo tên có build number, và checkout đúng ref đã dựng:

### release-staging.distribute-jobs.example.yml · 55-app-distribution/native-config/.github/workflows/release-staging.distribute-jobs.example.yml (trích)

```yaml
  distribute:
    name: Distribute to the internal tester group (ubuntu)
    needs: [prepare, android, ios, notes]
    runs-on: ubuntu-24.04
    timeout-minutes: 20
    environment: staging
    env:
      BUILD_NUMBER: ${{ needs.prepare.outputs.build_number }}
      FIREBASE_CLI_VERSION: '15.35.0'
    steps:
      # Chỉ để đọc app-distribution.json: job này không chạy mã của repository.
      - uses: actions/checkout@v7.0.1
        with:
          persist-credentials: false
      - name: Download APK
        uses: actions/download-artifact@v8.0.2
        with:
          name: toi-noi-customer-staging-apk-${{ needs.prepare.outputs.build_number }}
          path: dist/apk
```

Job có secret không chạy mã của repository: việc cần `dart run` (release notes) nằm ở job `notes` không secret, và test cấm `dart`, `flutter`, `./script` trong `run` của `distribute`. Chạy tay phải chọn tag vì environment từ chối nhánh; phân phối lại dùng Re-run failed jobs. Test còn ghép đoạn mẫu vào workflow bài 54 rồi chạy bộ audit của lab 54: không có phát hiện nào, và thêm `pull_request` hay đưa secret lên cấp job thì bị bắt.

Bước phân phối là phần có bí mật: khoá đi qua `env`, được kiểm không rỗng, ghi ra file với `umask 077`, và `GOOGLE_APPLICATION_CREDENTIALS` trỏ tới nó:

### release-staging.distribute-jobs.example.yml · 55-app-distribution/native-config/.github/workflows/release-staging.distribute-jobs.example.yml (trích)

```yaml
        run: |
          set -euo pipefail
          umask 077
          : "${FIREBASE_SERVICE_ACCOUNT_JSON:?missing FIREBASE_SERVICE_ACCOUNT_JSON}"
          CREDENTIALS_FILE="$RUNNER_TEMP/firebase-service-account.json"
          printf '%s' "$FIREBASE_SERVICE_ACCOUNT_JSON" > "$CREDENTIALS_FILE"
          export GOOGLE_APPLICATION_CREDENTIALS="$CREDENTIALS_FILE"
          ANDROID_APP_ID="$(jq -er '.apps.customer.staging.android.appId' app-distribution.json)"
          IOS_APP_ID="$(jq -er '.apps.customer.staging.ios.appId' app-distribution.json)"
          GROUP="$(jq -er '.testerGroups.internal.alias' app-distribution.json)"
          firebase appdistribution:distribute "$APK_FILE" --app "$ANDROID_APP_ID" --groups "$GROUP" --release-notes-file "$NOTES_FILE"
          firebase appdistribution:distribute "$IPA_FILE" --app "$IOS_APP_ID" --groups "$GROUP" --release-notes-file "$NOTES_FILE"
```

Service account có vai trò **Firebase App Distribution Admin**; khoá JSON của nó cấp quyền quản trị App Distribution, nên chỉ nằm trong environment secret `staging` và bị xoá ở bước cuối với `if: always()`. Ta **không** dùng `FIREBASE_TOKEN`, `login:ci` hay `--token`: trang CLI của Firebase ghi `login:ci` là deprecated và `FIREBASE_TOKEN` không còn được khuyên dùng (kiểm tại 11/2026); trang hướng dẫn App Distribution vẫn còn `--token`, đừng lấy làm chuẩn. Khoá JSON là thông tin dài hạn nên vẫn là đánh đổi: ai lấy được nó dùng được tới khi ta thu hồi. Workload Identity Federation với Application Default Credentials là hướng cần thử; ta chưa kiểm Firebase CLI nhận kiểu đăng nhập đó.

Test chạy script này với `firebase` giả và kiểm điều lệnh giả thấy lúc bị gọi: khoá đúng nội dung, quyền `600`, nằm trong `RUNNER_TEMP`, không lộ ra log; thiếu file nhị phân thì dừng trước khi ghi khoá; APK lỗi thì IPA không được gửi. IPA tải lên từ runner ubuntu, không tốn phút macOS: mã nguồn CLI chỉ kiểm đuôi file rồi gọi API, ta chưa chạy thật.

## Release notes tự sinh từ commit

Tester không đọc `git log`, họ cần vài dòng tiếng Việt. Lớp `ReleaseNotes` lọc theo danh sách trắng: chỉ `feat`, `fix`, `perf` của Conventional Commits được vào. Kiểu khác (`chore`, `ci`, `test`...), commit merge và commit không theo quy ước là nội bộ. Thẻ `[skip-notes]` loại cả một `feat`, và với app khách, scope `driver` bị loại (ngược lại với app tài xế).

### release_notes.dart · 55-app-distribution/lib/src/release_notes.dart (trích)

```dart
NoteItem? classifyCommit(Commit commit, {required AppKind app}) {
  final subject = commit.subject.trim();
  if (_internalMarker.hasMatch(subject)) return null;
  final match = _conventional.firstMatch(subject);
  if (match == null) return null;

  final type = match.group(1)!.toLowerCase();
  final scope = match.group(2)?.trim().toLowerCase();
  final section = NoteSection.values.where((s) => s.type == type).firstOrNull;
  if (section == null) return null;

  // Commit của app kia không liên quan tới tester của app này.
  final otherApp = AppKind.values.firstWhere((a) => a != app);
  if (scope == otherApp.name) return null;

  final text = _cleanDescription(match.group(3)!);
  if (text.isEmpty) return null;
  return NoteItem(section, text);
}
```

Danh sách trắng không để commit tên lạ lọt ra ngoài; cái giá là commit thiếu tiền tố bị bỏ trong im lặng, nên log CI in dòng tóm tắt. SHA và tác giả không vào notes. Dữ liệu mẫu có 13 commit và cho 4 dòng; số cước do `calculateFare(2.001)` của `toi_noi_core` tính:

```text
Tới Nơi khách · staging 1.4.0 (build 42)

Tính năng mới
- Thêm đếm ngược khi tìm tài xế

Sửa lỗi
- Cước 2,001 km hiển thị đúng 12.005 VND
- Sửa nút gọi lại bị ẩn sau bàn phím

Hiệu năng
- Giảm số lần vẽ lại polyline
```

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 430" role="img" aria-label="Hình 2. Bốn bước sinh release notes. Git log có 13 commit dạng sha và subject. Bước lọc giữ feat, fix, perf, bỏ phần còn lại, bỏ thẻ skip-notes và scope của app kia: 5 giữ, 8 bỏ. Bước làm sạch bỏ type và scope, số PR, dấu chấm, viết hoa, bỏ trùng: còn 4 dòng. Bước cắt độ dài bỏ cả dòng từ cuối và thêm dòng tóm tắt khi vượt giới hạn: 4 dòng, không lược. Kết quả là file release-notes.txt như ví dụ.">
<defs><marker id="fl55-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="30" width="180" height="140" rx="10" class="f-surface s-muted"/><text x="34" y="60" class="t-b">git log</text><text x="34" y="86" class="t-small">13 commit</text><text x="34" y="106" class="t-small">sha, TAB, subject</text><text x="34" y="126" class="t-small">mẫu có cả merge</text>
<rect x="230" y="30" width="180" height="140" rx="10" class="f-pk-soft s-pk"/><text x="244" y="60" class="t-b">Lọc</text><text x="244" y="86" class="t-small">giữ feat, fix, perf</text><text x="244" y="106" class="t-small">bỏ phần còn lại</text><text x="244" y="126" class="t-small">bỏ [skip-notes]</text><text x="244" y="146" class="t-small">bỏ scope app kia</text>
<rect x="440" y="30" width="180" height="140" rx="10" class="f-pk-soft s-pk"/><text x="454" y="60" class="t-b">Làm sạch</text><text x="454" y="86" class="t-small">bỏ type(scope):</text><text x="454" y="106" class="t-small">bỏ (#PR), dấu chấm</text><text x="454" y="126" class="t-small">viết hoa chữ đầu</text><text x="454" y="146" class="t-small">bỏ dòng trùng</text>
<rect x="650" y="30" width="180" height="140" rx="10" class="f-pk-soft s-pk"/><text x="664" y="60" class="t-b">Cắt độ dài</text><text x="664" y="86" class="t-small">tối đa 500 ký tự</text><text x="664" y="106" class="t-small">bỏ dòng từ cuối</text><text x="664" y="126" class="t-small">thêm dòng tóm tắt</text><text x="664" y="146" class="t-small">dòng dài: cắt ở từ</text>
<path d="M202 100 L228 100" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-2-a)"/>
<path d="M412 100 L438 100" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-2-a)"/>
<path d="M622 100 L648 100" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-2-a)"/>
<text x="230" y="192" class="t-b">5 giữ, 8 bỏ</text><text x="440" y="192" class="t-b">4 dòng, 1 trùng</text><text x="650" y="192" class="t-b">4 dòng, 0 lược</text>
<path d="M810 172 L810 238" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-2-a)"/>
<rect x="20" y="240" width="840" height="170" rx="10" class="f-surface s-line"/><text x="34" y="266" class="t-b">release-notes.txt (215 ký tự)</text>
<text x="34" y="292" class="t-mono">Tới Nơi khách · staging 1.4.0 (build 42)</text><text x="34" y="312" class="t-mono">Tính năng mới</text><text x="34" y="330" class="t-mono">- Thêm đếm ngược khi tìm tài xế</text><text x="34" y="348" class="t-mono">Sửa lỗi</text><text x="34" y="366" class="t-mono">- Cước 2,001 km hiển thị đúng 12.005 VND</text><text x="34" y="384" class="t-mono">- Sửa nút gọi lại bị ẩn sau bàn phím</text><text x="470" y="292" class="t-small">Ghi ra file, rồi truyền cho</text><text x="470" y="312" class="t-small">--release-notes-file</text><text x="470" y="342" class="t-small">Rút gọn hình: nhóm Hiệu năng</text><text x="470" y="362" class="t-small">(1 dòng) nằm cuối file thật</text>
</svg>
</div>
<figcaption>Hình 2. Từ 13 commit tới 4 dòng: 4 giữ, 1 trùng, 8 bị bỏ.</figcaption>
</figure>

Cắt độ dài là chỗ dễ sai. Mức 500 ký tự là lựa chọn biên tập cho vừa màn hình điện thoại: tài liệu REST không nêu giới hạn cho `releaseNotes.text` (kiểm tại 11/2026). Thuật toán bớt dòng cuối cho tới khi vừa, rồi thêm dòng "… và N thay đổi khác". Độ dài tính bằng `runes`, không bằng `length` (đơn vị UTF-16), vì emoji chiếm hai đơn vị và tiếng Việt dạng NFD tách chữ khỏi dấu. Một dòng vẫn quá dài thì `truncateAtWord` cắt ở khoảng trắng và lùi nếu điểm cắt rơi giữa chữ và dấu kết hợp:

### release_notes_test.dart · 55-app-distribution/test/release_notes_test.dart (trích)

```dart
    test('chuỗi NFD không để lại dấu kết hợp mồ côi', () {
      // "ê" ở dạng NFD là e + U+0302; lặp để có điểm cắt giữa chữ và dấu.
      final text = List.filled(20, 'e\u0302').join();
      for (var limit = 2; limit <= 12; limit++) {
        final out = truncateAtWord(text, limit);
        final runes = out.runes.toList();
        expect(runes.last, 0x2026, reason: 'limit=$limit');
```

Job `notes` lấy commit từ tag `staging-v*` gần nhất trước commit đã dựng: `SHA^` bỏ qua tag đứng ngay trên commit đó; chưa có tag thì lấy 30 commit cuối. Test tạo repo git tạm cho cả ba ca:

### workflow_scripts_test.dart · 55-app-distribution/test/workflow_scripts_test.dart (trích)

```dart
    test('commit đã dựng nằm trên tag mới: vẫn lấy từ tag trước đó', () {
      box.git(['tag', 'staging-v1.1.0']);
      final r = runStep();
      expect(r.exitCode, 0, reason: r.log);
      expect(commitsFed(), ['chore: bốn', 'feat: ba']);
    });
```

## iOS ad-hoc và vòng UDID

IPA ad-hoc chỉ chạy trên máy có UDID trong provisioning profile. Ta không dùng Enterprise: chương trình đó dành cho tổ chức từ 100 nhân viên trở lên, phân phối ứng dụng nội bộ cho nhân viên (kiểm tại 11/2026).

App Distribution gỡ phần khó nhất là thu UDID: tester mở email mời bằng Safari, cài Firebase profile và đăng ký thiết bị; App Distribution lấy UDID rồi gửi email cho thành viên project có quyền `firebase.projects.update`. Phần còn lại là việc của ta, và Hình 3 vẽ cả vòng.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 400" role="img" aria-label="Hình 3. Vòng đăng ký thiết bị iOS gồm sáu bước. Hàng trên từ trái sang phải: tester nhận lời mời và cài Firebase profile; tester đăng ký thiết bị để App Distribution lấy UDID; email báo UDID tới thành viên Firebase. Hàng dưới từ phải sang trái: người quản lý nhập UDID vào Apple Developer và chọn vào profile ad-hoc; build lại IPA bằng profile mới (CI của ta tăng build number) và phân phối lại; tester cài bản mới, trên iOS 16 trở lên phải bật Developer Mode. Giới hạn của Apple là 100 thiết bị mỗi dòng sản phẩm mỗi năm thành viên.">
<defs><marker id="fl55-3-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="30" width="260" height="130" rx="10" class="f-phys-soft s-phys"/><text x="34" y="58" class="t-b">1. Tester nhận lời mời</text><text x="34" y="84" class="t-small">mở email bằng Safari,</text><text x="34" y="104" class="t-small">cài Firebase profile</text>
<rect x="310" y="30" width="260" height="130" rx="10" class="f-phys-soft s-phys"/><text x="324" y="58" class="t-b">2. Đăng ký thiết bị</text><text x="324" y="84" class="t-small">App Distribution xin phép</text><text x="324" y="104" class="t-small">lấy UDID của máy</text>
<rect x="600" y="30" width="260" height="130" rx="10" class="f-phys-soft s-phys"/><text x="614" y="58" class="t-b">3. Email báo UDID</text><text x="614" y="84" class="t-small">gửi tới thành viên Firebase</text><text x="614" y="104" class="t-small">có quyền nhận cảnh báo</text>
<rect x="600" y="200" width="260" height="130" rx="10" class="f-warn-soft s-warn"/><text x="614" y="228" class="t-b">4. Apple Developer</text><text x="614" y="254" class="t-small">thêm UDID vào Devices,</text><text x="614" y="274" class="t-small">chọn vào profile ad-hoc</text>
<rect x="310" y="200" width="260" height="130" rx="10" class="f-pk-soft s-pk"/><text x="324" y="228" class="t-b">5. Build lại IPA</text><text x="324" y="254" class="t-small">tải profile mới, build lại;</text><text x="324" y="274" class="t-small">CI của ta tăng build number</text>
<rect x="20" y="200" width="260" height="130" rx="10" class="f-phys-soft s-phys"/><text x="34" y="228" class="t-b">6. Tester cài bản mới</text><text x="34" y="254" class="t-small">email báo có bản mới;</text><text x="34" y="274" class="t-small">iOS 16+: bật Developer Mode</text>
<path d="M282 95 L308 95" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-3-a)"/>
<path d="M572 95 L598 95" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-3-a)"/>
<path d="M730 162 L730 198" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-3-a)"/>
<path d="M598 265 L572 265" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-3-a)"/>
<path d="M308 265 L282 265" class="f-none s-muted" stroke-width="2" marker-end="url(#fl55-3-a)"/>
<rect x="20" y="352" width="840" height="36" rx="8" class="f-warn-soft s-warn"/><text x="34" y="375" class="t-small">Apple: 100 thiết bị mỗi dòng sản phẩm mỗi năm thành viên, iPhone và iPad riêng (11/2026)</text>
</svg>
</div>
<figcaption>Hình 3. Mỗi thiết bị mới đi hết vòng này một lần; bước 4 và 5 không tự động.</figcaption>
</figure>

Lane `download_udids` xuất UDID thành CSV để nhập hàng loạt vào Apple Developer, và ghi vào `build/` (đã trong `.gitignore`) vì UDID là mã định danh thiết bị của người thật. Lane phân phối iOS nằm cạnh nó:

### Distribute.fastfile · 55-app-distribution/native-config/ios/fastlane/Distribute.fastfile (trích)

```ruby
def distro_credentials
  path = ENV["GOOGLE_APPLICATION_CREDENTIALS"].to_s.strip
  UI.user_error!("Thiếu GOOGLE_APPLICATION_CREDENTIALS (đường dẫn file service account)") if path.empty?
  path
end

platform :ios do
  desc "Phân phối IPA ad-hoc của flavor staging cho nhóm tester nội bộ."
  lane :distribute_staging do |options|
    app_id, group = distro_target(options[:app].to_s, "ios")
    ipa = options[:ipa].to_s
    UI.user_error!("ipa:<đường dẫn> là bắt buộc") if ipa.empty?

    firebase_app_distribution(
      app: app_id,
      ipa_path: ipa,
      groups: group,
      release_notes_file: options[:notes_file].to_s,
      service_credentials_file: distro_credentials,
    )
  end

  desc "Xuất UDID thiết bị tester đã đăng ký thành CSV để nhập vào Apple Developer."
  lane :download_udids do |options|
    app_id, _group = distro_target(options[:app].to_s, "ios")
    # Số dự án là đoạn thứ hai của App ID (1:<số dự án>:ios:<mã>). Plugin 1.0.0
    # đánh dấu tham số app: là deprecated cho action này, dù trang Firebase còn ghi.
    project_number = app_id.split(":")[1]
    # build/ nằm trong .gitignore: UDID là mã định danh thiết bị của người thật.
    firebase_app_distribution_get_udids(
      project_number: project_number,
      output_file: "../build/udids.csv",
      service_credentials_file: distro_credentials,
    )
  end
end
```

Plugin 1.0.0 đánh dấu `app:` của `download_udids` là deprecated (dùng `project_number`, lấy từ App ID) dù trang Firebase còn ghi `app:`. Ta truyền `service_credentials_file` tường minh vì theo mã nguồn plugin (nhánh `main`), khi không có tham số nào plugin thử `FIREBASE_TOKEN`, rồi đăng nhập Firebase CLI lưu trên máy, rồi mới tới Application Default Credentials: trên máy cá nhân lane chạy bằng tài khoản Google của bạn, và lỗi quyền chỉ lộ ra trên CI. Plugin cần Ruby từ 3.2.

Để kiểm máy đã vào profile chưa, dùng công cụ của bài 52 với `--device=<UDID>`. Từ iOS 16, mở app ad-hoc mà chưa bật Developer Mode sẽ báo lỗi. Và theo Firebase, build lại chỉ để thêm thiết bị mà giữ nguyên phiên bản, build number và mã thì không tạo release mới, chỉ báo người dùng thiết bị mới; CI của ta luôn tăng build number nên build lại nào cũng là release mới và báo mọi người.

## Feedback, crash và so với TestFlight

Trên Android, App Distribution có SDK feedback (bản beta): tester chạm vào thông báo hoặc nút do ta tự làm, SDK chụp màn hình rồi mở form, và phản hồi hiện ở tab Tester feedback của release. Trên iOS, trang feedback hiện hành chỉ mô tả SDK Android (kiểm tại 11/2026). Flutter không có plugin chính thức cho SDK này: gói `firebase_app_distribution` trên pub.dev do bên thứ ba duy trì và chỉ lo cập nhật trong app, nên lab không dùng.

Bản đầy đủ của SDK Android chứa tính năng tự cập nhật, và tài liệu cảnh báo nó có thể vi phạm chính sách Google Play dù mã không chạy: app có nó gửi lên Play có thể bị gỡ. Vì vậy bản "api" vào mọi variant, bản đầy đủ chỉ vào `staging`:

### app-distribution.gradle.kts.example · 55-app-distribution/native-config/android/app-distribution.gradle.kts.example

```kotlin
dependencies {
    implementation("com.google.firebase:firebase-appdistribution-api:16.0.0-beta21")
    "stagingImplementation"("com.google.firebase:firebase-appdistribution:16.0.0-beta21")
}
```

Test khẳng định bản đầy đủ chỉ xuất hiện một lần, qua `stagingImplementation`. Test không build nên chưa chứng minh Gradle nhận cú pháp; việc gọi `showFeedbackNotification()` cần source set riêng cho `staging`, lab chưa làm.

Crash đi đường khác: theo tài liệu, app có Crashlytics thì có số liệu ổn định cho mọi bản build phân phối; bản `staging` cần bật nó (bài 48) và giữ symbols như ở bài 54. Bảng dưới so với TestFlight (kiểm tại 11/2026); bài 56 đi sâu hơn.

| | App Distribution | TestFlight |
|---|---|---|
| Nền tảng | iOS và Android | Nền tảng Apple |
| IPA | Ad-hoc, UDID trong profile | Ký cho App Store Connect (bài 52) |
| Tester tối đa | 500 mỗi project, 200 mỗi nhóm | 100 nội bộ, 10.000 ngoài |
| Duyệt trước | Không | Tester ngoài: bản đầu của mỗi version có thể cần duyệt |
| Hết hạn bản build | 150 ngày | 90 ngày |
| Cài đặt | Firebase profile, web clip hoặc tester app | App TestFlight, email mời hoặc link |
| Feedback | SDK Android (beta), iOS không có | Qua app TestFlight hoặc chụp màn hình |

Quy tắc của Tới Nơi: App Distribution cho vòng nhanh của đội QA; TestFlight cho tester ngoài và bản đi đúng đường App Store. Hai IPA khác nhau vì khác phương thức export.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi phân phối nội bộ</span>
<ul>
<li><strong>IPA không cài được.</strong> Dấu hiệu: cài lỗi, hoặc báo "Developer Mode Required". Nguyên nhân: UDID chưa có trong profile của build đó, hoặc iOS 16+ chưa bật Developer Mode. Cách sửa: đi hết vòng Hình 3, rồi bật Developer Mode.</li>
<li><strong>Sai tài khoản hoặc token cũ.</strong> Dấu hiệu: CLI cảnh báo token deprecated, hoặc lane chạy trên máy bạn mà lỗi quyền trên CI. Nguyên nhân: còn token, hoặc plugin lấy đăng nhập CLI có sẵn. Cách sửa: service account qua <code>GOOGLE_APPLICATION_CREDENTIALS</code>, truyền <code>service_credentials_file</code>.</li>
<li><strong>Tải AAB bị từ chối.</strong> Dấu hiệu: CLI báo project chưa liên kết Google Play. Nguyên nhân: AAB cần liên kết Play. Cách sửa: dùng APK cho staging, hoặc hoàn tất liên kết.</li>
<li><strong>Release biến mất khỏi dashboard.</strong> Dấu hiệu: tester không thấy bản cũ. Nguyên nhân: quá 150 ngày, hoặc vượt 1.000 release mỗi app. Cách sửa: phân phối bản mới, hoặc tải bản cũ về trước hạn.</li>
<li><strong>SDK đầy đủ lọt vào bản đưa lên Play.</strong> Dấu hiệu: bản prod có thư viện <code>firebase-appdistribution</code>. Nguyên nhân: khai báo bằng <code>implementation</code> thay vì <code>stagingImplementation</code>. Cách sửa: chỉ bản api cho mọi variant; test của lab bắt đúng lỗi này.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/55-app-distribution
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3 trên macOS Apple silicon: `dart analyze --fatal-infos` báo `No issues found!`, `dart test` chạy 123 test xanh. Mức: **Kiểm chứng cấu hình**. Test không dùng mạng hay thiết bị; cần `bash`, `git` và `jq`.

Test chia sáu nhóm: release notes (33), cấu hình `staging` (17), CLI (7), đoạn dán cùng audit của lab 54 (28), APK, fastlane, Gradle và quét bí mật (21), chạy script thật với `dart`, `firebase` giả (17). Thử phá ngược (bỏ `--no-merges`, `SHA^`, `always()`) thì ít nhất một test đỏ.

**Chưa làm**, nói thẳng:

- Chưa phân phối bản nào lên Firebase: chưa có service account, nhóm tester hay thiết bị. Workflow ghép chưa chạy trên GitHub, chưa chạy `actionlint`, chưa kiểm `download-artifact` thật.
- Chưa chạy lane fastlane (không có Ruby 3.2; `ruby -c` chỉ kiểm cú pháp), chưa build APK và IPA.
- Chưa có mã gọi SDK feedback, Crashlytics, hay tự động hoá vòng UDID; AAB chỉ được mô tả.

## Bài tập

1. Tester thứ hai đăng ký thiết bị, ta thêm UDID vào profile rồi bấm Re-run failed jobs cho run của build 42 để phân phối lại. Tester đó cài được không?
2. Ai đó thêm `FIREBASE_TOKEN: ${{ secrets.FIREBASE_TOKEN }}` vào `env` của job `distribute`. Hai kiểm tra nào của lab đỏ, và vì sao vẫn là sai dù phân phối chạy được?
3. Sáu commit: `feat(driver): bật đếm ngược`, `fix(customer): sửa nút gọi lại`, `chore: nâng lints`, `feat: thêm ảnh [skip-notes]`, `perf: giảm vẽ lại bản đồ`, `Sửa linh tinh`. Notes cho app khách có những dòng nào?

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Không. Chạy lại job gửi cùng IPA build 42, mà IPA mang profile cũ không có UDID mới. Phải build lại bằng profile đã cập nhật: một run mới, build number mới (Hình 3, bước 5).</p>
<p><strong>2.</strong> Test <code>không dùng FIREBASE_TOKEN, login:ci hay --token</code> đỏ vì chuỗi có trong file, và audit phạm vi secret của bài 54 đỏ vì secret nằm ở <code>env</code> cấp job, có mặt ở mọi bước kể cả action bên thứ ba. Token cũng là cách xác thực đã deprecated.</p>
<p><strong>3.</strong> Hai dòng: "Sửa nút gọi lại" ở nhóm Sửa lỗi và "Giảm vẽ lại bản đồ" ở nhóm Hiệu năng. Bốn commit còn lại bị bỏ: scope của app kia, <code>chore</code>, <code>[skip-notes]</code>, và commit không theo quy ước.</p>
</details>

Bài trước: [CI/CD build: GitHub Actions macOS và ubuntu, secrets, build number và artifact](/articles/flutter-ci-cd-github-actions-macos-build-ios-android-secrets). Bài tiếp: [TestFlight: internal/external testing, Beta App Review và export compliance](/articles/flutter-testflight-internal-external-testing-beta-review).

## Tài liệu gốc

- [Firebase App Distribution](https://firebase.google.com/docs/app-distribution): tổng quan, khả năng, liên kết tới CLI, fastlane, Gradle (firebase.google.com).
- [Distribute Android apps to testers using the Firebase CLI](https://firebase.google.com/docs/app-distribution/android/distribute-cli) và [iOS](https://firebase.google.com/docs/app-distribution/ios/distribute-cli): `appdistribution:distribute`, tham số, 150 ngày (firebase.google.com).
- [Authenticate with a service account](https://firebase.google.com/docs/app-distribution/authenticate-service-account) và [Firebase CLI reference](https://firebase.google.com/docs/cli): `GOOGLE_APPLICATION_CREDENTIALS`, `login:ci` và `FIREBASE_TOKEN` đã deprecated (firebase.google.com).
- [Add and remove testers](https://firebase.google.com/docs/app-distribution/manage-testers) và [Troubleshooting & FAQ](https://firebase.google.com/docs/app-distribution/troubleshooting): giới hạn tester, 1.000 release, 2.048 MiB, Developer Mode (firebase.google.com).
- [Register additional iOS devices](https://firebase.google.com/docs/app-distribution/register-additional-devices): UDID, `firebase_app_distribution_get_udids` (firebase.google.com).
- [Distribute Android apps using the console, tab AAB](https://firebase.google.com/docs/app-distribution/android/distribute-console?apptype=aab): liên kết Google Play, chứng chỉ thử (firebase.google.com).
- [Collect feedback from testers](https://firebase.google.com/docs/app-distribution/collect-feedback-from-testers): SDK feedback Android, cảnh báo chính sách Google Play (firebase.google.com).
- [Distribute iOS apps using fastlane](https://firebase.google.com/docs/app-distribution/ios/distribute-fastlane) và [Using plugins](https://docs.fastlane.tools/plugins/using-plugins/) (firebase.google.com, docs.fastlane.tools).
- [Events that trigger workflows: workflow_run](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows) (docs.github.com).
- [TestFlight overview](https://developer.apple.com/help/app-store-connect/test-a-beta-version/testflight-overview/), [Devices overview](https://developer.apple.com/help/account/devices/devices-overview/) và [Apple Developer Enterprise Program](https://developer.apple.com/programs/enterprise/) (developer.apple.com).
