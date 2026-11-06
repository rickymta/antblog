import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:toi_noi_api/toi_noi_api.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'providers.dart';

sealed class BookingState {
  const BookingState();
}

/// Đang nhập: chưa báo giá, hoặc vừa lỗi ([message] nếu có).
final class BookingIdle extends BookingState {
  const BookingIdle({this.message});

  final String? message;
}

final class BookingQuoting extends BookingState {
  const BookingQuoting();
}

/// Đã có giá. [idempotencyKey] sinh một lần cho ý định đặt chuyến này và dùng
/// lại khi người dùng bấm đặt lại sau lỗi: server nhận ra cùng khoá thì trả
/// đúng chuyến cũ thay vì tạo chuyến thứ hai. [message] là lỗi của lần đặt trước.
final class BookingQuoted extends BookingState {
  const BookingQuoted(
    this.request,
    this.fare,
    this.idempotencyKey, {
    this.message,
  });

  final QuoteRequestDto request;
  final Money fare;
  final String idempotencyKey;
  final String? message;
}

final class BookingSubmitting extends BookingState {
  const BookingSubmitting(this.quoted);

  final BookingQuoted quoted;

  Money get fare => quoted.fare;
}

/// Chuyến đã được tạo trên server; từ đây màn hình theo dõi chuyến.
final class BookingPlaced extends BookingState {
  const BookingPlaced(this.snapshot);

  final TripSnapshot snapshot;
}

final bookingProvider = NotifierProvider<BookingController, BookingState>(
  BookingController.new,
);

class BookingController extends Notifier<BookingState> {
  @override
  BookingState build() => const BookingIdle();

  Future<void> requestQuote(ServiceKind kind, double distanceKm) async {
    if (state is BookingQuoting || state is BookingSubmitting) {
      return;
    }
    final request = QuoteRequestDto(serviceKind: kind, distanceKm: distanceKm);
    state = const BookingQuoting();
    try {
      final quote = await guardApi(
        () => ref.read(apiClientProvider).quote(request),
      );
      if (ref.mounted) {
        state = BookingQuoted(
          request,
          quote.toMoney(),
          ref.read(idempotencyKeyProvider)(),
        );
      }
    } on ApiFailure catch (failure) {
      if (ref.mounted) {
        state = BookingIdle(message: failureMessage(failure));
      }
    }
  }

  Future<void> book() async {
    final current = state;
    if (current is! BookingQuoted) {
      return;
    }
    state = BookingSubmitting(current);
    try {
      final trip = await guardApi(
        () => ref
            .read(apiClientProvider)
            .createTrip(
              current.request,
              idempotencyKey: current.idempotencyKey,
            ),
      );
      if (ref.mounted) {
        state = BookingPlaced(trip.toSnapshot());
      }
    } on ApiFailure catch (failure) {
      if (ref.mounted) {
        state = BookingQuoted(
          current.request,
          current.fare,
          current.idempotencyKey,
          message: failureMessage(failure),
        );
      }
    }
  }

  /// Quay lại màn hình nhập để đặt chuyến khác.
  void startOver() => state = const BookingIdle();
}

/// Câu chữ cho người dùng. Rút gọn từ `describeFailure` của bài 20: lab này
/// chỉ cần phân biệt "lỗi của ta" với "lỗi mạng hay máy chủ".
String failureMessage(ApiFailure failure) => switch (failure) {
  ClientFailure(statusCode: 422) =>
    'Quãng đường phải nằm trong khoảng 0 đến 100 km.',
  ClientFailure() => 'Yêu cầu không được chấp nhận.',
  NetworkFailure() ||
  TimeoutFailure() => 'Không tới được máy chủ. Hãy kiểm tra mạng rồi thử lại.',
  ServerFailure() => 'Máy chủ đang gặp sự cố. Hãy thử lại sau ít phút.',
  ParseFailure() => 'Dữ liệu nhận về không đọc được.',
};
