import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import 'package:workmanager/workmanager.dart';

import 'background_tasks.dart';
import 'buffer_store.dart';
import 'location_sample.dart';
import 'location_uploader.dart';

/// Sink của lab: chỉ ghi log. Bản thật gọi API của Tới Nơi bằng token đọc từ
/// secure storage (bài 24); lab không có server nhận vị trí nên không giả vờ.
final class LoggingLocationSink implements LocationSink {
  const LoggingLocationSink();

  @override
  Future<void> send(List<LocationSample> batch) async {
    debugPrint('Gửi bù ${batch.length} mẫu vị trí (lab: chỉ ghi log)');
  }
}

/// Điểm vào của task nền, do hệ điều hành gọi trong một isolate riêng.
///
/// Phải là hàm cấp cao nhất (top-level) và có `@pragma('vm:entry-point')`:
/// plugin giữ một handle tới hàm này, và bản release (AOT) sẽ cắt bỏ hàm
/// không ai gọi trực tiếp nếu thiếu pragma.
@pragma('vm:entry-point')
void callbackDispatcher() {
  Workmanager().executeTask((taskName, inputData) async {
    try {
      final dir = await getApplicationSupportDirectory();
      return await runBackgroundTask(
        taskName,
        store: FileBufferStore(File('${dir.path}/pending_samples.json')),
        sink: const LoggingLocationSink(),
        now: () => DateTime.now().toUtc(),
      );
    } on Exception {
      return false;
    }
  });
}
