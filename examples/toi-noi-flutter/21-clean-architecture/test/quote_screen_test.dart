import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_21/composition.dart' show noProviderRetry;
import 'package:toi_noi_21/domain/domain.dart';
import 'package:toi_noi_21/presentation/app.dart';
import 'package:toi_noi_api/fault_injection.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_fake_api/toi_noi_fake_api.dart';

import 'support.dart';

Future<void> _pumpApp(WidgetTester tester, TripRepository repository) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: overridesFor(repository),
      retry: noProviderRetry,
      child: const ToiNoiApp(),
    ),
  );
}

Future<void> _tap(WidgetTester tester, String text) async {
  await tester.tap(find.text(text));
  await tester.pumpAndSettle();
}

void main() {
  group('với repository giả (không có dio, không có HTTP)', () {
    testWidgets('báo giá rồi đặt chuyến', (tester) async {
      final repository = FakeTripRepository();
      await _pumpApp(tester, repository);
      await _tap(tester, 'Báo giá');
      expect(find.text('25.500 VND'), findsOneWidget);
      await _tap(tester, 'Đặt chuyến');
      expect(find.text('Chuyến T-1'), findsOneWidget);
      expect(find.text('Chưa có tài xế'), findsOneWidget);
      expect(repository.requested.single.request, ride5km);
    });

    testWidgets('lỗi không thử lại được: câu chữ, không có nút Thử lại', (
      tester,
    ) async {
      final repository = FakeTripRepository()
        ..onQuote = (_) async =>
            throw const Rejected(RejectionKind.invalidDistance);
      await _pumpApp(tester, repository);
      await _tap(tester, 'Báo giá');
      expect(find.textContaining('0 đến 100 km'), findsOneWidget);
      expect(find.text('Thử lại'), findsNothing);
    });

    testWidgets('lỗi thử lại được: nút Thử lại gọi lại repository', (
      tester,
    ) async {
      final repository = FakeTripRepository();
      repository.onQuote = (request) async {
        if (repository.quoted.length == 1) {
          throw const Unreachable(UnreachableReason.noConnection);
        }
        return const Money(25500);
      };
      await _pumpApp(tester, repository);
      await _tap(tester, 'Báo giá');
      expect(find.textContaining('Không tới được máy chủ'), findsOneWidget);
      await _tap(tester, 'Thử lại');
      expect(find.text('25.500 VND'), findsOneWidget);
      expect(repository.quoted, hasLength(2));
    });

    testWidgets('đổi dịch vụ: use case nhận đúng yêu cầu', (tester) async {
      final repository = FakeTripRepository();
      await _pumpApp(tester, repository);
      await _tap(tester, 'Giao đồ ăn');
      await _tap(tester, 'Báo giá');
      await _tap(tester, 'Đặt chuyến');
      expect(repository.requested.single.request.serviceKind.name, 'food');
    });
  });

  group('với data thật + fake API trong tiến trình', () {
    testWidgets('500: mã hỗ trợ lên màn hình, Thử lại thành công', (
      tester,
    ) async {
      final harness = Harness(faults: const FaultConfig(failFirst: 1));
      await _pumpApp(tester, harness.repository);
      await _tap(tester, 'Báo giá');
      expect(find.textContaining('Máy chủ đang gặp sự cố'), findsOneWidget);
      expect(find.text('Mã hỗ trợ: fake-000001'), findsOneWidget);
      expect(find.textContaining('Unexpected'), findsNothing);
      await _tap(tester, 'Thử lại');
      expect(find.text('25.500 VND'), findsOneWidget);
    });

    testWidgets(
      'mất response khi đặt chuyến rồi Thử lại: một chuyến duy nhất',
      (tester) async {
        final harness = Harness(script: [null, NetworkFault.loseResponse]);
        await _pumpApp(tester, harness.repository);
        await _tap(tester, 'Báo giá');
        await _tap(tester, 'Đặt chuyến');
        expect(find.textContaining('Không tới được máy chủ'), findsOneWidget);
        await _tap(tester, 'Thử lại');
        expect(find.text('Chuyến T-1'), findsOneWidget);
        expect(harness.store.trips, hasLength(1));
      },
    );
  });
}
