import 'dart:io';

import 'package:test/test.dart';
import 'package:toi_noi_42/src/integrity_policy.dart';
import 'package:toi_noi_42/src/masvs_checklist.dart';
import 'package:toi_noi_42/src/storage_policy.dart';
import 'package:toi_noi_42/src/threat_model.dart';

void main() {
  group('StoragePolicy', () {
    test('mọi loại dữ liệu đều có quy tắc', () {
      expect(StoragePolicy.rules.keys, unorderedEquals(DataKind.values));
    });

    test('token chỉ được ở Keychain/Keystore hoặc RAM', () {
      for (final kind in [DataKind.accessToken, DataKind.refreshToken]) {
        expect(StoragePolicy.isAllowed(kind, Store.secureStorage), isTrue);
        expect(StoragePolicy.isAllowed(kind, Store.memoryOnly), isTrue);
        expect(StoragePolicy.isAllowed(kind, Store.appDatabase), isFalse);
        expect(StoragePolicy.isAllowed(kind, Store.preferences), isFalse);
      }
    });

    test('OTP chỉ ở RAM', () {
      expect(
        StoragePolicy.isAllowed(DataKind.otpCode, Store.memoryOnly),
        isTrue,
      );
      expect(
        StoragePolicy.isAllowed(DataKind.otpCode, Store.secureStorage),
        isFalse,
      );
    });

    test('khoá bên thứ ba và số thẻ không được giữ ở bất cứ đâu', () {
      for (final kind in [DataKind.thirdPartyApiKey, DataKind.cardNumber]) {
        for (final store in Store.values) {
          expect(
            StoragePolicy.isAllowed(kind, store),
            isFalse,
            reason: '$kind/$store',
          );
        }
      }
    });

    test('cấu hình không nhạy cảm được ở nơi nào cũng được', () {
      for (final store in Store.values) {
        expect(StoragePolicy.isAllowed(DataKind.themeMode, store), isTrue);
      }
    });

    test(
      'không loại dữ liệu nào đã có lý do là được vào preferences nếu nhạy cảm',
      () {
        final inPreferences = [
          for (final kind in DataKind.values)
            if (StoragePolicy.isAllowed(kind, Store.preferences)) kind,
        ];
        expect(inPreferences, [DataKind.themeMode, DataKind.apiBaseUrl]);
      },
    );
  });

  group('decide (tín hiệu root/jailbreak)', () {
    // Mọi tập con của ba tín hiệu.
    final subsets = [
      for (var mask = 0; mask < 8; mask++)
        {
          for (var i = 0; i < 3; i++)
            if (mask & (1 << i) != 0) IntegritySignal.values[i],
        },
    ];

    test('không có tín hiệu: việc thường cứ chạy', () {
      expect(decide(AppAction.createTrip, {}), Decision.allow);
      expect(decide(AppAction.viewTripHistory, {}), Decision.allow);
    });

    test('có tín hiệu: việc thường không bị chặn, chỉ báo lên server', () {
      for (final signals in subsets.where((s) => s.isNotEmpty)) {
        expect(decide(AppAction.createTrip, signals), Decision.allowAndReport);
      }
    });

    test('việc nhạy cảm đòi xác thực lại cả khi máy "sạch"', () {
      for (final signals in subsets) {
        expect(
          decide(AppAction.changePayoutAccount, signals),
          Decision.requireReauthentication,
        );
      }
    });

    test('thêm tín hiệu không bao giờ làm quyết định dễ dãi hơn', () {
      for (final action in AppAction.values) {
        for (final fewer in subsets) {
          for (final more in subsets.where((s) => s.containsAll(fewer))) {
            expect(
              decide(action, more).index,
              greaterThanOrEqualTo(decide(action, fewer).index),
              reason: '$action $fewer -> $more',
            );
          }
        }
      }
    });

    test('Decision không có mức chặn hẳn', () {
      expect(Decision.values.map((d) => d.name), isNot(contains('deny')));
    });
  });

  group('threatModel', () {
    test(
      'mọi biện pháp tham chiếu một control MASVS có thật trong checklist',
      () {
        for (final threat in threatModel) {
          expect(referencesKnownControl(threat), isTrue, reason: threat.masvs);
        }
      },
    );

    test('cả hai app đều có mối đe doạ riêng và chung', () {
      for (final app in AppKind.values) {
        expect(threatModel.where((t) => t.apps.contains(app)), isNotEmpty);
      }
      expect(
        threatModel.where(
          (t) => t.apps.length == 1 && t.apps.single == AppKind.driver,
        ),
        isNotEmpty,
      );
    });

    test('mọi loại kẻ tấn công đều được xét ít nhất một lần', () {
      expect(
        threatModel.map((t) => t.attacker).toSet(),
        unorderedEquals(Attacker.values),
      );
    });

    test('mối đe doạ từ người sửa app luôn có biện pháp thực thi ở server', () {
      final tamper = threatModel.where((t) => t.attacker == Attacker.tamperer);
      expect(tamper, isNotEmpty);
      expect(tamper.every((t) => t.enforcedOn == EnforcedOn.server), isTrue);
    });
  });

  group('masvsChecklist', () {
    test('đủ 24 control của MASVS v2, ID không trùng', () {
      expect(masvsChecklist, hasLength(24));
      final ids = masvsChecklist.map((i) => i.id).toList();
      expect(ids.toSet(), hasLength(24));
      expect(ids.first, 'MASVS-STORAGE-1');
      expect(ids.last, 'MASVS-PRIVACY-4');
    });

    test('số control mỗi nhóm khớp trang MASVS: 2,2,3,2,3,4,4,4', () {
      final perGroup = [
        for (final g in MasvsGroup.values)
          masvsChecklist.where((i) => i.group == g).length,
      ];
      expect(perGroup, [2, 2, 3, 2, 3, 4, 4, 4]);
    });

    test('đánh số trong từng nhóm liên tục từ 1', () {
      for (final g in MasvsGroup.values) {
        final numbers = masvsChecklist
            .where((i) => i.group == g)
            .map((i) => i.number)
            .toList();
        expect(numbers, [for (var n = 1; n <= numbers.length; n++) n]);
      }
    });

    test('mục đã làm phải nêu ít nhất một bài', () {
      for (final item in masvsChecklist.where(
        (i) => i.coverage == Coverage.done,
      )) {
        expect(item.lessons, isNotEmpty, reason: item.id);
      }
    });

    test('mọi mục đều ghi chú; mục chưa đủ nói rõ thiếu gì', () {
      for (final item in masvsChecklist) {
        expect(item.note.trim(), isNotEmpty, reason: item.id);
      }
    });

    test('tổng kết: 3 đã làm, 12 một phần, 9 chưa làm', () {
      expect(coverageCounts(), {
        Coverage.done: 3,
        Coverage.partial: 12,
        Coverage.notDone: 9,
      });
    });
  });

  group('native-config Android', () {
    final manifest = File(
      'native-config/android/app/src/main/AndroidManifest.application.fragment.xml',
    ).readAsStringSync();
    final rules = File(
      'native-config/android/app/src/main/res/xml/data_extraction_rules.xml',
    ).readAsStringSync();

    test('tắt backup và trỏ tới luật trích xuất dữ liệu', () {
      expect(manifest, contains('android:allowBackup="false"'));
      expect(
        manifest,
        contains('android:dataExtractionRules="@xml/data_extraction_rules"'),
      );
    });

    test('luật loại cả tệp dữ liệu và tệp khoá bọc khỏi cloud backup và chuyển máy', () {
      for (final section in ['cloud-backup', 'device-transfer']) {
        final body = RegExp(
          '<$section>(.*?)</$section>',
          dotAll: true,
        ).firstMatch(rules)?.group(1);
        expect(body, isNotNull, reason: section);
        for (final file in [
          'FlutterSecureStorage',
          'FlutterSecureKeyStorage',
        ]) {
          expect(
            body,
            contains('<exclude domain="sharedpref" path="$file.xml"'),
            reason: '$section/$file',
          );
        }
      }
    });
  });
}
