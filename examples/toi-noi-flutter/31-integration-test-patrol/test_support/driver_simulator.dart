import 'package:dio/dio.dart';
import 'package:toi_noi_api/toi_noi_api.dart';

/// Tên header mà fake API đọc để biết ai đang nhận chuyến. Test trong `test/`
/// đối chiếu hằng này với `fakeDriverIdHeader` của `toi_noi_fake_api`; nó được
/// chép ở đây để test chạy trên thiết bị không phải kéo cả server vào bản build.
const driverIdHeader = 'x-fake-driver-id';

/// Một "tài xế" nói chuyện với cùng server mà app khách đang dùng. Test gọi
/// [accept] để chuyển chuyến sang `accepted` đúng lúc, thay cho app tài xế thật.
final class DriverSimulator {
  DriverSimulator({
    required Uri baseUrl,
    HttpClientAdapter? adapter,
    this.driverId = 'D-01',
  }) : _client = ToiNoiApiClient(
         baseUrl: baseUrl,
         adapter: adapter,
         interceptors: [
           InterceptorsWrapper(
             onRequest: (options, handler) {
               options.headers[driverIdHeader] = driverId;
               handler.next(options);
             },
           ),
         ],
       );

  final String driverId;
  final ToiNoiApiClient _client;

  /// Đọc phiên bản hiện tại rồi nhận chuyến với đúng phiên bản đó.
  Future<void> accept(String tripId) async {
    final current = await _client.getTrip(tripId);
    await _client.acceptTrip(tripId, expectedVersion: current.version);
  }

  void close() => _client.close();
}
