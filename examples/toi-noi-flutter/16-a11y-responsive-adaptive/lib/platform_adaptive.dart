import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// `true` khi theme hiện tại đang ở nền tảng Apple.
///
/// Đọc `Theme.of(context).platform`, không đọc `dart:io` `Platform`: giá trị
/// này đổi được theo từng nhánh cây (`Theme(data: ...copyWith(platform: ...))`)
/// và đổi được trong test bằng `debugDefaultTargetPlatformOverride`.
bool isApplePlatform(BuildContext context) =>
    switch (Theme.of(context).platform) {
      TargetPlatform.iOS || TargetPlatform.macOS => true,
      TargetPlatform.android ||
      TargetPlatform.fuchsia ||
      TargetPlatform.linux ||
      TargetPlatform.windows => false,
    };

/// Hỏi khách có chắc muốn huỷ [trip]. Trả `true` khi khách chọn huỷ.
///
/// Một hàm, hai giao diện: `showAdaptiveDialog` và `AlertDialog.adaptive`
/// dựng `CupertinoAlertDialog` trên iOS/macOS và `AlertDialog` ở nơi khác;
/// nút bấm ta chọn theo cùng một điều kiện.
Future<bool> confirmCancelTrip(BuildContext context, Trip trip) async {
  final result = await showAdaptiveDialog<bool>(
    context: context,
    builder: (dialogContext) => AlertDialog.adaptive(
      title: Text('Huỷ chuyến ${trip.id}?'),
      content: const Text('Chuyến sẽ chuyển sang trạng thái Đã huỷ.'),
      actions: [
        _DialogAction(label: 'Giữ chuyến', result: false),
        _DialogAction(label: 'Huỷ chuyến', result: true, destructive: true),
      ],
    ),
  );
  return result ?? false;
}

class _DialogAction extends StatelessWidget {
  const _DialogAction({
    required this.label,
    required this.result,
    this.destructive = false,
  });

  final String label;
  final bool result;
  final bool destructive;

  @override
  Widget build(BuildContext context) {
    void onPressed() => Navigator.of(context).pop(result);
    if (isApplePlatform(context)) {
      return CupertinoDialogAction(
        isDestructiveAction: destructive,
        onPressed: onPressed,
        child: Text(label),
      );
    }
    return TextButton(
      style: destructive
          ? TextButton.styleFrom(
              foregroundColor: Theme.of(context).colorScheme.error,
            )
          : null,
      onPressed: onPressed,
      child: Text(label),
    );
  }
}
