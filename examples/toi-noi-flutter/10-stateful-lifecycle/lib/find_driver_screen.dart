import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'searching_countdown.dart';
import 'service_picker.dart';

/// Thời gian chờ tìm tài xế theo dịch vụ. Số minh hoạ để bài giảng có tham
/// số đổi được, không phải cam kết của sản phẩm.
int searchSecondsFor(ServiceKind kind) => switch (kind) {
  ServiceKind.ride => 30,
  ServiceKind.food => 45,
  ServiceKind.parcel => 60,
};

/// Màn hình ghép hai widget: chọn dịch vụ rồi bấm tìm tài xế.
///
/// Đổi dịch vụ khi đang tìm làm [SearchingCountdown] nhận `seconds` mới và
/// đếm lại từ đầu qua `didUpdateWidget`, không cần GlobalKey.
class FindDriverScreen extends StatefulWidget {
  const FindDriverScreen({super.key, this.cancelDelay = Duration.zero});

  /// Độ trễ của thao tác huỷ giả lập (thay cho một lời gọi API thật).
  final Duration cancelDelay;

  @override
  State<FindDriverScreen> createState() => _FindDriverScreenState();
}

class _FindDriverScreenState extends State<FindDriverScreen> {
  ServiceKind _kind = ServiceKind.ride;
  bool _searching = false;
  String? _notice;

  void _startSearch() => setState(() {
    _searching = true;
    _notice = null;
  });

  void _onTimeout() => setState(() {
    _searching = false;
    _notice = 'Chưa tìm được tài xế, hãy thử lại';
  });

  Future<void> _cancelSearch() async {
    await Future<void>.delayed(widget.cancelDelay);
    // Người dùng có thể đã thoát màn hình trong lúc chờ: state đã bị dispose.
    if (!mounted) return;
    setState(() {
      _searching = false;
      _notice = 'Bạn đã huỷ tìm tài xế';
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Tìm tài xế')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            ServicePicker(onChanged: (kind) => setState(() => _kind = kind)),
            const SizedBox(height: 24),
            if (_searching)
              SearchingCountdown(
                seconds: searchSecondsFor(_kind),
                onTimeout: _onTimeout,
                onCancel: _cancelSearch,
              )
            else ...[
              if (_notice != null) Text(_notice!),
              FilledButton(
                onPressed: _startSearch,
                child: const Text('Tìm tài xế'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
