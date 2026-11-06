import 'package:dio/dio.dart';

/// Backend lưu một dòng cho mỗi lần cài app (`deviceId`), kèm định danh push
/// hiện tại. Khoá theo `deviceId` chứ không theo định danh push: định danh đổi
/// thì cập nhật cùng dòng, đăng xuất thì xoá đúng dòng đó.
abstract interface class DeviceApi {
  Future<void> upsert({
    required String deviceId,
    required String pushId,
    required String platform,
  });

  Future<void> remove(String deviceId);
}

/// Backend trả lỗi hoặc không liên lạc được.
final class DeviceApiException implements Exception {
  const DeviceApiException(this.message);

  final String message;

  @override
  String toString() => 'DeviceApiException($message)';
}

/// Bản dùng `dio`. Định danh push đi trong body, không đi trong URL: URL hay
/// bị ghi vào log của proxy và máy chủ.
final class HttpDeviceApi implements DeviceApi {
  HttpDeviceApi(this._dio);

  final Dio _dio;

  @override
  Future<void> upsert({
    required String deviceId,
    required String pushId,
    required String platform,
  }) async {
    await _send(
      () => _dio.put<void>(
        '/devices/${Uri.encodeComponent(deviceId)}',
        data: {'provider': 'fcm', 'fid': pushId, 'platform': platform},
      ),
    );
  }

  @override
  Future<void> remove(String deviceId) async {
    await _send(
      () => _dio.delete<void>('/devices/${Uri.encodeComponent(deviceId)}'),
      // Đã không còn dòng nào thì mục đích của việc xoá đã đạt.
      okStatuses: const {404},
    );
  }

  Future<void> _send(
    Future<Response<void>> Function() call, {
    Set<int> okStatuses = const {},
  }) async {
    try {
      await call();
    } on DioException catch (e) {
      final status = e.response?.statusCode;
      if (status != null && okStatuses.contains(status)) return;
      throw DeviceApiException(
        status == null ? 'không kết nối được (${e.type.name})' : 'HTTP $status',
      );
    }
  }
}
