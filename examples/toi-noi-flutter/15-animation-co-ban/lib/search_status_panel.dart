import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'motion.dart';

/// Khung hiển thị trạng thái tìm tài xế. Nội dung đổi theo [status] và được
/// chuyển bằng `AnimatedSwitcher`: nội dung cũ mờ đi, nội dung mới hiện lên.
class SearchStatusPanel extends StatelessWidget {
  const SearchStatusPanel({super.key, required this.status});

  final TripStatus status;

  @override
  Widget build(BuildContext context) {
    return AnimatedSwitcher(
      duration: ToiNoiMotion.duration(context, ToiNoiMotion.standard),
      switchInCurve: Curves.easeOut,
      switchOutCurve: Curves.easeIn,
      // Key theo status: nhờ vậy AnimatedSwitcher thấy "một con khác" và chạy
      // chuyển cảnh. Thiếu key thì cùng kiểu widget nên chỉ cập nhật tại chỗ.
      child: _StatusMessage(key: ValueKey(status), status: status),
    );
  }
}

class _StatusMessage extends StatelessWidget {
  const _StatusMessage({super.key, required this.status});

  final TripStatus status;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        TripStatusChip(status: status),
        const SizedBox(height: ToiNoiSpacing.sm),
        Text(
          messageOf(status),
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyLarge,
        ),
      ],
    );
  }
}

/// Câu mô tả cho từng trạng thái. `switch` vét cạn, không có nhánh `_`.
String messageOf(TripStatus status) => switch (status) {
  TripStatus.requested => 'Đang gửi yêu cầu của bạn',
  TripStatus.searching => 'Đang tìm tài xế gần điểm đón',
  TripStatus.accepted => 'Đã có tài xế nhận chuyến',
  TripStatus.arriving => 'Tài xế đang trên đường tới điểm đón',
  TripStatus.pickedUp => 'Đã đón khách hoặc lấy hàng',
  TripStatus.inTransit => 'Đang trên đường tới điểm đến',
  TripStatus.completed => 'Chuyến đã hoàn thành',
  TripStatus.cancelled => 'Chuyến đã huỷ',
};
