import 'dart:async';
import 'dart:math' as math;

class Position {
  const Position(this.lat, this.lng, this.at);

  final double lat;
  final double lng;
  final DateTime at;

  @override
  bool operator ==(Object other) =>
      other is Position &&
      lat == other.lat &&
      lng == other.lng &&
      at == other.at;

  @override
  int get hashCode => Object.hash(lat, lng, at);

  @override
  String toString() =>
      'Position(${lat.toStringAsFixed(4)}, ${lng.toStringAsFixed(4)})';
}

/// Khoảng cách xấp xỉ (equirectangular) giữa hai vị trí, tính bằng mét.
/// Đủ chính xác cho quãng ngắn trong thành phố; không dùng để định vị thật.
double distanceMeters(Position a, Position b) {
  const earthRadiusM = 6371000.0;
  final meanLat = (a.lat + b.lat) / 2 * math.pi / 180;
  final dx = (b.lng - a.lng) * math.pi / 180 * math.cos(meanLat);
  final dy = (b.lat - a.lat) * math.pi / 180;
  return earthRadiusM * math.sqrt(dx * dx + dy * dy);
}

/// Mô phỏng nguồn vị trí của một tài xế: phát lần lượt các điểm của [route].
class DriverLocationFeed {
  DriverLocationFeed(this.route);

  final List<Position> route;

  /// Phát một vị trí mỗi [every]. Timer chỉ chạy khi có người nghe.
  ///
  /// Mặc định là single-subscription: nghe lần hai ném StateError, tạm dừng
  /// (pause) thì nguồn dừng phát. Với [broadcast] = true, nhiều người cùng
  /// nghe được, nhưng người nghe muộn bỏ lỡ các vị trí đã phát.
  Stream<Position> positions({
    required Duration every,
    bool broadcast = false,
  }) {
    Timer? timer;
    var index = 0;
    late final StreamController<Position> controller;

    void stop() {
      timer?.cancel();
      timer = null;
    }

    void start() {
      timer = Timer.periodic(every, (_) {
        controller.add(route[index++]);
        if (index == route.length) {
          stop();
          controller.close();
        }
      });
    }

    controller = broadcast
        ? StreamController<Position>.broadcast(onListen: start, onCancel: stop)
        : StreamController<Position>(
            onListen: start,
            onPause: stop,
            onResume: start,
            onCancel: stop,
          );
    return controller.stream;
  }

  /// Chỉ giữ vị trí cách vị trí đã phát trước đó hơn [meters] mét.
  /// Vị trí đầu tiên luôn được phát.
  Stream<Position> movedMoreThan(
    double meters, {
    Duration every = const Duration(seconds: 1),
  }) => onlyMoved(positions(every: every), meters);
}

/// Biến đổi một stream vị trí thành stream chỉ gồm các lần di chuyển đáng kể.
Stream<Position> onlyMoved(Stream<Position> source, double meters) async* {
  Position? last; // trạng thái riêng cho từng lần listen
  await for (final p in source) {
    final previous = last;
    if (previous == null || distanceMeters(previous, p) > meters) {
      last = p;
      yield p;
    }
  }
}
