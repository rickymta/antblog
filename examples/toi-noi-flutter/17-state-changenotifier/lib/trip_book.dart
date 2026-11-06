import 'package:flutter/widgets.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Đồng hồ tiêm được để test không phụ thuộc giờ thật.
typedef UtcClock = DateTime Function();

DateTime systemUtcNow() => DateTime.now().toUtc();

/// Sổ chuyến của khách: thay cho `_trips` và `_replace` nằm trong `State` của
/// khung app ở bài 16.
///
/// Mỗi [Trip] bất biến; sổ thay cả danh sách bằng một danh sách mới (không sửa
/// tại chỗ) rồi báo một lần, nên ai giữ `trips` cũ không thấy nó đổi ngầm.
class TripBook extends ChangeNotifier {
  TripBook({
    List<Trip> initial = const [],
    this._clock = systemUtcNow,
    this._nextNumber = 1046,
  }) : _trips = List.unmodifiable(initial);

  final UtcClock _clock;
  int _nextNumber;
  List<Trip> _trips;

  /// Danh sách chỉ đọc, chuyến mới nhất ở đầu.
  List<Trip> get trips => _trips;

  Trip? byId(String id) => _trips.where((t) => t.id == id).firstOrNull;

  /// Đặt chuyến mới: `Trip.request` rồi `SearchStarted`. Ném [ArgumentError]
  /// khi [km] ngoài miền.
  Trip book(ServiceKind kind, double km) {
    final trip = Trip.request(
      id: 'T-${_nextNumber++}',
      serviceKind: kind,
      distanceKm: km,
      requestedAt: _clock(),
    ).apply(const SearchStarted());
    _trips = List.unmodifiable([trip, ..._trips]);
    notifyListeners();
    return trip;
  }

  /// Khách huỷ chuyến [id]. Ném [ArgumentError] nếu không có chuyến đó và
  /// [StateError] (từ `Trip.apply`) nếu trạng thái không cho huỷ.
  void cancel(String id) {
    final trip = byId(id);
    if (trip == null) {
      throw ArgumentError.value(id, 'id', 'không có chuyến này trong sổ');
    }
    final cancelled = trip.apply(const Cancelled(by: CancelledBy.customer));
    _trips = List.unmodifiable([
      for (final t in _trips) t.isSameTrip(cancelled) ? cancelled : t,
    ]);
    notifyListeners();
  }
}

/// Đưa [TripBook] xuống cây widget; cùng khuôn với `QuoteScope`.
class TripBookScope extends InheritedNotifier<TripBook> {
  const TripBookScope({super.key, required TripBook book, required super.child})
    : super(notifier: book);

  /// Đăng ký phụ thuộc: dựng lại khi sổ đổi.
  static TripBook of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<TripBookScope>()?.notifier ??
      (throw _missing(context));

  /// Không đăng ký phụ thuộc: dùng trong trình xử lý sự kiện.
  static TripBook read(BuildContext context) =>
      context.getInheritedWidgetOfExactType<TripBookScope>()?.notifier ??
      (throw _missing(context));

  static FlutterError _missing(BuildContext context) => FlutterError(
    'Không tìm thấy TripBookScope phía trên ${context.widget.runtimeType}.',
  );
}
