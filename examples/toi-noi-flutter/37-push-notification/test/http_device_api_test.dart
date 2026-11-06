import 'dart:typed_data';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:toi_noi_37/device_api.dart';

/// Adapter ghi lại request và trả về mã trạng thái định sẵn (hoặc lỗi mạng).
class _RecordingAdapter implements HttpClientAdapter {
  _RecordingAdapter({this.status = 204, this.networkError = false});

  final int status;
  final bool networkError;
  final requests = <RequestOptions>[];

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    requests.add(options);
    if (networkError) {
      throw DioException.connectionError(
        requestOptions: options,
        reason: 'không có mạng',
      );
    }
    return ResponseBody.fromString('', status);
  }

  @override
  void close({bool force = false}) {}
}

HttpDeviceApi _api(_RecordingAdapter adapter) {
  final dio = Dio(BaseOptions(baseUrl: 'https://api.toinoi.example'))
    ..httpClientAdapter = adapter;
  return HttpDeviceApi(dio);
}

void main() {
  test('upsert: PUT /devices/{deviceId}, định danh nằm trong body', () async {
    final adapter = _RecordingAdapter();
    await _api(adapter)
        .upsert(deviceId: 'dev-A', pushId: 'fid-secret-1', platform: 'ios');

    final request = adapter.requests.single;
    expect(request.method, 'PUT');
    expect(request.uri.toString(), 'https://api.toinoi.example/devices/dev-A');
    expect(request.data, {
      'provider': 'fcm',
      'fid': 'fid-secret-1',
      'platform': 'ios',
    });
  });

  test('định danh push không bao giờ nằm trong URL', () async {
    final adapter = _RecordingAdapter();
    final api = _api(adapter);
    await api.upsert(
      deviceId: 'dev-A',
      pushId: 'fid-secret-1',
      platform: 'ios',
    );
    await api.remove('dev-A');
    for (final request in adapter.requests) {
      expect(request.uri.toString(), isNot(contains('fid-secret-1')));
    }
  });

  test('deviceId lạ được mã hoá, không vỡ đường dẫn', () async {
    final adapter = _RecordingAdapter();
    await _api(adapter).remove('a/b?c');
    expect(adapter.requests.single.uri.path, '/devices/a%2Fb%3Fc');
  });

  test('remove: DELETE /devices/{deviceId}', () async {
    final adapter = _RecordingAdapter();
    await _api(adapter).remove('dev-A');
    expect(adapter.requests.single.method, 'DELETE');
    expect(adapter.requests.single.uri.path, '/devices/dev-A');
  });

  test('remove: 404 coi như đã xoá', () async {
    await _api(_RecordingAdapter(status: 404)).remove('dev-A');
  });

  test('upsert: 404 vẫn là lỗi', () async {
    await expectLater(
      _api(_RecordingAdapter(status: 404))
          .upsert(deviceId: 'd', pushId: 'f', platform: 'ios'),
      throwsA(
        isA<DeviceApiException>().having(
          (e) => e.message,
          'message',
          'HTTP 404',
        ),
      ),
    );
  });

  test('5xx thành DeviceApiException có mã trạng thái', () async {
    await expectLater(
      _api(_RecordingAdapter(status: 503)).remove('dev-A'),
      throwsA(
        isA<DeviceApiException>().having(
          (e) => e.message,
          'message',
          'HTTP 503',
        ),
      ),
    );
  });

  test('mất mạng thành DeviceApiException, không lộ DioException', () async {
    await expectLater(
      _api(_RecordingAdapter(networkError: true)).remove('dev-A'),
      throwsA(isA<DeviceApiException>()),
    );
  });
}
