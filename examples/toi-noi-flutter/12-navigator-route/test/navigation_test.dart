import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_12/app.dart';
import 'package:toi_noi_12/confirm_trip_screen.dart';
import 'package:toi_noi_12/quote.dart';
import 'package:toi_noi_12/quote_screen.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

final _fixedNow = DateTime.utc(2026, 11, 6, 8, 30);
DateTime _clock() => _fixedNow;

Future<void> _openConfirm(WidgetTester tester) async {
  await tester.tap(find.text('Đặt chuyến'));
  await tester.pumpAndSettle();
}

Future<void> _chooseWallet(WidgetTester tester) async {
  await tester.tap(find.text('Thanh toán'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('Ví VíXanh').last);
  await tester.pumpAndSettle();
}

void main() {
  group('truyền Quote qua constructor', () {
    testWidgets('màn hình xác nhận hiển thị đúng dữ liệu báo giá', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);

      expect(find.text('Xác nhận chuyến'), findsOneWidget);
      expect(find.text('Đón: Ngã tư Hoa Sữa'), findsOneWidget);
      expect(find.text('Đến: Công viên Tre Xanh'), findsOneWidget);
      expect(find.text('25.500 VND'), findsOneWidget);
    });

    testWidgets('cước lấy từ calculateFare: 2,001 km là 12.005 VND', (
      tester,
    ) async {
      const quote = Quote(
        id: '0002',
        serviceKind: ServiceKind.food,
        distanceKm: 2.001,
        pickup: 'Quán Cơm Nhà',
        dropoff: 'Ngã tư Hoa Sữa',
      );
      await tester.pumpWidget(const ToiNoiApp(quote: quote, clock: _clock));
      expect(find.text('12.005 VND'), findsOneWidget);

      await _openConfirm(tester);
      expect(find.text('12.005 VND'), findsOneWidget);
    });
  });

  group('push rồi pop nhận kết quả có kiểu', () {
    testWidgets('xác nhận trả về Trip đúng nội dung', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsNothing);
      expect(find.text('Đã đặt chuyến T-0001: 25.500 VND'), findsOneWidget);
      expect(find.text('Đang tìm tài xế cho T-0001'), findsOneWidget);
    });

    testWidgets('Future của push hoàn thành với đúng đối tượng Trip', (
      tester,
    ) async {
      late BuildContext homeContext;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              homeContext = context;
              return const Scaffold();
            },
          ),
        ),
      );

      Trip? result;
      var completed = false;
      Navigator.of(homeContext)
          .push<Trip>(
            MaterialPageRoute<Trip>(
              builder: (_) =>
                  const ConfirmTripScreen(quote: demoQuote, clock: _clock),
            ),
          )
          .then((trip) {
            result = trip;
            completed = true;
          });
      await tester.pumpAndSettle();
      expect(completed, isFalse, reason: 'chưa pop thì Future chưa xong');

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(completed, isTrue);
      expect(
        result,
        Trip.request(
          id: 'T-0001',
          serviceKind: ServiceKind.ride,
          distanceKm: 5,
          requestedAt: _fixedNow,
        ),
      );
      expect(result!.status, TripStatus.requested);
      expect(result!.fare, const Money(25500));
    });

    testWidgets('pop sai kiểu kết quả bị chặn khi bật assertions', (
      tester,
    ) async {
      late BuildContext homeContext;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              homeContext = context;
              return const Scaffold();
            },
          ),
        ),
      );
      Navigator.of(homeContext).push<Trip>(
        MaterialPageRoute<Trip>(
          builder: (_) =>
              const ConfirmTripScreen(quote: demoQuote, clock: _clock),
        ),
      );
      await tester.pumpAndSettle();

      // Route khai báo Trip, ta pop một String: assert trong Navigator bắt lỗi.
      expect(
        () => Navigator.of(homeContext).pop('đã đặt'),
        throwsA(
          isA<FlutterError>().having(
            (e) => e.message,
            'message',
            contains('result of type String'),
          ),
        ),
      );
    });

    testWidgets('back khi chưa thay đổi gì trả null', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);

      await tester.pageBack();
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsNothing);
      expect(find.text('Bạn đã quay lại, chưa đặt chuyến'), findsOneWidget);
      expect(find.byType(SnackBar), findsNothing);
    });
  });

  group('bottom sheet chọn phương thức trả tiền', () {
    testWidgets('chọn Ví VíXanh cập nhật màn hình xác nhận', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      expect(find.text('Tiền mặt'), findsOneWidget);

      await _chooseWallet(tester);

      expect(find.text('Ví VíXanh'), findsOneWidget);
      expect(find.text('Tiền mặt'), findsNothing);
    });

    testWidgets(
      'đóng sheet không chọn thì giữ lựa chọn cũ và back không bị chặn',
      (tester) async {
        await tester.pumpWidget(const ToiNoiApp(clock: _clock));
        await _openConfirm(tester);

        await tester.tap(find.text('Thanh toán'));
        await tester.pumpAndSettle();
        // Chạm vào vùng mờ phía trên để đóng sheet: kết quả là null.
        await tester.tapAt(const Offset(10, 10));
        await tester.pumpAndSettle();

        expect(find.text('Tiền mặt'), findsOneWidget);
        await tester.pageBack();
        await tester.pumpAndSettle();
        expect(find.byType(ConfirmTripScreen), findsNothing);
        expect(find.text('Rời màn hình?'), findsNothing);
      },
    );
  });

  group('PopScope chặn back khi có thay đổi', () {
    testWidgets('back hiện hộp thoại và ở lại thì giữ nguyên màn hình', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      await _chooseWallet(tester);

      await tester.pageBack();
      await tester.pumpAndSettle();
      expect(find.text('Rời màn hình?'), findsOneWidget);
      expect(find.byType(ConfirmTripScreen), findsOneWidget);

      await tester.tap(find.text('Ở lại'));
      await tester.pumpAndSettle();
      expect(find.text('Rời màn hình?'), findsNothing);
      expect(find.byType(ConfirmTripScreen), findsOneWidget);
      expect(find.text('Ví VíXanh'), findsOneWidget);
    });

    testWidgets('đổi lại về Tiền mặt thì back không còn bị chặn', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      await _chooseWallet(tester);

      await tester.tap(find.text('Thanh toán'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Tiền mặt').last);
      await tester.pumpAndSettle();

      await tester.pageBack();
      await tester.pumpAndSettle();

      expect(find.text('Rời màn hình?'), findsNothing);
      expect(find.byType(ConfirmTripScreen), findsNothing);
    });

    testWidgets('chạm ra ngoài hộp thoại (kết quả null) thì ở lại', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      await _chooseWallet(tester);
      await tester.pageBack();
      await tester.pumpAndSettle();

      await tester.tapAt(const Offset(5, 5));
      await tester.pumpAndSettle();

      expect(find.text('Rời màn hình?'), findsNothing);
      expect(find.byType(ConfirmTripScreen), findsOneWidget);
    });

    testWidgets('chọn Rời đi thì pop thật và kết quả là null', (tester) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      await _chooseWallet(tester);

      await tester.pageBack();
      await tester.pumpAndSettle();
      await tester.tap(find.text('Rời đi'));
      await tester.pumpAndSettle();

      expect(find.byType(ConfirmTripScreen), findsNothing);
      expect(find.text('Bạn đã quay lại, chưa đặt chuyến'), findsOneWidget);
    });

    testWidgets('onPopInvokedWithResult báo didPop false khi bị chặn', (
      tester,
    ) async {
      final calls = <bool>[];
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () => Navigator.of(context).push<void>(
                  MaterialPageRoute<void>(
                    builder: (_) => PopScope<void>(
                      canPop: false,
                      onPopInvokedWithResult: (didPop, result) =>
                          calls.add(didPop),
                      child: const Scaffold(body: Text('chặn')),
                    ),
                  ),
                ),
                child: const Text('mở'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('mở'));
      await tester.pumpAndSettle();

      // Giả lập nút back hệ thống của Android (route này không có AppBar).
      await tester.binding.handlePopRoute();
      await tester.pumpAndSettle();

      expect(calls, [false]);
      expect(find.text('chặn'), findsOneWidget);
    });

    testWidgets('nút Xác nhận vẫn pop được khi có thay đổi chưa lưu', (
      tester,
    ) async {
      await tester.pumpWidget(const ToiNoiApp(clock: _clock));
      await _openConfirm(tester);
      await _chooseWallet(tester);

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(find.text('Rời màn hình?'), findsNothing);
      expect(find.text('Đã đặt chuyến T-0001: 25.500 VND'), findsOneWidget);
    });
  });

  group('kiểm mounted sau await', () {
    testWidgets('QuoteScreen bị gỡ khi màn hình xác nhận còn mở: không lỗi', (
      tester,
    ) async {
      final showQuote = ValueNotifier<bool>(true);
      addTearDown(showQuote.dispose);
      await tester.pumpWidget(
        MaterialApp(
          home: ValueListenableBuilder<bool>(
            valueListenable: showQuote,
            builder: (_, visible, _) => visible
                ? const QuoteScreen(quote: demoQuote, clock: _clock)
                : const Scaffold(body: Text('đã gỡ báo giá')),
          ),
        ),
      );
      await _openConfirm(tester);

      showQuote.value = false;
      await tester.pump();
      expect(find.byType(QuoteScreen), findsNothing);

      await tester.tap(find.text('Xác nhận đặt chuyến'));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(find.text('đã gỡ báo giá'), findsOneWidget);
    });

    testWidgets(
      'cùng tình huống nhưng không kiểm mounted thì setState ném lỗi',
      (tester) async {
        final showButton = ValueNotifier<bool>(true);
        addTearDown(showButton.dispose);
        Object? caught;
        await tester.pumpWidget(
          MaterialApp(
            home: ValueListenableBuilder<bool>(
              valueListenable: showButton,
              builder: (_, visible, _) => visible
                  ? _UnguardedButton(onError: (error) => caught = error)
                  : const Scaffold(),
            ),
          ),
        );
        await tester.tap(find.text('Đặt chuyến'));
        await tester.pumpAndSettle();

        showButton.value = false;
        await tester.pump();
        await tester.tap(find.text('Xác nhận đặt chuyến'));
        await tester.pumpAndSettle();

        expect(caught, isA<FlutterError>());
        expect(
          caught.toString(),
          contains('setState() called after dispose()'),
        );
      },
    );

    testWidgets(
      'hộp thoại rời đi: màn hình bị gỡ trước khi bấm thì không pop nhầm',
      (tester) async {
        await tester.pumpWidget(const ToiNoiApp(clock: _clock));
        await _openConfirm(tester);
        await _chooseWallet(tester);
        await tester.pageBack();
        await tester.pumpAndSettle();
        expect(find.text('Rời màn hình?'), findsOneWidget);

        // Gỡ route xác nhận khỏi Navigator trong lúc hộp thoại còn mở.
        final route = ModalRoute.of(
          tester.element(find.byType(ConfirmTripScreen)),
        )!;
        tester.state<NavigatorState>(find.byType(Navigator)).removeRoute(route);
        await tester.pump();
        expect(find.byType(ConfirmTripScreen), findsNothing);

        await tester.tap(find.text('Rời đi'));
        await tester.pumpAndSettle();

        expect(tester.takeException(), isNull);
        // Không route nào khác bị pop nhầm: vẫn ở màn hình báo giá.
        expect(find.byType(QuoteScreen), findsOneWidget);
      },
    );
  });
}

/// Phiên bản sai cố ý: dùng setState sau await mà không kiểm `mounted`.
class _UnguardedButton extends StatefulWidget {
  const _UnguardedButton({required this.onError});

  final void Function(Object error) onError;

  @override
  State<_UnguardedButton> createState() => _UnguardedButtonState();
}

class _UnguardedButtonState extends State<_UnguardedButton> {
  Trip? _trip;

  Future<void> _book() async {
    final trip = await Navigator.of(context).push<Trip>(
      MaterialPageRoute<Trip>(
        builder: (_) =>
            const ConfirmTripScreen(quote: demoQuote, clock: _clock),
      ),
    );
    try {
      setState(() => _trip = trip); // thiếu: if (!mounted) return;
    } catch (error) {
      widget.onError(error);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: TextButton(
        onPressed: _book,
        child: Text(_trip == null ? 'Đặt chuyến' : 'Xong'),
      ),
    );
  }
}
