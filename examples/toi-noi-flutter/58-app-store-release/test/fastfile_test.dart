import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

const _path = 'native-config/ios/fastlane/Fastfile.store.example';

String _laneBody(String source, String lane) {
  final m = RegExp(
    'lane :$lane do \\|options\\|\\n(.*?)\\n  end\\n',
    dotAll: true,
  ).firstMatch(source);
  if (m == null) throw StateError('Không thấy lane $lane');
  return m.group(1)!;
}

void main() {
  final source = File(_path).readAsStringSync();

  test('cú pháp Ruby hợp lệ (ruby -c); bỏ qua nếu máy không có ruby', () {
    final ProcessResult result;
    try {
      result = Process.runSync('ruby', ['-c', _path]);
    } on ProcessException {
      markTestSkipped('Máy không có ruby');
      return;
    }
    expect(result.exitCode, 0, reason: '${result.stderr}');
    expect(result.stdout, contains('Syntax OK'));
  });

  test('bảng Bundle ID khớp StoreApp (curriculum §3.2, bản prod)', () {
    for (final app in StoreApp.values) {
      expect(source, contains('"${app.name}" => "${app.bundleId}"'));
    }
  });

  group('lane listing', () {
    final body = _laneBody(source, 'listing');

    test('chỉ đẩy metadata và ảnh: không tải IPA, không gửi duyệt', () {
      expect(body, contains('skip_binary_upload: true'));
      expect(body, contains('submit_for_review: false'));
      expect(body, isNot(contains('submit_for_review: true')));
    });

    test('đọc metadata và ảnh từ store/<app>/ như bố cục của lab', () {
      expect(body, contains('metadata_path: "../store/#{app}/metadata"'));
      expect(body, contains('screenshots_path: "../store/#{app}/screenshots"'));
      for (final app in StoreApp.values) {
        expect(Directory('store/${app.name}/metadata/vi').existsSync(), isTrue);
      }
    });
  });

  group('lane submit', () {
    final body = _laneBody(source, 'submit');

    test('đòi confirm:yes trước khi gọi upload_to_app_store', () {
      expect(body.indexOf('confirm'), greaterThanOrEqualTo(0));
      expect(
        body.indexOf('confirm'),
        lessThan(body.indexOf('upload_to_app_store')),
      );
    });

    test('chỉ chọn build có sẵn: không tải IPA, không đẩy lại ảnh', () {
      expect(body, contains('skip_binary_upload: true'));
      expect(body, contains('skip_screenshots: true'));
      expect(body, contains('build_number: build_number'));
      expect(body, contains('submit_for_review: true'));
    });

    test('không skip_metadata: deliver sẽ bỏ qua automatic_release và phased_release', () {
      expect(body, isNot(contains('skip_metadata: true')));
      expect(body, contains('metadata_path: "../store/#{app}/metadata"'));
    });

    test('phased release và tự phát hành chỉ bật khi không phải lần đầu', () {
      expect(body, contains('automatic_release: !first_release'));
      expect(body, contains('phased_release: !first_release'));
    });
  });

  test('cả hai lane tắt precheck mục In-App Purchase (api_key không kiểm được mục này)', () {
    for (final lane in ['listing', 'submit']) {
      expect(
        _laneBody(source, lane),
        contains('precheck_include_in_app_purchases: false'),
        reason: lane,
      );
    }
  });

  test('luật chung: lane có automatic_release hoặc phased_release không được skip_metadata', () {
    for (final lane in ['listing', 'submit']) {
      final body = _laneBody(source, lane);
      final needsMetadata =
          body.contains('automatic_release') || body.contains('phased_release');
      if (needsMetadata) {
        expect(body, isNot(contains('skip_metadata: true')), reason: lane);
      }
    }
  });

  test('bí mật chỉ đọc từ ENV, không có giá trị gõ thẳng', () {
    for (final key in ['key_id', 'issuer_id', 'key_content']) {
      expect(source, contains('$key: env!('));
    }
    expect(source, isNot(contains('BEGIN PRIVATE KEY')));
  });
}
