import 'dart:async';

/// Lỗi nghiệp vụ của dịch vụ báo giá (khác với lỗi lập trình như ArgumentError).
class QuoteException implements Exception {
  QuoteException(this.message, {this.cause});

  final String message;
  final Object? cause;

  @override
  String toString() => 'QuoteException: $message';
}

/// Bảng giá minh hoạ: 12.000 VND cho 2 km đầu, 4.500 VND mỗi km vượt,
/// miền hợp lệ 0 đến 100 km. Không phải bảng giá thực tế.
int calculateFareVnd(double distanceKm) {
  if (!(distanceKm >= 0 && distanceKm <= 100)) {
    throw ArgumentError.value(
      distanceKm,
      'distanceKm',
      'phải nằm trong khoảng 0 đến 100 km',
    );
  }
  // Tính bằng số nguyên mét để 2,001 km luôn ra 12.005 đồng (làm tròn nửa lên).
  final meters = (distanceKm * 1000).round();
  final extraMeters = meters > 2000 ? meters - 2000 : 0;
  return 12000 + (extraMeters * 4500 + 500) ~/ 1000;
}

/// Mô phỏng một dịch vụ báo giá gọi qua mạng: trả lời sau [latency].
class QuoteService {
  QuoteService({this.latency = const Duration(milliseconds: 300)});

  final Duration latency;

  /// Số lần gọi đã chạy xong phía "server", kể cả khi người gọi đã bỏ cuộc.
  int completedCalls = 0;

  Future<int> fetchFareVnd(double distanceKm) async {
    await Future<void>.delayed(latency);
    completedCalls++;
    try {
      return calculateFareVnd(distanceKm);
    } on ArgumentError catch (e) {
      throw QuoteException('Khoảng cách $distanceKm km không hợp lệ', cause: e);
    }
  }
}

/// Chờ tối đa [timeout]. Quá hạn thì ném [TimeoutException].
/// Lưu ý: việc đang chạy phía service không bị huỷ.
Future<int> fetchWithTimeout(
  QuoteService service,
  double distanceKm, {
  required Duration timeout,
}) {
  return service
      .fetchFareVnd(distanceKm)
      .timeout(
        timeout,
        onTimeout: () => throw TimeoutException(
          'Báo giá quá ${timeout.inMilliseconds} ms',
          timeout,
        ),
      );
}

/// Gọi song song nhiều quãng đường; kết quả giữ đúng thứ tự đầu vào.
Future<List<int>> fetchFaresInParallel(
  QuoteService service,
  List<double> distancesKm,
) {
  return Future.wait(distancesKm.map(service.fetchFareVnd));
}

/// Lấy báo giá của nguồn trả lời đầu tiên thành công hay thất bại.
Future<int> fetchFromFastest(List<QuoteService> services, double distanceKm) {
  return Future.any(services.map((s) => s.fetchFareVnd(distanceKm)));
}
