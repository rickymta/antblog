import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_35/location_access.dart';
import 'package:toi_noi_35/location_models.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

LocationSnapshot snap({
  bool service = true,
  SystemPermission permission = SystemPermission.denied,
  LocationPrecision precision = LocationPrecision.precise,
  bool askedFg = false,
  bool askedBg = false,
  bool blocked = false,
}) => LocationSnapshot(
  serviceEnabled: service,
  permission: permission,
  precision: precision,
  askedForeground: askedFg,
  askedBackground: askedBg,
  blockedByRequest: blocked,
);

LocationAccessState observe(LocationSnapshot s) =>
    reduce(const NotAsked(), Observed(s));

void main() {
  group('Observed: bảy trạng thái từ ảnh chụp hệ điều hành', () {
    test('denied mà chưa hỏi lần nào là NotAsked', () {
      expect(observe(snap()), isA<NotAsked>());
    });

    test('denied sau khi app đã hỏi là Denied (còn hỏi lại được)', () {
      expect(observe(snap(askedFg: true)), isA<Denied>());
    });

    test(
      'denied kèm cờ bị chặn (Android, sau lần xin thứ hai) là DeniedForever',
      () {
        expect(
          observe(snap(askedFg: true, blocked: true)),
          isA<DeniedForever>(),
        );
        expect(
          observe(snap(blocked: true)),
          isA<NotAsked>(),
          reason: 'cờ chỉ có nghĩa sau khi đã hỏi',
        );
      },
    );

    test('deniedForever là DeniedForever, dù app nhớ gì', () {
      expect(
        observe(snap(permission: SystemPermission.deniedForever)),
        isA<DeniedForever>(),
      );
      expect(
        observe(
          snap(permission: SystemPermission.deniedForever, askedFg: true),
        ),
        isA<DeniedForever>(),
      );
    });

    test('whileInUse mang theo độ chính xác và việc đã xin nền chưa', () {
      expect(
        observe(
          snap(
            permission: SystemPermission.whileInUse,
            precision: LocationPrecision.reduced,
            askedBg: true,
          ),
        ),
        const WhileInUse(LocationPrecision.reduced, backgroundAsked: true),
      );
    });

    test('always mang theo độ chính xác', () {
      expect(
        observe(snap(permission: SystemPermission.always)),
        const Always(LocationPrecision.precise),
      );
    });

    test('dịch vụ vị trí tắt thắng mọi quyền khác', () {
      for (final permission in SystemPermission.values) {
        expect(
          observe(snap(service: false, permission: permission, askedFg: true)),
          isA<ServiceOff>(),
          reason: permission.name,
        );
      }
    });

    test('Observed bỏ qua trạng thái cũ, kể cả Asking', () {
      final s = snap(permission: SystemPermission.whileInUse);
      expect(
        reduce(const Asking(AskTarget.foreground), Observed(s)),
        isA<WhileInUse>(),
      );
    });
  });

  group('RequestStarted: chỉ mở hộp thoại ở trạng thái cho phép', () {
    const all = <LocationAccessState>[
      NotAsked(),
      Asking(AskTarget.foreground),
      WhileInUse(LocationPrecision.precise),
      Always(LocationPrecision.precise),
      Denied(),
      DeniedForever(),
      ServiceOff(),
    ];

    test('xin foreground: chỉ từ NotAsked và Denied', () {
      final opens = [
        for (final s in all)
          if (s is! Asking &&
              reduce(s, const RequestStarted(AskTarget.foreground)) is Asking)
            s.runtimeType,
      ];
      expect(opens, [NotAsked, Denied]);
    });

    test('xin nền: chỉ từ WhileInUse', () {
      final opens = [
        for (final s in all)
          if (s is! Asking &&
              reduce(s, const RequestStarted(AskTarget.background)) is Asking)
            s.runtimeType,
      ];
      expect(opens, [WhileInUse]);
    });

    test('trạng thái không cho phép giữ nguyên, không ném lỗi', () {
      expect(
        reduce(
          const DeniedForever(),
          const RequestStarted(AskTarget.foreground),
        ),
        isA<DeniedForever>(),
      );
      expect(
        reduce(const ServiceOff(), const RequestStarted(AskTarget.foreground)),
        isA<ServiceOff>(),
      );
    });
  });

  group('adviceFor và canTrack', () {
    test('mỗi trạng thái có một gợi ý cho giao diện', () {
      expect(
        adviceFor(const NotAsked(), AppRole.customer),
        LocationAdvice.explainThenRequest,
      );
      expect(
        adviceFor(const Denied(), AppRole.customer),
        LocationAdvice.explainThenRequest,
      );
      expect(
        adviceFor(const Asking(AskTarget.foreground), AppRole.customer),
        LocationAdvice.wait,
      );
      expect(
        adviceFor(const DeniedForever(), AppRole.customer),
        LocationAdvice.openAppSettings,
      );
      expect(
        adviceFor(const ServiceOff(), AppRole.customer),
        LocationAdvice.openLocationSettings,
      );
      expect(
        adviceFor(const Always(LocationPrecision.precise), AppRole.driver),
        LocationAdvice.ready,
      );
    });

    test(
      'chỉ tài xế có chuyến, chưa từng xin nền, mới được mời nâng quyền',
      () {
        const state = WhileInUse(LocationPrecision.precise);
        expect(
          adviceFor(state, AppRole.driver, tripActive: true),
          LocationAdvice.offerBackground,
        );
        expect(
          adviceFor(state, AppRole.driver),
          LocationAdvice.ready,
          reason: 'chưa có chuyến thì chưa xin nền',
        );
        expect(
          adviceFor(state, AppRole.customer, tripActive: true),
          LocationAdvice.ready,
          reason: 'app khách không bao giờ xin nền',
        );
        expect(
          adviceFor(
            const WhileInUse(LocationPrecision.precise, backgroundAsked: true),
            AppRole.driver,
            tripActive: true,
          ),
          LocationAdvice.ready,
          reason: 'đã xin một lần, không làm phiền lại',
        );
      },
    );

    test('nền tảng không có bước xin nền thì không mời', () {
      expect(
        adviceFor(
          const WhileInUse(LocationPrecision.precise),
          AppRole.driver,
          tripActive: true,
          backgroundAvailable: false,
        ),
        LocationAdvice.ready,
      );
    });

    test('canTrack chỉ đúng khi có quyền', () {
      expect(const WhileInUse(LocationPrecision.reduced).canTrack, isTrue);
      expect(const Always(LocationPrecision.precise).canTrack, isTrue);
      for (final s in const <LocationAccessState>[
        NotAsked(),
        Asking(AskTarget.background),
        Denied(),
        DeniedForever(),
        ServiceOff(),
      ]) {
        expect(s.canTrack, isFalse, reason: s.runtimeType.toString());
      }
    });
  });

  group('profileFor: vị trí nền gắn với chuyến hợp lệ', () {
    test('app khách không bao giờ có profile nền', () {
      for (final status in [...TripStatus.values, null]) {
        final p = profileFor(AppRole.customer, status);
        expect(p.background, isFalse, reason: '$status');
        expect(p, TrackingProfile.customerPickup);
      }
    });

    test('tài xế chỉ chạy nền từ accepted tới inTransit', () {
      final onTrip = {
        for (final s in TripStatus.values)
          if (profileFor(AppRole.driver, s).background) s,
      };
      expect(onTrip, {
        TripStatus.accepted,
        TripStatus.arriving,
        TripStatus.pickedUp,
        TripStatus.inTransit,
      });
      expect(
        profileFor(AppRole.driver, null),
        TrackingProfile.driverOnline,
        reason: 'online chưa có chuyến',
      );
    });

    test('profile nền dày hơn profile thường', () {
      expect(
        TrackingProfile.driverOnTrip.distanceFilterMeters,
        lessThan(TrackingProfile.driverOnline.distanceFilterMeters),
      );
    });
  });
}
