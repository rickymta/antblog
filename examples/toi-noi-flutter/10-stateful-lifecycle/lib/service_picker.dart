import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

/// Chọn một trong ba dịch vụ của Tới Nơi bằng [SegmentedButton].
///
/// Lựa chọn hiện tại là state của [ServicePicker] (`_ServicePickerState`),
/// không phải của widget cha. Cha chỉ cho giá trị ban đầu qua [initial] và
/// nhận thông báo qua [onChanged], giống `initialValue` của `TextFormField`.
class ServicePicker extends StatefulWidget {
  const ServicePicker({
    super.key,
    this.initial = ServiceKind.ride,
    this.onChanged,
    this.summaryBuilder,
  });

  /// Dịch vụ được chọn lúc widget xuất hiện. Đổi giá trị này về sau không
  /// đổi lựa chọn đang có: [initial] chỉ được đọc một lần ở `initState`.
  final ServiceKind initial;

  /// Gọi sau khi người dùng chọn một dịch vụ khác.
  final ValueChanged<ServiceKind>? onChanged;

  /// Dựng dòng tóm tắt dưới nút chọn. Mặc định in nhãn của dịch vụ đang chọn.
  /// Hàm này chạy lại mỗi lần `build` của state chạy.
  final Widget Function(BuildContext context, ServiceKind selected)?
  summaryBuilder;

  @override
  State<ServicePicker> createState() => _ServicePickerState();
}

class _ServicePickerState extends State<ServicePicker> {
  late ServiceKind _selected;

  @override
  void initState() {
    super.initState();
    _selected = widget.initial;
  }

  void _select(Set<ServiceKind> selection) {
    final kind = selection.first;
    setState(() => _selected = kind);
    widget.onChanged?.call(kind);
  }

  @override
  Widget build(BuildContext context) {
    final summaryBuilder = widget.summaryBuilder;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SegmentedButton<ServiceKind>(
          segments: [
            for (final kind in ServiceKind.values)
              ButtonSegment<ServiceKind>(value: kind, label: Text(kind.label)),
          ],
          selected: {_selected},
          onSelectionChanged: _select,
        ),
        const SizedBox(height: 12),
        if (summaryBuilder != null)
          summaryBuilder(context, _selected)
        else
          Text('Dịch vụ: ${_selected.label}'),
      ],
    );
  }
}
