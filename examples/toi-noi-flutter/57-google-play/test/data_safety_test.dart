import 'package:test/test.dart';
import 'package:toi_noi_57/data_safety.dart';
import 'package:toi_noi_57/manifest_permissions.dart';

import 'support.dart';

String manifestWith(String body) =>
    '<manifest xmlns:android="http://schemas.android.com/apk/res/android" '
    'xmlns:tools="http://schemas.android.com/tools">$body</manifest>';

Set<FindingCode> codes(List<Finding> findings, Severity severity) => {
  for (final f in findings)
    if (f.severity == severity) f.code,
};

void main() {
  group('readPermissions', () {
    test('manifest mẫu khai đúng sáu quyền và không có vị trí nền', () {
      expect(readPermissions(readSampleManifest()), {
        'android.permission.INTERNET',
        coarseLocation,
        fineLocation,
        'android.permission.FOREGROUND_SERVICE',
        foregroundServiceLocation,
        'android.permission.POST_NOTIFICATIONS',
      });
      expect(
        readPermissions(readSampleManifest()),
        isNot(contains(backgroundLocation)),
      );
    });

    test('bỏ qua chú thích, kể cả chú thích nhiều dòng chứa thẻ quyền', () {
      final xml = manifestWith('''
<!-- <uses-permission android:name="$backgroundLocation" /> -->
<!--
  <uses-permission android:name="$fineLocation" />
-->
<uses-permission android:name="$coarseLocation" />''');
      expect(readPermissions(xml), {coarseLocation});
    });

    test('quyền có tools:node="remove" không được tính', () {
      final xml = manifestWith('''
<uses-permission android:name="$fineLocation" />
<uses-permission android:name="$backgroundLocation" tools:node="remove" />''');
      expect(readPermissions(xml), {fineLocation});
    });

    test(
      'không nhầm uses-permission-sdk-23, chịu được xuống dòng và nháy đơn',
      () {
        final xml = manifestWith('''
<uses-permission-sdk-23 android:name="$fineLocation" />
<uses-permission
    android:maxSdkVersion="32"
    android:name='$coarseLocation'
/>''');
        expect(readPermissions(xml), {coarseLocation});
      },
    );

    test('manifest rỗng thì không có quyền', () {
      expect(readPermissions(manifestWith('')), isEmpty);
    });
  });

  group('consoleDeclarationsFor', () {
    test('manifest mẫu chỉ kéo theo khai báo foreground service', () {
      expect(consoleDeclarationsFor(readPermissions(readSampleManifest())), {
        ConsoleDeclaration.foregroundServiceLocation,
      });
    });

    test('vị trí nền kéo theo khai báo vị trí nền', () {
      expect(consoleDeclarationsFor({backgroundLocation}), {
        ConsoleDeclaration.backgroundLocation,
      });
    });
  });

  group('requiredTypesFor', () {
    test('chỉ COARSE thì cần approximate', () {
      expect(requiredTypesFor({coarseLocation}), {
        DataType.approximateLocation,
      });
    });

    test('FINE kéo theo cả precise và approximate', () {
      expect(requiredTypesFor({fineLocation}), {
        DataType.preciseLocation,
        DataType.approximateLocation,
      });
    });

    test('INTERNET một mình không kéo theo loại dữ liệu nào', () {
      expect(requiredTypesFor({'android.permission.INTERNET'}), isEmpty);
    });
  });

  group('DataSafetyDraft.check', () {
    final xml = readSampleManifest();

    test('bản nháp tài xế khớp manifest mẫu: không lỗi, không cảnh báo', () {
      expect(driverDraft.check(xml), isEmpty);
      expect(driverDraft.isCompleteFor(xml), isTrue);
    });

    DataSafetyDraft draft({
      List<DataDeclaration>? declarations,
      bool encrypted = true,
      bool deletion = true,
      String? url = 'https://toinoi.example/privacy',
    }) => DataSafetyDraft(
      declarations: declarations ?? driverDraft.declarations,
      encryptedInTransit: encrypted,
      deletionRequestSupported: deletion,
      privacyPolicyUrl: url,
    );

    test('thiếu precise hoặc approximate khi manifest có quyền thì là lỗi', () {
      for (final keep in [
        DataType.preciseLocation,
        DataType.approximateLocation,
      ]) {
        final only = [
          for (final d in driverDraft.declarations)
            if (d.type == keep) d,
        ];
        final findings = draft(declarations: only).check(xml);
        expect(codes(findings, Severity.error), {
          FindingCode.missingDeclaration,
        }, reason: 'chỉ khai $keep');
      }
      expect(codes(draft(declarations: const []).check(xml), Severity.error), {
        FindingCode.missingDeclaration,
      });
    });

    test('thu thập mà không nêu mục đích là lỗi', () {
      final findings = draft(
        declarations: const [
          DataDeclaration(type: DataType.preciseLocation, purposes: {}),
          DataDeclaration(
            type: DataType.approximateLocation,
            purposes: {DataPurpose.appFunctionality},
          ),
        ],
      ).check(xml);
      expect(codes(findings, Severity.error), {FindingCode.noPurpose});
    });

    test('thiếu hoặc sai liên kết chính sách là lỗi', () {
      expect(codes(draft(url: null).check(xml), Severity.error), {
        FindingCode.noPrivacyPolicy,
      });
      expect(codes(draft(url: '   ').check(xml), Severity.error), {
        FindingCode.noPrivacyPolicy,
      });
      expect(
        codes(
          draft(url: 'http://toinoi.example/privacy').check(xml),
          Severity.error,
        ),
        {FindingCode.privacyPolicyNotHttps},
      );
    });

    test('khai vị trí mà manifest không có quyền chỉ là cảnh báo', () {
      final noLocation = manifestWith(
        '<uses-permission android:name="android.permission.INTERNET" />',
      );
      final findings = driverDraft.check(noLocation);
      expect(codes(findings, Severity.error), isEmpty);
      expect(codes(findings, Severity.warning), {
        FindingCode.declaredWithoutPermission,
      });
      expect(driverDraft.isCompleteFor(noLocation), isTrue);
    });

    test('không mã hoá khi truyền hoặc không có đường xoá là cảnh báo', () {
      final findings = draft(encrypted: false, deletion: false).check(xml);
      expect(codes(findings, Severity.error), isEmpty);
      expect(codes(findings, Severity.warning), {
        FindingCode.notEncryptedInTransit,
        FindingCode.noDeletionPath,
      });
    });

    test(
      'đổi manifest thành có quyền FINE thì bản nháp thiếu precise bị bắt',
      () {
        final fineOnly = manifestWith(
          '<uses-permission android:name="$fineLocation" />',
        );
        final approxOnly = const DataSafetyDraft(
          declarations: [
            DataDeclaration(
              type: DataType.approximateLocation,
              purposes: {DataPurpose.appFunctionality},
            ),
          ],
          encryptedInTransit: true,
          deletionRequestSupported: true,
          privacyPolicyUrl: 'https://toinoi.example/privacy',
        );
        expect(approxOnly.isCompleteFor(fineOnly), isFalse);
      },
    );
  });
}
