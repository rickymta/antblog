import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_18/trip_draft.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// State không định nghĩa `==`: hai đối tượng cùng giá trị vẫn khác nhau.
final class _NoEquals {
  const _NoEquals(this.km);
  final double km;
}

class _NoEqualsNotifier extends Notifier<_NoEquals> {
  @override
  _NoEquals build() => const _NoEquals(5);

  void assign(double km) => state = _NoEquals(km);
}

class _ListNotifier extends Notifier<List<int>> {
  @override
  List<int> build() => [];

  void addInPlace(int value) {
    state.add(value);
    state = state;
  }
}

void main() {
  group('TripDraftNotifier (ProviderContainer, không cần widget)', () {
    test('build trả về bản nháp khởi đầu: xe máy 5 km', () {
      final container = ProviderContainer.test();

      expect(
        container.read(tripDraftProvider),
        const TripDraft(serviceKind: ServiceKind.ride, distanceKm: 5),
      );
    });

    test('selectService và setDistanceKm đổi từng trường', () {
      final container = ProviderContainer.test();
      final draft = container.read(tripDraftProvider.notifier);

      draft.selectService(ServiceKind.food);
      draft.setDistanceKm(2.001);

      expect(
        container.read(tripDraftProvider),
        const TripDraft(serviceKind: ServiceKind.food, distanceKm: 2.001),
      );
    });

    test('đúng 0 và 100 km được nhận; ngoài miền ném ArgumentError', () {
      final container = ProviderContainer.test();
      final draft = container.read(tripDraftProvider.notifier);

      draft.setDistanceKm(0);
      draft.setDistanceKm(100);
      expect(container.read(tripDraftProvider).distanceKm, 100);

      for (final km in [-0.001, 100.001, double.nan, double.infinity]) {
        expect(
          () => draft.setDistanceKm(km),
          throwsArgumentError,
          reason: '$km',
        );
      }
      // Giá trị sai không làm hỏng state.
      expect(container.read(tripDraftProvider).distanceKm, 100);
    });

    test('notifier là một đối tượng ổn định giữa các lần read', () {
      final container = ProviderContainer.test();

      expect(
        identical(
          container.read(tripDraftProvider.notifier),
          container.read(tripDraftProvider.notifier),
        ),
        isTrue,
      );
    });

    test('đặt lại đúng giá trị cũ không báo listener (so bằng ==)', () {
      final container = ProviderContainer.test();
      final seen = <TripDraft>[];
      container.listen(tripDraftProvider, (_, next) => seen.add(next));
      final draft = container.read(tripDraftProvider.notifier);

      draft.setDistanceKm(5); // đã là 5
      draft.selectService(ServiceKind.ride); // đã là ride
      expect(seen, isEmpty);

      draft.setDistanceKm(6);
      expect(seen, [
        const TripDraft(serviceKind: ServiceKind.ride, distanceKm: 6),
      ]);
    });

    test('thiếu ==: gán đúng giá trị cũ vẫn báo listener (build thừa)', () {
      final provider = NotifierProvider<_NoEqualsNotifier, _NoEquals>(
        _NoEqualsNotifier.new,
      );
      final container = ProviderContainer.test();
      var notifications = 0;
      container.listen(provider, (_, _) => notifications++);
      final notifier = container.read(provider.notifier);

      notifier.assign(5);
      notifier.assign(5);

      expect(notifications, 2);
    });

    test('sửa state tại chỗ rồi gán lại cùng đối tượng: không báo ai', () {
      final provider = NotifierProvider<_ListNotifier, List<int>>(
        _ListNotifier.new,
      );
      final container = ProviderContainer.test();
      var notifications = 0;
      container.listen(provider, (_, _) => notifications++);

      container.read(provider.notifier).addInPlace(1);

      expect(container.read(provider), [1]); // dữ liệu đã đổi
      expect(notifications, 0); // nhưng không ai được báo
    });

    test('listener nhận cả giá trị trước và sau', () {
      final container = ProviderContainer.test();
      final pairs = <(TripDraft?, TripDraft)>[];
      container.listen(
        tripDraftProvider,
        (prev, next) => pairs.add((prev, next)),
      );

      container.read(tripDraftProvider.notifier).setDistanceKm(8);

      expect(pairs.single.$1?.distanceKm, 5);
      expect(pairs.single.$2.distanceKm, 8);
    });
  });
}
