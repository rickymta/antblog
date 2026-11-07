import 'package:flutter/material.dart';

import 'location_access.dart';
import 'location_access_controller.dart';
import 'location_models.dart';

/// Khoá cho test.
abstract final class PanelKeys {
  static const action = Key('location-action');
  static const status = Key('location-status');
  static const precise = Key('location-precise');
  static const dialogContinue = Key('location-dialog-continue');
  static const dialogLater = Key('location-dialog-later');
}

/// Khối "dùng vị trí hiện tại" ở màn hình chọn điểm đón.
///
/// Không xin quyền khi dựng màn hình: chỉ xin sau khi người dùng bấm nút và
/// đọc lời giải thích.
class PickupLocationPanel extends StatelessWidget {
  const PickupLocationPanel({
    super.key,
    required this.controller,
    this.tripActive = false,
  });

  final LocationAccessController controller;
  final bool tripActive;

  @override
  Widget build(BuildContext context) {
    return ValueListenableBuilder<LocationAccessState>(
      valueListenable: controller,
      builder: (context, state, _) {
        final advice = adviceFor(
          state,
          controller.role,
          tripActive: tripActive,
          backgroundAvailable: controller.backgroundSupported,
        );
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(_statusText(state), key: PanelKeys.status),
            const SizedBox(height: 8),
            if (_buttonLabel(advice) case final label?)
              FilledButton(
                key: PanelKeys.action,
                onPressed: () => _onAction(context, advice),
                child: Text(label),
              ),
            if (state.precision == LocationPrecision.reduced)
              TextButton(
                key: PanelKeys.precise,
                onPressed: controller.requestPrecise,
                child: const Text('Cho phép vị trí chính xác'),
              ),
          ],
        );
      },
    );
  }

  static String _statusText(LocationAccessState state) => switch (state) {
    NotAsked() => 'Chưa dùng vị trí',
    Asking() => 'Đang chờ bạn trả lời hộp thoại của hệ thống',
    WhileInUse(precision: LocationPrecision.reduced) =>
      'Vị trí gần đúng: điểm đón có thể lệch, hãy kéo ghim để sửa',
    WhileInUse() => 'Đang dùng vị trí khi mở app',
    Always() => 'Được dùng vị trí cả khi app ở nền',
    Denied() => 'Bạn đã từ chối vị trí',
    DeniedForever() => 'Vị trí đang tắt cho Tới Nơi trong Cài đặt',
    ServiceOff() => 'Dịch vụ vị trí của điện thoại đang tắt',
  };

  static String? _buttonLabel(LocationAdvice advice) => switch (advice) {
    LocationAdvice.wait || LocationAdvice.ready => null,
    LocationAdvice.explainThenRequest => 'Dùng vị trí hiện tại',
    LocationAdvice.offerBackground => 'Cho phép gửi vị trí khi app ở nền',
    LocationAdvice.openAppSettings => 'Mở Cài đặt của Tới Nơi',
    LocationAdvice.openLocationSettings => 'Bật dịch vụ vị trí',
  };

  Future<void> _onAction(BuildContext context, LocationAdvice advice) async {
    switch (advice) {
      case LocationAdvice.explainThenRequest:
        final ok = await _explain(
          context,
          title: 'Tới Nơi dùng vị trí để làm gì?',
          body:
              'Để đặt ghim điểm đón ngay chỗ bạn đứng và tìm tài xế gần nhất. '
              'Tới Nơi chỉ lấy vị trí khi bạn đang mở app.',
        );
        if (ok) {
          await controller.requestForeground();
        }
      case LocationAdvice.offerBackground:
        final ok = await _explain(
          context,
          title: 'Gửi vị trí khi app ở nền',
          body:
              'Trong chuyến, khách theo dõi xe của bạn trên bản đồ. Cho phép '
              'để vị trí vẫn được gửi khi bạn chuyển sang app dẫn đường. '
              'Hết chuyến, Tới Nơi ngừng gửi.',
        );
        if (ok) {
          await controller.requestBackground();
        }
      case LocationAdvice.openAppSettings:
        await controller.openAppSettings();
      case LocationAdvice.openLocationSettings:
        await controller.openLocationSettings();
      case LocationAdvice.wait || LocationAdvice.ready:
        break;
    }
  }

  Future<bool> _explain(
    BuildContext context, {
    required String title,
    required String body,
  }) async {
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(title),
        content: Text(body),
        actions: [
          TextButton(
            key: PanelKeys.dialogLater,
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Để sau'),
          ),
          FilledButton(
            key: PanelKeys.dialogContinue,
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('Tiếp tục'),
          ),
        ],
      ),
    );
    return result ?? false;
  }
}
