import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_17/sample_trips.dart';
import 'package:toi_noi_17/trip_book.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

DateTime _clock() => DateTime.utc(2026, 11, 6, 8, 30);

void main() {
  group('TripBook', () {
    late TripBook book;
    late int notifications;

    setUp(() {
      book = TripBook(initial: sampleTrips(), clock: _clock);
      notifications = 0;
      book.addListener(() => notifications++);
      addTearDown(book.dispose);
    });

    test('book: chuyến mới ở đầu, đang tìm tài xế, cước từ calculateFare', () {
      final trip = book.book(ServiceKind.ride, 5);

      expect(book.trips.first, trip);
      expect(trip.id, 'T-1046');
      expect(trip.status, TripStatus.searching);
      expect(trip.fare, calculateFare(5));
      expect(trip.requestedAt, _clock());
      expect(book.trips, hasLength(5));
      expect(notifications, 1);
    });

    test('id tăng dần theo từng lần đặt', () {
      expect(book.book(ServiceKind.food, 2).id, 'T-1046');
      expect(book.book(ServiceKind.parcel, 3).id, 'T-1047');
    });

    test('book với km ngoài miền: ném ArgumentError, sổ không đổi', () {
      expect(() => book.book(ServiceKind.ride, 101), throwsArgumentError);
      expect(book.trips, hasLength(4));
      expect(notifications, 0);
    });

    test('danh sách chỉ đọc, và bản cũ không đổi ngầm sau khi sổ thay đổi', () {
      final before = book.trips;
      expect(() => before.add(before.first), throwsUnsupportedError);

      book.book(ServiceKind.ride, 5);

      expect(before, hasLength(4));
      expect(book.trips, hasLength(5));
      expect(identical(before, book.trips), isFalse);
    });

    test('cancel ở searching: chuyến thành cancelled, vị trí giữ nguyên', () {
      book.cancel('T-1045'); // searching trong dữ liệu mẫu

      expect(book.byId('T-1045')!.status, TripStatus.cancelled);
      expect(book.trips.first.id, 'T-1045');
      expect(notifications, 1);
    });

    test(
      'cancel chuyến đã hoàn thành: StateError từ Trip.apply, không báo',
      () {
        expect(() => book.cancel('T-1043'), throwsStateError);
        expect(book.byId('T-1043')!.status, TripStatus.completed);
        expect(notifications, 0);
      },
    );

    test('cancel id không có: ArgumentError', () {
      expect(() => book.cancel('T-0'), throwsArgumentError);
      expect(notifications, 0);
    });
  });
}
