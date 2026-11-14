---
slug: flutter-ci-cd-github-actions-macos-build-ios-android-secrets
title: "CI/CD build: GitHub Actions macOS và ubuntu, secrets, build number và artifact cho Tới Nơi"
summary: "Bài 54: workflow mẫu dựng AAB trên ubuntu và IPA trên macOS: build number từ run_number, keystore và certificate từ secrets, keychain tạm có dọn, artifact, chi phí phút so với Codemagic. Test Dart chạy script thật với secret giả. Chưa chạy trên GitHub."
category: flutter
level: advanced
tags: [flutter, dart, mobile, toi-noi, ios, android, release, ci, github-actions]
language: vi
---

Sáng thứ hai, tester của Tới Nơi xin bản staging mới của app khách. Người giữ keystore Android và chiếc Mac ký iOS là Lan, và hôm nay Lan nghỉ. Mọi thứ cần để build nằm trên laptop của Lan: `key.properties` ngoài repo (bài 51) và certificate trong keychain (bài 52).

CI/CD đưa cả quy trình lên một máy sạch, chạy bằng lệnh viết trong repository. Cái giá là bí mật phải đi lên máy đó. Bài này viết workflow GitHub Actions dựng AAB trên ubuntu và IPA trên macOS, lấy build number từ `run_number`, và xử lý phần khó nhất: để keystore, certificate, profile đi qua runner mà không lộ ra log.

Đây là bài 54 của lộ trình **Flutter qua Tới Nơi**. Ta nên đọc [bài 53 về fastlane](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build) trước: bài này đặt certificate và profile thẳng trong secret để thấy cơ chế, thay vì qua `match`. Lab dùng lại bộ kiểm workflow của [bài 33](/articles/flutter-ci-github-actions-analyze-test-golden-cache). Cách ký Android và iOS nằm ở bài 51 và 52.

Nói thẳng từ đầu: workflow nằm trong `native-config/`, **không** ở `.github/workflows/` của repository này, vì repository không có secret. Workflow này chưa chạy trên GitHub và máy chạy kiểm chứng không build được AAB hay IPA. Ta kiểm tính chất của file, và chạy thật từng script `run` bằng bash với secret giả.

<div class="tkh-callout tkh-callout--example">
<span class="tkh-callout-title">Thử trước khi đọc</span>
<p>Run số 57 dựng xong cả hai nền tảng. Ta bấm <strong>Re-run all jobs</strong>. Build number của lần chạy lại là bao nhiêu, và điều đó gây vấn đề gì nếu bản 57 đã lên store?</p>
<details><summary>Xem đáp án</summary><p>Vẫn là 57. Theo tài liệu GitHub, <code>github.run_number</code> không đổi khi chạy lại; chỉ <code>run_attempt</code> tăng. Tài liệu Android nói không được tải lên Play một APK có <code>versionCode</code> đã dùng, và <code>flutter build</code> yêu cầu mỗi build một số duy nhất. Chạy lại run đã phát hành sẽ trùng số cũ; muốn bản mới thì tạo run mới (tag mới hoặc chạy tay).</p></details>
</div>

## Một workflow, ba job, hai runner

Workflow có ba job. `prepare` tính build number một lần. `android` chạy trên ubuntu, `ios` chạy trên macOS, cả hai đợi `prepare`. Phần đầu file quyết định ai kích hoạt được và token làm được gì.

### release-staging.yml · 54-ci-cd-build/native-config/.github/workflows/release-staging.yml (trích)

```yaml
name: Release staging (mẫu)
on:
  workflow_dispatch:
  push:
    tags: ['staging-v*']
permissions:
  contents: read
concurrency:
  group: release-staging-${{ github.ref }}
  cancel-in-progress: false
```

Chỉ có hai cách kích hoạt: chạy tay và đẩy tag `staging-v*`. Không có `pull_request`: theo tài liệu, secret không được truyền cho workflow chạy từ fork (trừ `GITHUB_TOKEN`), và ta không mở môi trường có secret cho mọi nhánh. `workflow_dispatch` chỉ nhận sự kiện khi file nằm trên nhánh mặc định. `permissions` giữ `contents: read` như bài 33. Run đang build không bị huỷ vì huỷ giữa chừng chỉ lãng phí số phút đã chạy.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 380" role="img" aria-label="Hình 1. Luồng của workflow phát hành. Tag staging-v hoặc chạy tay kích hoạt job prepare trên ubuntu để tính build number. Từ prepare, hai job chạy song song: android trên ubuntu giải mã keystore, build appbundle và dọn keystore, cho ra artifact AAB và symbols; ios trên macOS cài keychain tạm và profile, build ipa và dọn keychain, cho ra artifact IPA và symbols. Hai job build nằm trong environment staging, giữ secret và chờ reviewer duyệt nếu gói cho phép.">
<defs><marker id="fl54-1-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-muted"/></marker></defs>
<rect x="20" y="120" width="170" height="110" rx="10" class="f-surface s-muted"/><text x="34" y="150" class="t-b">Kích hoạt</text><text x="34" y="176" class="t-small">tag staging-v*</text><text x="34" y="196" class="t-small">hoặc chạy tay</text>
<rect x="230" y="120" width="170" height="110" rx="10" class="f-pk-soft s-pk"/><text x="244" y="150" class="t-b">prepare</text><text x="244" y="176" class="t-small">ubuntu-24.04</text><text x="244" y="196" class="t-small">build number</text>
<rect x="450" y="30" width="240" height="130" rx="10" class="f-phys-soft s-phys"/><text x="464" y="58" class="t-b">android</text><text x="464" y="84" class="t-small">ubuntu-24.04, JDK 17</text><text x="464" y="104" class="t-small">giải mã keystore</text><text x="464" y="124" class="t-small">flutter build appbundle</text><text x="464" y="144" class="t-small">dọn keystore: always()</text>
<rect x="450" y="200" width="240" height="130" rx="10" class="f-phys-soft s-phys"/><text x="464" y="228" class="t-b">ios</text><text x="464" y="254" class="t-small">macos-26, Xcode 26.6</text><text x="464" y="274" class="t-small">keychain tạm + profile</text><text x="464" y="294" class="t-small">flutter build ipa</text><text x="464" y="314" class="t-small">dọn keychain: always()</text>
<rect x="735" y="30" width="125" height="130" rx="10" class="f-surface s-line"/><text x="749" y="58" class="t-b">Artifact</text><text x="749" y="84" class="t-small">AAB</text><text x="749" y="104" class="t-small">symbols</text><text x="749" y="124" class="t-small">giữ 14 ngày</text>
<rect x="735" y="200" width="125" height="130" rx="10" class="f-surface s-line"/><text x="749" y="228" class="t-b">Artifact</text><text x="749" y="254" class="t-small">IPA</text><text x="749" y="274" class="t-small">symbols</text><text x="749" y="294" class="t-small">giữ 14 ngày</text>
<path d="M192 175 L228 175" class="f-none s-muted" stroke-width="2" marker-end="url(#fl54-1-a)"/>
<path d="M402 175 C426 175 426 95 448 95" class="f-none s-muted" stroke-width="2" marker-end="url(#fl54-1-a)"/>
<path d="M402 175 C426 175 426 265 448 265" class="f-none s-muted" stroke-width="2" marker-end="url(#fl54-1-a)"/>
<path d="M692 95 L733 95" class="f-none s-muted" stroke-width="2" marker-end="url(#fl54-1-a)"/>
<path d="M692 265 L733 265" class="f-none s-muted" stroke-width="2" marker-end="url(#fl54-1-a)"/>
<rect x="20" y="344" width="840" height="30" rx="8" class="f-warn-soft s-warn"/><text x="34" y="364" class="t-small">Environment staging: giữ secret, giới hạn tag, reviewer duyệt nếu gói cho phép</text>
</svg>
</div>
<figcaption>Hình 1. Ba job: prepare cấp một build number cho cả hai nền tảng; android và ios chạy song song trên hai loại runner.</figcaption>
</figure>

## Build number từ run_number

Android gọi số này là `versionCode`, iOS gọi là `CFBundleVersion`. Cờ `--build-number` của `flutter build` đặt cả hai (theo `flutter build ipa --help`), còn tên phiên bản lấy từ `version: x.y.z+build` trong `pubspec.yaml`, nên workflow không hard-code `--build-name`. Hai nền tảng dùng chung một số nên `prepare` tính một lần.

### release-staging.yml · 54-ci-cd-build/native-config/.github/workflows/release-staging.yml (trích)

```yaml
      - name: Compute build number
        id: number
        env:
          RUN_NUMBER: ${{ github.run_number }}
          OFFSET: ${{ vars.BUILD_NUMBER_OFFSET || '0' }}
        run: |
          for v in "$RUN_NUMBER" "$OFFSET"; do
            case "$v" in
              ''|*[!0-9]*) echo "RUN_NUMBER and OFFSET must be non-negative integers" >&2; exit 1 ;;
            esac
            if [ "${#v}" -gt 10 ]; then echo "value too long" >&2; exit 1; fi
          done
          BUILD_NUMBER=$((10#$RUN_NUMBER + 10#$OFFSET))
          if [ "$BUILD_NUMBER" -lt 1 ] || [ "$BUILD_NUMBER" -gt 2100000000 ]; then
            echo "build number $BUILD_NUMBER is outside 1..2100000000" >&2
            exit 1
          fi
          echo "build_number=$BUILD_NUMBER" >> "$GITHUB_OUTPUT"
          echo "Build number: $BUILD_NUMBER" >> "$GITHUB_STEP_SUMMARY"
```

`run_number` bắt đầu từ 1 cho mỗi workflow và tăng với mỗi run mới (tài liệu contexts): một nguồn số tăng dần miễn phí. Ba chỗ cần để ý:

- **Offset.** Mỗi workflow có bộ đếm riêng, nên workflow mới bắt đầu lại từ 1. Nếu store đã nhận số cao hơn, biến repository `BUILD_NUMBER_OFFSET` bù vào. Biểu thức của Actions không có phép cộng nên phép cộng nằm trong shell. Chỉ được tăng offset; giảm offset sẽ làm build number giảm theo.
- **Trần.** Google Play cho `versionCode` tối đa 2.100.000.000 (developer.android.com, kiểm tại 11/2026), và script chặn ở đó. `10#` ép số thập phân để `008` không bị hiểu là số bát phân.
- **Không dùng `run_attempt`, `run_id`, SHA hay giờ.** Test cấm các chuỗi đó: số phải tăng theo thứ tự phát hành, không theo commit.

Test chạy chính script này bằng bash với `RUN_NUMBER` từ 1 đến 54321 và so với hàm Dart `computeBuildNumber`:

### step_scripts_test.dart · 54-ci-cd-build/test/step_scripts_test.dart (trích)

```dart
      test('chạy lại cùng run_number cho đúng cùng số (re-run không đổi số)', () {
        final a = compute('42', '0').$2;
        final b = compute('42', '0').$2;
        expect(a, '42');
        expect(b, a);
      }, skip: skip);
```

## Job Android: keystore từ secret

Upload keystore được lưu trong secret của GitHub (không nằm trong repository) dưới dạng chuỗi base64. Cờ của lệnh mã hoá khác nhau giữa hai hệ điều hành: macOS `base64 -i file -o file.b64`, Linux `base64 -w 0 file > file.b64`; đặt secret bằng `gh secret set TÊN --env staging < file.b64`. Bước đầu tiên giải mã chuỗi này ra thư mục tạm của runner và ghi `android/key.properties` mà `signingConfigs` (bài 51) đọc. Mỗi giá trị đi qua `env` của bước, không bao giờ nằm trong thân script.

### release-staging.yml · 54-ci-cd-build/native-config/.github/workflows/release-staging.yml (trích)

```yaml
      - name: Restore upload keystore
        env:
          ANDROID_KEYSTORE_BASE64: ${{ secrets.ANDROID_KEYSTORE_BASE64 }}
          ANDROID_KEYSTORE_PASSWORD: ${{ secrets.ANDROID_KEYSTORE_PASSWORD }}
          ANDROID_KEY_ALIAS: ${{ secrets.ANDROID_KEY_ALIAS }}
          ANDROID_KEY_PASSWORD: ${{ secrets.ANDROID_KEY_PASSWORD }}
        run: |
          set -euo pipefail
          umask 077
          : "${ANDROID_KEYSTORE_BASE64:?ANDROID_KEYSTORE_BASE64 is empty}"
          : "${ANDROID_KEYSTORE_PASSWORD:?ANDROID_KEYSTORE_PASSWORD is empty}"
          : "${ANDROID_KEY_ALIAS:?ANDROID_KEY_ALIAS is empty}"
          : "${ANDROID_KEY_PASSWORD:?ANDROID_KEY_PASSWORD is empty}"
          KEYSTORE_PATH="$RUNNER_TEMP/upload-keystore.jks"
          PROPS=android/key.properties
          printf '%s' "$ANDROID_KEYSTORE_BASE64" | base64 --decode > "$KEYSTORE_PATH"
          : > "$PROPS"
          printf 'storeFile=%s\n' "$KEYSTORE_PATH" >> "$PROPS"
          printf 'storePassword=%s\n' "$ANDROID_KEYSTORE_PASSWORD" >> "$PROPS"
          printf 'keyAlias=%s\n' "$ANDROID_KEY_ALIAS" >> "$PROPS"
          printf 'keyPassword=%s\n' "$ANDROID_KEY_PASSWORD" >> "$PROPS"
```

Bốn dòng `: "${VAR:?...}"` chặn secret rỗng hoặc chưa đặt: thiếu chúng, secret rỗng vẫn thoát 0 và sinh keystore 0 byte (test chạy với từng secret rỗng và thiếu). `umask 077` để chỉ chủ sở hữu đọc được hai file; test kiểm quyền `600`. Mọi dòng nhắc biến bí mật đều có `|` hoặc `>>`, nên giá trị đi vào file hoặc đường ống, không ra stdout. Bước build theo sau:

```yaml
      - name: Build AAB
        run: |
          flutter pub get --enforce-lockfile
          flutter build appbundle --release --flavor staging \
            --dart-define-from-file=config/staging.json \
            --build-number="$BUILD_NUMBER" \
            --obfuscate --split-debug-info=build/symbols/android
```

`--flavor staging` và `config/staging.json` phải cùng tên; test mở file cấu hình của `toi-noi-app` và kiểm `FLAVOR` là `staging`. `--obfuscate` phải đi cùng `--split-debug-info` (theo `--help`), và thư mục symbols được upload để dịch stack trace sau này (bài 42). JDK cài bằng `actions/setup-java`, Temurin 17. Image ubuntu-24.04 đã có Java 17 mặc định và Android SDK (readme runner image, kiểm tại 11/2026); ghim rõ ràng để việc đổi image không làm đổi JDK. Bước `Remove keystore and key.properties` (`if: always()`) đứng ngay sau build, trước các bước upload. AAB nằm ở `build/app/outputs/bundle/stagingRelease/`, đường dẫn suy ra từ source công cụ Flutter 3.47.4 chứ chưa từ một lần build.

## Job iOS: keychain tạm, profile và việc dọn

Trên macOS, `codesign` lấy khoá từ keychain. Ta không đụng keychain đăng nhập của runner: tạo một keychain riêng cho job, nhập `.p12` vào đó, rồi chép profile vào hai thư mục: `~/Library/MobileDevice/Provisioning Profiles` theo hướng dẫn của GitHub, và `~/Library/Developer/Xcode/UserData/Provisioning Profiles`, nơi Flutter 3.47.4 tìm profile (`code_signing.dart`, bài 52). Ta chưa kiểm được Xcode trên runner thật đọc thư mục nào trong hai thư mục này, nên chép cả hai. Các lệnh `security` theo hướng dẫn "Installing an Apple certificate on macOS runners" của GitHub.

### release-staging.yml · 54-ci-cd-build/native-config/.github/workflows/release-staging.yml (trích)

```yaml
      - name: Install certificate and provisioning profile
        env:
          IOS_CERTIFICATE_P12_BASE64: ${{ secrets.IOS_CERTIFICATE_P12_BASE64 }}
          IOS_CERTIFICATE_PASSWORD: ${{ secrets.IOS_CERTIFICATE_PASSWORD }}
          IOS_PROVISIONING_PROFILE_BASE64: ${{ secrets.IOS_PROVISIONING_PROFILE_BASE64 }}
        run: |
          set -euo pipefail
          umask 077
          : "${IOS_CERTIFICATE_P12_BASE64:?IOS_CERTIFICATE_P12_BASE64 is empty}"
          : "${IOS_CERTIFICATE_PASSWORD:?IOS_CERTIFICATE_PASSWORD is empty}"
          : "${IOS_PROVISIONING_PROFILE_BASE64:?IOS_PROVISIONING_PROFILE_BASE64 is empty}"
          CERT_PATH="$RUNNER_TEMP/distribution.p12"
          PROFILE_NAME=toi-noi-staging.mobileprovision
          KEYCHAIN_PATH="$RUNNER_TEMP/toi-noi-signing.keychain-db"
          KEYCHAIN_PASSWORD="$(openssl rand -hex 16)"
          printf '%s' "$IOS_CERTIFICATE_P12_BASE64" | base64 --decode > "$CERT_PATH"
          for dir in "$HOME/Library/MobileDevice/Provisioning Profiles" "$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles"; do
            mkdir -p "$dir"
            printf '%s' "$IOS_PROVISIONING_PROFILE_BASE64" | base64 --decode > "$dir/$PROFILE_NAME"
          done
          security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security import "$CERT_PATH" -P "$IOS_CERTIFICATE_PASSWORD" -A -t cert -f pkcs12 -k "$KEYCHAIN_PATH"
          security set-key-partition-list -S apple-tool:,apple: -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security list-keychain -d user -s "$KEYCHAIN_PATH"
          rm -f "$CERT_PATH"
```

Mật khẩu keychain sinh ngẫu nhiên mỗi run, không cần secret. Bước build gọi `flutter build ipa --release --flavor staging` với `--export-options-plist=ios/ExportOptions-adhoc.plist`, tệp mẫu của bài 52 (profile ad-hoc cho bài 55). Đi qua lane của bài 53 thì hai bước ký được thay bằng `bundle exec fastlane ios beta`, còn secret đổi thành khoá App Store Connect API, mật khẩu `match` và quyền đọc repo chứa chứng chỉ (`MATCH_GIT_BASIC_AUTHORIZATION` hoặc `MATCH_GIT_PRIVATE_KEY`).

Ngay sau bước build là bước dọn, đứng trước mọi bước upload để file ký không còn trên đĩa khi các action khác chạy:

```yaml
      - name: Clean up keychain and provisioning profile
        if: always()
        run: |
          KEYCHAIN_PATH="$RUNNER_TEMP/toi-noi-signing.keychain-db"
          if [ -f "$KEYCHAIN_PATH" ]; then security delete-keychain "$KEYCHAIN_PATH"; fi
          rm -f "$RUNNER_TEMP/distribution.p12" "$HOME/Library/MobileDevice/Provisioning Profiles/toi-noi-staging.mobileprovision" "$HOME/Library/Developer/Xcode/UserData/Provisioning Profiles/toi-noi-staging.mobileprovision"
```

Trên runner GitHub-hosted, máy ảo bị huỷ khi job xong, nên bước này chỉ mang tính phòng thủ: tài liệu GitHub nói bước dọn dành cho self-hosted runner, nơi keychain và profile có thể còn lại. Nếu sau này chuyển iOS sang self-hosted Mac, bước dọn là bắt buộc. `if: always()` là chỗ quyết định: mặc định một bước chỉ chạy khi các bước trước thành công, mà ta cần dọn đúng lúc build đỏ.

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 330" role="img" aria-label="Hình 2. Bốn bước của job iOS: cài keychain, build ipa, dọn, upload. Đường liền đi theo thứ tự khi mọi thứ thành công. Đường đứt từ hai bước đầu đi tới bước dọn: bước nào đỏ thì job dừng nhưng bước dọn có if always nên vẫn chạy. Dải bên dưới cho thấy keychain tạm và profile nằm trên đĩa từ bước đầu tới hết bước dọn, trước khi upload.">
<defs><marker id="fl54-2-a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-phys"/></marker><marker id="fl54-2-w" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto"><path d="M0 0 L10 5 L0 10 z" class="f-warn"/></marker></defs>
<rect x="20" y="40" width="170" height="100" rx="10" class="f-phys-soft s-phys"/><text x="34" y="70" class="t-b">Cài keychain</text><text x="34" y="96" class="t-small">import p12 vào</text><text x="34" y="116" class="t-small">keychain tạm</text>
<rect x="240" y="40" width="170" height="100" rx="10" class="f-phys-soft s-phys"/><text x="254" y="70" class="t-b">Build IPA</text><text x="254" y="96" class="t-small">ký bằng cert</text><text x="254" y="116" class="t-small">và profile</text>
<rect x="460" y="40" width="170" height="100" rx="10" class="f-pk-soft s-pk"/><text x="474" y="70" class="t-b">Dọn</text><text x="474" y="96" class="t-small">delete-keychain</text><text x="474" y="116" class="t-small">xoá profile, p12</text>
<rect x="680" y="40" width="170" height="100" rx="10" class="f-phys-soft s-phys"/><text x="694" y="70" class="t-b">Upload</text><text x="694" y="96" class="t-small">IPA + symbols</text>
<path d="M192 90 L238 90" class="f-none s-phys" stroke-width="2" marker-end="url(#fl54-2-a)"/>
<path d="M412 90 L458 90" class="f-none s-phys" stroke-width="2" marker-end="url(#fl54-2-a)"/>
<path d="M632 90 L678 90" class="f-none s-phys" stroke-width="2" marker-end="url(#fl54-2-a)"/>
<path d="M105 142 L105 200 L545 200 L545 144" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5" marker-end="url(#fl54-2-w)"/>
<path d="M325 142 L325 200" class="f-none s-warn" stroke-width="2" stroke-dasharray="6 5"/>
<text x="20" y="232" class="t-small">Bước nào đỏ thì job dừng, nhưng bước dọn có if: always() nên vẫn chạy</text>
<rect x="20" y="256" width="610" height="46" rx="8" class="f-warn-soft s-warn"/><text x="34" y="284" class="t-small">Keychain tạm và profile nằm trên đĩa từ bước đầu tới hết bước dọn</text>
</svg>
</div>
<figcaption>Hình 2. Đường đứt là đường lỗi: bước dọn chạy ngay sau build, dù cài đặt hay build đỏ, và trước khi upload.</figcaption>
</figure>

Test chạy hai script này với `security` giả (ghi lại mỗi lệnh, tạo và xoá file keychain) và `HOME` trong thư mục tạm. Test case đáng giá nhất là cho `import` thất bại:

### step_scripts_test.dart · 54-ci-cd-build/test/step_scripts_test.dart (trích)

```dart
    test('import lỗi giữa chừng: p12 và keychain ở lại, bước dọn gỡ sạch', () {
      final failed = runInstall(fail: 'import');
      expect(failed.exitCode, isNot(0));
      // Đây là lý do bước dọn cần if: always(): script cài đặt đã dừng.
```

Script cài đặt dừng giữa chừng và để lại `.p12`, keychain, profile; bước dọn gỡ sạch cả ba, và chạy lại lần hai vẫn thoát 0.

## Secret không lộ ra log

Mỗi quy tắc dưới đây là tính chất kiểm được của file. Lab 33 đã kiểm ghim Flutter, ref của action và chuỗi giống bí mật; lab này thêm các quy tắc riêng.

| Quy tắc | Vì sao |
|---|---|
| Secret chỉ nằm trong `env` của bước `run`, không ở `env` cấp workflow hay job, không trong `with` của action | Action bên thứ ba và các bước khác không nhận secret qua `env`; file đã ghi ra đĩa thì xoá ngay sau build |
| Không có `${{ ... }}` trong thân `run` | Giá trị đi qua biến môi trường, không bị ghép vào mã shell |
| Không `echo`, `printf`, `cat` biến secret ra stdout, stderr hay `GITHUB_ENV`; không `set -x`, không in cả môi trường, không `cat` file ghi bí mật | Flutter docs cảnh báo đừng echo lại biến vào console; ta không dựa vào việc log tự che |
| Job có secret phải có `environment: staging` | Reviewer và giới hạn tag gắn vào environment; secret chỉ được nạp sau khi job vượt qua luật bảo vệ |
| Không `id-token`, không quyền ghi | Workflow không gọi cloud, không cần OIDC |

Environment cho phép đặt tối đa 6 reviewer, chặn tự duyệt và giới hạn tag được deploy; job chỉ đọc được environment secret sau khi các luật bảo vệ đã qua. Giới hạn gói (docs.github.com, kiểm tại 11/2026): với Free, Pro và Team, required reviewers chỉ có ở repository công khai, repository riêng cần Enterprise; luật giới hạn tag thì Pro và Team dùng được ở repository riêng. Chỉ có luật tag thì không có người duyệt: ai đẩy được tag `staging-v*` đều kích hoạt được job ký, nên phải hạn chế quyền tạo tag. Chạy tay từ nhánh sẽ bị từ chối; hãy chọn tag ở ô "Use workflow from". Reviewer và secret thật nằm trong Settings chứ không trong YAML, nên test không kiểm được. OIDC đổi token ngắn hạn lấy secret dài hạn, nhưng tài liệu chỉ nêu AWS, Azure, GCP và không nhắc Apple: certificate và profile vẫn là secret tĩnh.

Kiểm tĩnh chưa đủ, nên test còn chạy script Android với secret giả và khẳng định log không chứa giá trị nào. Để chắc phép thử bắt được lỗi, một test case sai thay `printf` bằng `echo` và đòi giá trị **phải** hiện ra stdout. Phép khẳng định chính:

### step_scripts_test.dart · 54-ci-cd-build/test/step_scripts_test.dart (trích)

```dart
        for (final value in secrets.values) {
          expect(result.log, isNot(contains(value)), reason: 'lộ $value');
        }
```

## Artifact, chi phí phút và Codemagic

Mỗi job upload hai artifact: AAB hoặc IPA, và thư mục symbols. Tên chứa build number để truy từ một artifact về đúng run và đúng bản build trên store. `if-no-files-found: error` biến đường dẫn sai thành lỗi, và `retention-days: 14` giới hạn thời gian lưu artifact. Theo tài liệu, ai có quyền đọc repository đều tải được artifact, nên bản staging chỉ nên nằm ở repository riêng.

Giá theo phút của runner chuẩn trên trang billing của GitHub, **kiểm tại 11/2026**; Codemagic lấy từ trang pricing của họ. Thời gian job (15 phút Android, 30 phút iOS) là giả định minh hoạ, chưa đo.

| Dịch vụ | Linux | macOS | Một lần phát hành |
|---|---|---|---|
| GitHub Actions | 0,006 USD/phút (2 nhân) | 0,062 USD/phút (3 hoặc 4 nhân) | 1,95 USD |
| Codemagic (pay as you go) | 0,045 USD/phút (Linux X2) | 0,095 USD/phút (Mac mini M2) | 3,53 USD |

<figure class="tkh-fig">
<div class="tkh-fig-box">
<svg viewBox="0 0 880 240" role="img" aria-label="Hình 3. Chi phí một lần phát hành gồm 15 phút Linux và 30 phút macOS. GitHub Actions: 1,95 USD, phần Linux rất nhỏ, phần macOS chiếm gần hết. Codemagic pay as you go: 3,53 USD, phần Linux 0,68 USD và phần macOS 2,85 USD. Đơn giá kiểm tại 11/2026, số phút là minh hoạ.">
<text x="20" y="30" class="t-small">GitHub: 15 phút Linux + 30 phút macOS</text>
<rect x="20" y="40" width="15" height="40" rx="3" class="f-phys-soft s-phys"/><rect x="35" y="40" width="317" height="40" rx="3" class="f-pk-soft s-pk"/><text x="370" y="66" class="t-b">1,95 USD</text>
<text x="20" y="110" class="t-small">Codemagic: 15 phút Linux X2 + 30 phút Mac mini M2</text>
<rect x="20" y="120" width="115" height="40" rx="3" class="f-phys-soft s-phys"/><rect x="135" y="120" width="485" height="40" rx="3" class="f-pk-soft s-pk"/><text x="640" y="146" class="t-b">3,53 USD</text>
<rect x="20" y="190" width="18" height="18" rx="3" class="f-phys-soft s-phys"/><text x="46" y="204" class="t-small">Linux</text>
<rect x="120" y="190" width="18" height="18" rx="3" class="f-pk-soft s-pk"/><text x="146" y="204" class="t-small">macOS</text>
<text x="240" y="204" class="t-small">Đơn giá kiểm tại 11/2026; số phút là minh hoạ</text>
</svg>
</div>
<figcaption>Hình 3. Phút macOS chiếm khoảng 95% hoá đơn GitHub của lần phát hành này; phần Linux gần như không đáng kể.</figcaption>
</figure>

Tỷ lệ macOS trên Linux của GitHub là 62 trên 6, khoảng 10,3 lần, khớp bảng ở bài 33. Repository công khai chạy runner chuẩn miễn phí; repository riêng ở gói Free có 12 USD included mỗi tháng, đủ 6 lần phát hành như trên (chỉ dựng iOS 30 phút thì cũng 6 lần, vì 12.000 chia 1.860). Codemagic có gói miễn phí 500 phút Mac M2 mỗi tháng cho một người dùng (trang giá không ghi ngày, kiểm tại 11/2026): 16 lần dựng iOS 30 phút, cùng đơn vị với con số 6 của GitHub. Gói pay as you go có một build song song, mỗi build song song thêm tốn 49 USD một tháng, tối đa ba.

Đơn giá phút không phải tất cả. GitHub rẻ hơn theo đơn giá, nhưng ta tự viết phần ký (bài này) và tự lo bước tải lên store. Docs.flutter.dev xếp Codemagic vào nhóm tích hợp sẵn chức năng cho Flutter, và tài liệu Codemagic mô tả việc đẩy lên App Store Connect bằng API key qua giao diện. Ta không so tốc độ vì phần cứng khác nhau và chưa đo job nào. Chọn theo ba câu hỏi: repository ở đâu, mỗi tháng dựng iOS bao nhiêu lần, và đội có muốn tự giữ phần ký không.

## Lỗi thường gặp

<div class="tkh-callout tkh-callout--warn">
<span class="tkh-callout-title">Năm lỗi hay gặp khi đưa build lên CI</span>
<ul>
<li><strong>Build number trùng khi chạy lại.</strong> Dấu hiệu: store từ chối bản tải lên vì số đã dùng. Nguyên nhân: <code>run_number</code> không đổi khi re-run. Cách sửa: dựng bản mới bằng run mới; có số cũ trên store thì đặt <code>BUILD_NUMBER_OFFSET</code>.</li>
<li><strong>Secret xuất hiện trong log.</strong> Dấu hiệu: log có giá trị hoặc lệnh đã ghép sẵn giá trị. Nguyên nhân: <code>echo</code> để gỡ lỗi, <code>set -x</code>, nội suy <code>${{ secrets.X }}</code> vào thân script. Cách sửa: đưa secret qua <code>env</code>, chỉ pipe hoặc chuyển hướng ra file.</li>
<li><strong>Secret mở cho mọi nhánh và PR.</strong> Dấu hiệu: ai đẩy nhánh cũng chạy được job ký. Nguyên nhân: secret cấp repository, workflow có <code>pull_request</code> hay <code>push</code> nhánh. Cách sửa: environment secret kèm giới hạn tag (và reviewer nếu gói cho phép), trigger chỉ chạy tay và tag.</li>
<li><strong>Bước dọn không chạy khi build đỏ.</strong> Dấu hiệu: trên self-hosted runner còn keychain và profile của lần trước. Nguyên nhân: thiếu <code>if: always()</code>. Cách sửa: thêm nó, và đặt bước dọn sau bước build.</li>
<li><strong>Nhãn <code>-latest</code> cho bản phát hành.</strong> Dấu hiệu: hai lần phát hành có Xcode hay JDK khác nhau. Nguyên nhân: <code>-latest</code> trỏ tới image ổn định mới nhất (tài liệu GitHub) nên đổi mà không báo. Cách sửa: nhãn cố định (<code>ubuntu-24.04</code>, <code>macos-26</code>), chọn Xcode bằng <code>xcode-select</code>.</li>
</ul>
</div>

## Chạy ví dụ và kiểm chứng

Từ thư mục gốc repository:

```bash
cd examples/toi-noi-flutter/54-ci-cd-build
dart pub get
dart format --output=none --set-exit-if-changed .
dart analyze --fatal-infos
dart test
```

Đã kiểm với Flutter 3.47.4 và Dart 3.13.3 trên macOS Apple silicon: `dart analyze --fatal-infos` báo `No issues found!`, `dart test` chạy 118 test xanh. Mức: **Kiểm chứng cấu hình**. Test không dùng mạng, không đụng keychain thật, cần `bash`, `base64` và `openssl`.

`sample_workflow_test.dart` (28 test) khẳng định tính chất của workflow mẫu. `audit_rules_test.dart` (52 test) sửa đúng một chỗ của file và đòi kiểm tra đỏ đúng mã. `step_scripts_test.dart` (22 test) chạy script thật với secret giả: giải mã đúng, không lộ log, secret rỗng làm bước đỏ, dọn sạch cả khi cài đặt hỏng, build number tăng ngặt. `models_test.dart` (16 test) kiểm build number và mô hình chi phí.

**Chưa làm**, nói thẳng:

- Workflow chưa chạy trên GitHub: chưa có run, environment, reviewer hay secret thật. Chưa chạy `actionlint` (máy chạy kiểm chứng không có).
- Chưa build AAB hay IPA, chưa dùng `security` thật, chưa biết Xcode trên runner đọc profile ở thư mục nào trong hai thư mục đã chép.
- Chưa upload lên store hay Firebase (bài 55 đến 57), chưa có job test trước khi build, chưa ghim action bằng SHA.
- Gradle đọc `key.properties` như `java.util.Properties`, nên mật khẩu chứa dấu `\` cần escape; script chưa làm.
- Số phút và hoá đơn là minh hoạ; chưa so tốc độ với Codemagic.

## Bài tập

1. Run số 57: job Android xanh, job iOS đỏ vì profile hết hạn. Ta thay profile trong secret rồi bấm **Re-run failed jobs**. Build number của iOS lần chạy lại là bao nhiêu, và nó có khớp AAB đã dựng không?
2. Ai đó thêm vào job Android một bước `run: cat android/key.properties` "để xem file có đúng không". Kiểm tra nào của lab đỏ, và vì sao đó vẫn là lỗi?
3. Job iOS tăng từ 30 lên 45 phút. Tính chi phí một lần phát hành (Android vẫn 15 phút) và số lần phát hành trong 12 USD included.

<details><summary>Xem lời giải</summary>
<p><strong>1.</strong> Vẫn là 57 và khớp AAB: <code>run_number</code> không đổi khi chạy lại, nên cả hai nền tảng vẫn cùng một số. Muốn một số mới cho cả hai thì tạo run mới thay vì re-run.</p>
<p><strong>2.</strong> Quy tắc <code>print-secret-file</code> của <code>checkSecretLeaks</code> đỏ: lệnh <code>cat</code> trên <code>key.properties</code>. Ta không dựa vào việc log tự che giá trị, và Flutter docs dặn đừng echo lại biến vào console. Quy tắc của lab đơn giản hơn: bí mật, hay file chứa bí mật, không bao giờ được in.</p>
<p><strong>3.</strong> 15 phút Linux là 90 và 45 phút macOS là 2.790, cộng lại 2.880 phần nghìn USD, tức 2,88 USD. 12.000 chia 2.880 được 4 lần (chia nguyên). Job iOS chậm thêm 15 phút làm ngân sách mất 2 lần phát hành.</p>
</details>

Bài trước: [fastlane cho Flutter: match, gym, pilot, supply](/articles/flutter-fastlane-match-gym-pilot-supply-tu-dong-hoa-build). Bài tiếp: [Firebase App Distribution: phân phối nội bộ cho tester iOS và Android](/articles/flutter-firebase-app-distribution-phan-phoi-noi-bo-tester).

## Tài liệu gốc

- [Workflow syntax for GitHub Actions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax): `on.push.tags`, `workflow_dispatch`, `defaults.run`, `environment` (docs.github.com).
- [Contexts reference](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts): `github.run_number`, `run_attempt`, `run_id` (docs.github.com).
- [Using secrets in GitHub Actions](https://docs.github.com/en/actions/how-tos/security-for-github-actions/security-guides/using-secrets-in-github-actions): secret dạng base64, truyền qua `env`, fork (docs.github.com).
- [Installing an Apple certificate on macOS runners for Xcode development](https://docs.github.com/en/actions/how-tos/deploy/deploy-to-third-party-platforms/sign-xcode-applications) (docs.github.com).
- [Managing environments for deployment](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments) và [OpenID Connect](https://docs.github.com/en/actions/concepts/security/openid-connect) (docs.github.com).
- [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners), [About billing for GitHub Actions](https://docs.github.com/en/billing/concepts/product-billing/github-actions) và [Download workflow artifacts](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/download-workflow-artifacts) (docs.github.com).
- [Continuous delivery with Flutter](https://docs.flutter.dev/deployment/cd) (docs.flutter.dev).
- [Version your app](https://developer.android.com/studio/publish/versioning): `versionCode` (developer.android.com).
- [Codemagic pricing](https://codemagic.io/pricing/) và [Publishing to App Store Connect](https://docs.codemagic.io/flutter-publishing/publishing-to-app-store/) (trang chính thức Codemagic).
