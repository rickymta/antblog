import 'package:test/test.dart';
import 'package:toi_noi_58/toi_noi_58.dart';

import 'support.dart';

const _precise = ManifestVocabulary.preciseLocation;
const _fn = ManifestVocabulary.appFunctionality;

/// Checklist chạy trên manifest [xml], mọi thứ khác lấy từ gói app khách.
/// Chỉ giữ phát hiện có mã bắt đầu bằng "manifest-".
Set<String> manifestCodes(String xml) {
  final base = loadSample(StoreApp.customer);
  final input = change(base, manifest: PrivacyManifest.parse(xml));
  return {
    for (final c in codes(ReleaseChecklist(input).run()))
      if (c.startsWith('manifest-')) c,
  };
}

void main() {
  group('PrivacyInfo.xcprivacy thật của hai app', () {
    for (final app in StoreApp.values) {
      test(
        '${app.name}: không tracking, không domain, không required reason API',
        () {
          final m = loadSample(app).manifest;
          expect(m.tracking, isFalse);
          expect(m.trackingDomains, isEmpty);
          expect(m.accessedApis, isEmpty);
          expect(m.collected, isNotEmpty);
          expect(m.collected.every((c) => !c.tracking), isTrue);
        },
      );
    }

    test('app khách thu thập đúng bốn loại; app tài xế có Name nhưng không có PurchaseHistory', () {
      expect(loadSample(StoreApp.customer).manifest.collectedTypes, {
        ManifestVocabulary.phoneNumber,
        ManifestVocabulary.preciseLocation,
        ManifestVocabulary.userId,
        ManifestVocabulary.purchaseHistory,
      });
      expect(loadSample(StoreApp.driver).manifest.collectedTypes, {
        ManifestVocabulary.name,
        ManifestVocabulary.phoneNumber,
        ManifestVocabulary.preciseLocation,
        ManifestVocabulary.userId,
      });
    });

    test('mọi loại dữ liệu và mục đích đều nằm trong danh mục của Apple', () {
      for (final app in StoreApp.values) {
        for (final c in loadSample(app).manifest.collected) {
          expect(ManifestVocabulary.dataTypes, contains(c.type));
          expect(ManifestVocabulary.purposes.containsAll(c.purposes), isTrue);
        }
      }
    });
  });

  group('required reason API', () {
    test('lý do hợp lệ của từng nhóm được chấp nhận', () {
      for (final e in RequiredReasonApi.reasonsByCategory.entries) {
        for (final reason in e.value.difference(
          RequiredReasonApi.sdkOnlyReasons,
        )) {
          final xml = manifestXml(
            apis: [
              (category: e.key, reasons: [reason]),
            ],
          );
          expect(manifestCodes(xml), isEmpty, reason: '${e.key} $reason');
        }
      }
    });

    test('lý do của nhóm khác bị từ chối: CA92.1 không dùng được cho FileTimestamp', () {
      final xml = manifestXml(
        apis: [
          (category: RequiredReasonApi.fileTimestamp, reasons: ['CA92.1']),
        ],
      );
      expect(manifestCodes(xml), {'manifest-api-reason-invalid'});
    });

    test('lý do không tồn tại bị từ chối', () {
      final xml = manifestXml(
        apis: [
          (category: RequiredReasonApi.userDefaults, reasons: ['ZZZZ.9']),
        ],
      );
      expect(manifestCodes(xml), {'manifest-api-reason-invalid'});
    });

    test('lý do chỉ dành cho SDK (0A2A.1, C56D.1) bị từ chối trong manifest của app', () {
      final xml = manifestXml(
        apis: [
          (category: RequiredReasonApi.fileTimestamp, reasons: ['0A2A.1']),
          (category: RequiredReasonApi.userDefaults, reasons: ['C56D.1']),
        ],
      );
      expect(manifestCodes(xml), {'manifest-api-sdk-only-reason'});
    });

    test('nhóm API lạ và nhóm không có lý do bị báo', () {
      expect(
        manifestCodes(
          manifestXml(
            apis: [
              (
                category: 'NSPrivacyAccessedAPICategoryFoo',
                reasons: ['CA92.1'],
              ),
            ],
          ),
        ),
        {'manifest-api-category-unknown'},
      );
      expect(
        manifestCodes(
          manifestXml(
            apis: [
              (category: RequiredReasonApi.diskSpace, reasons: <String>[]),
            ],
          ),
        ),
        {'manifest-api-reasons-empty'},
      );
    });

    test('một nhóm khai hai lần là cảnh báo, không phải lỗi', () {
      final input = change(
        loadSample(StoreApp.customer),
        manifest: PrivacyManifest.parse(
          manifestXml(
            collected: [
              for (final t in loadSample(StoreApp.customer).manifest.collected)
                (
                  type: t.type,
                  linked: t.linked,
                  tracking: t.tracking,
                  purposes: t.purposes.toList(),
                ),
            ],
            apis: [
              (category: RequiredReasonApi.userDefaults, reasons: ['CA92.1']),
              (category: RequiredReasonApi.userDefaults, reasons: ['CA92.1']),
            ],
          ),
        ),
      );
      final found = ReleaseChecklist(input)
          .run()
          .where((f) => f.code == 'manifest-api-category-duplicate');
      expect(found.single.severity, Severity.warning);
    });
  });

  group('tracking và dữ liệu thu thập', () {
    test('NSPrivacyTracking true mà không có domain là lỗi', () {
      expect(
        manifestCodes(manifestXml(tracking: true)),
        contains('manifest-tracking-domains'),
      );
      expect(
        manifestCodes(
          manifestXml(tracking: true, domains: ['ads.toinoi.example']),
        ),
        isNot(contains('manifest-tracking-domains')),
      );
    });

    test(
      'dữ liệu đánh dấu tracking khi manifest khai không tracking là lỗi',
      () {
        final xml = manifestXml(
          collected: [
            (type: _precise, linked: true, tracking: true, purposes: [_fn]),
          ],
        );
        expect(manifestCodes(xml), contains('manifest-tracking-mismatch'));
      },
    );

    test(
      'kiểu dữ liệu và mục đích ngoài danh mục, hoặc không có mục đích, bị báo',
      () {
        expect(
          manifestCodes(
            manifestXml(
              collected: [
                (
                  type: 'NSPrivacyCollectedDataTypeFoo',
                  linked: true,
                  tracking: false,
                  purposes: [_fn],
                ),
              ],
            ),
          ),
          contains('manifest-data-type-unknown'),
        );
        expect(
          manifestCodes(
            manifestXml(
              collected: [
                (
                  type: _precise,
                  linked: true,
                  tracking: false,
                  purposes: ['Quảng cáo'],
                ),
              ],
            ),
          ),
          contains('manifest-purpose-unknown'),
        );
        expect(
          manifestCodes(
            manifestXml(
              collected: [
                (
                  type: _precise,
                  linked: true,
                  tracking: false,
                  purposes: <String>[],
                ),
              ],
            ),
          ),
          contains('manifest-purposes-empty'),
        );
      },
    );
  });

  group('file hỏng', () {
    test('thiếu khoá cấp cao nhất ném FormatException', () {
      expect(
        () => PrivacyManifest.parse(
          '<plist><dict><key>NSPrivacyTracking</key><false/></dict></plist>',
        ),
        throwsFormatException,
      );
    });

    test('NSPrivacyTracking sai kiểu ném FormatException', () {
      final xml = manifestXml().replaceFirst(
        '<key>NSPrivacyTracking</key><false/>',
        '<key>NSPrivacyTracking</key><string>no</string>',
      );
      expect(() => PrivacyManifest.parse(xml), throwsFormatException);
    });
  });
}
