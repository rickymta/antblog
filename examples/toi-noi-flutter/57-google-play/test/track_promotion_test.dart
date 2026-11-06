import 'package:test/test.dart';
import 'package:toi_noi_57/manifest_permissions.dart';
import 'package:toi_noi_57/track_promotion.dart';

import 'support.dart';

/// Đưa một bản tới [target] bằng đường hợp lệ ngắn nhất qua closed.
TrackPromotion reach(Track target, {PreflightInput? input}) {
  final i = input ?? readyInput();
  var p = TrackPromotion.uploaded(versionCode: 7);
  for (final step in switch (target) {
    Track.internal => const <Track>[],
    Track.closed => const [Track.closed],
    Track.open => const [Track.closed, Track.open],
    Track.production => const [Track.closed, Track.production],
  }) {
    p = p.promote(step, i);
  }
  return p;
}

void main() {
  group('bảng đường đi', () {
    test('trong 16 cặp (từ, tới) chỉ 4 cặp hợp lệ', () {
      final ok = <(Track, Track)>[];
      for (final from in Track.values) {
        for (final to in Track.values) {
          try {
            reach(from).promote(to, readyInput());
            ok.add((from, to));
          } on IllegalMove {
            // không hợp lệ: đúng như bảng
          }
        }
      }
      expect(ok, [
        (Track.internal, Track.closed),
        (Track.closed, Track.open),
        (Track.closed, Track.production),
        (Track.open, Track.production),
      ]);
    });

    test(
      'không đi lùi, không đứng yên, không từ internal thẳng production',
      () {
        final p = TrackPromotion.uploaded(versionCode: 7);
        expect(
          () => p.promote(Track.production, readyInput()),
          throwsA(isA<IllegalMove>()),
        );
        expect(
          () => p.promote(Track.internal, readyInput()),
          throwsA(isA<IllegalMove>()),
        );
        expect(
          () => reach(Track.closed).promote(Track.internal, readyInput()),
          throwsA(isA<IllegalMove>()),
        );
      },
    );

    test('lịch sử ghi đúng đường đi và không sửa được từ bên ngoài', () {
      final p = reach(Track.open);
      expect(p.history, [Track.internal, Track.closed, Track.open]);
      expect(p.versionCode, 7);
      expect(() => p.history.add(Track.production), throwsUnsupportedError);
    });

    test('promote trả về bản mới, bản cũ giữ nguyên', () {
      final start = TrackPromotion.uploaded(versionCode: 7);
      start.promote(Track.closed, readyInput());
      expect(start.track, Track.internal);
      expect(start.history, [Track.internal]);
    });

    test('đường đi hợp lệ nhưng còn điều chặn thì không đổi track', () {
      final closed = reach(Track.closed);
      expect(
        () => closed.promote(Track.open, readyInput(targetSdk: 30)),
        throwsA(isA<PromotionBlocked>()),
      );
      expect(closed.track, Track.closed);
    });
  });

  group('yêu cầu closed test của tài khoản cá nhân', () {
    List<Blocker> blockersFor(ClosedTestProgress? progress) {
      try {
        reach(Track.closed)
            .promote(Track.production, readyInput(closedTest: progress));
        return const [];
      } on PromotionBlocked catch (e) {
        return e.blockers;
      }
    }

    test('11 tester hoặc 13 ngày vẫn bị chặn, 12 tester 14 ngày thì qua', () {
      const notYet = [Blocker.productionAccessNotEarned];
      expect(
        blockersFor(
          const ClosedTestProgress(testersOptedIn: 11, consecutiveDays: 14),
        ),
        notYet,
      );
      expect(
        blockersFor(
          const ClosedTestProgress(testersOptedIn: 12, consecutiveDays: 13),
        ),
        notYet,
      );
      expect(
        blockersFor(
          const ClosedTestProgress(testersOptedIn: 12, consecutiveDays: 14),
        ),
        isEmpty,
      );
    });

    test('chưa chạy closed test lần nào thì bị chặn', () {
      expect(blockersFor(null), [Blocker.productionAccessNotEarned]);
    });

    test('yêu cầu chỉ chặn đích production', () {
      expect(
        () =>
            reach(Track.closed)
                .promote(Track.open, readyInput(closedTest: null)),
        returnsNormally,
      );
    });

    test('mốc tạo tài khoản: 13/11/2023 chưa bị áp, 14/11/2023 bị áp', () {
      DeveloperAccount at(DateTime t) =>
          DeveloperAccount(kind: AccountKind.personal, createdAt: t);
      expect(
        at(DateTime.utc(2023, 11, 13, 23, 59)).mustPassClosedTest,
        isFalse,
      );
      expect(at(DateTime.utc(2023, 11, 14)).mustPassClosedTest, isTrue);
    });

    test('tài khoản cá nhân cũ và tài khoản tổ chức không bị lab chặn', () {
      final org = DeveloperAccount(
        kind: AccountKind.organization,
        createdAt: DateTime.utc(2025, 1, 1),
      );
      for (final account in [oldPersonal, org]) {
        expect(
          preflight(
            Track.production,
            readyInput(account: account, closedTest: null),
          ),
          isEmpty,
        );
      }
    });
  });

  group('các điều chặn khác', () {
    test('Data safety chưa xong thì không ra khỏi internal', () {
      expect(
        () =>
            TrackPromotion.uploaded(versionCode: 1)
                .promote(Track.closed, readyInput(dataSafetyComplete: false)),
        throwsA(
          isA<PromotionBlocked>().having((e) => e.blockers, 'blockers', [
            Blocker.dataSafetyIncomplete,
          ]),
        ),
      );
    });

    test('target API 35 bị chặn, 36 qua', () {
      expect(preflight(Track.closed, readyInput(targetSdk: 35)), [
        Blocker.targetSdkTooLow,
      ]);
      expect(preflight(Track.closed, readyInput(targetSdk: 36)), isEmpty);
    });

    test(
      'manifest có vị trí nền mà chưa khai thì bị chặn; khai rồi thì qua',
      () {
        final perms = {
          ...readPermissions(readSampleManifest()),
          backgroundLocation,
        };
        expect(preflight(Track.closed, readyInput(permissions: perms)), [
          Blocker.backgroundLocationNotDeclared,
        ]);
        expect(
          preflight(
            Track.closed,
            readyInput(permissions: perms, backgroundLocationDeclared: true),
          ),
          isEmpty,
        );
      },
    );

    test('manifest mẫu không có vị trí nền nên không đòi khai báo đó', () {
      expect(
        preflight(Track.closed, readyInput(backgroundLocationDeclared: false)),
        isEmpty,
      );
    });

    test('foreground service vị trí chưa khai ở Console thì bị chặn', () {
      expect(
        preflight(Track.closed, readyInput(foregroundServiceDeclared: false)),
        [Blocker.foregroundServiceNotDeclared],
      );
    });

    test('nhiều điều chặn được báo cùng lúc, theo thứ tự cố định', () {
      final perms = {
        ...readPermissions(readSampleManifest()),
        backgroundLocation,
      };
      expect(
        preflight(
          Track.production,
          readyInput(
            dataSafetyComplete: false,
            targetSdk: 34,
            permissions: perms,
            foregroundServiceDeclared: false,
            closedTest: null,
          ),
        ),
        [
          Blocker.dataSafetyIncomplete,
          Blocker.targetSdkTooLow,
          Blocker.backgroundLocationNotDeclared,
          Blocker.foregroundServiceNotDeclared,
          Blocker.productionAccessNotEarned,
        ],
      );
    });
  });
}
