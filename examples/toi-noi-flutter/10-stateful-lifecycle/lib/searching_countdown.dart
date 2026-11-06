import 'dart:async';

import 'package:flutter/material.dart';

/// Đếm ngược thời gian chờ khi Tới Nơi tìm tài xế.
///
/// Mỗi giây `Timer.periodic` trừ một giây còn lại. Khi về 0, timer dừng và
/// [onTimeout] được gọi một lần. Đổi [seconds] từ widget cha thì đếm lại từ
/// đầu (xem `didUpdateWidget`).
class SearchingCountdown extends StatefulWidget {
  const SearchingCountdown({
    super.key,
    this.seconds = 30,
    this.onTimeout,
    this.onCancel,
  }) : assert(seconds > 0, 'seconds phải dương');

  /// Tổng số giây chờ (số minh hoạ, không phải cam kết của sản phẩm).
  final int seconds;

  /// Gọi một lần khi đếm ngược về 0.
  final VoidCallback? onTimeout;

  /// Huỷ tìm: chạy bất đồng bộ (ví dụ gọi API). Để `null` thì nút bị khoá.
  final Future<void> Function()? onCancel;

  @override
  State<SearchingCountdown> createState() => _SearchingCountdownState();
}

class _SearchingCountdownState extends State<SearchingCountdown> {
  Timer? _timer;
  late int _remaining;
  bool _cancelling = false;
  bool _cancelled = false;
  bool _cancelFailed = false;

  @override
  void initState() {
    super.initState();
    _remaining = widget.seconds;
    _startTimer();
  }

  @override
  void didUpdateWidget(SearchingCountdown oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Không cần setState: framework luôn gọi build ngay sau đây.
    if (oldWidget.seconds != widget.seconds) {
      _remaining = widget.seconds;
      _cancelled = false;
      _cancelFailed = false;
      _startTimer();
    }
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 1), (_) => _tick());
  }

  void _tick() {
    setState(() {
      _remaining--;
      _cancelFailed = false;
    });
    if (_remaining == 0) {
      _timer?.cancel();
      widget.onTimeout?.call();
    }
  }

  Future<void> _cancel() async {
    final onCancel = widget.onCancel;
    if (onCancel == null || _cancelling) return;
    setState(() {
      _cancelling = true;
      _cancelFailed = false;
    });
    try {
      await onCancel();
    } catch (_) {
      // Chi tiết lỗi và ghi log thuộc bài về xử lý lỗi; ở đây chỉ báo người
      // dùng và mở lại nút.
      if (!mounted) return;
      setState(() {
        _cancelling = false;
        _cancelFailed = true;
      });
      return;
    }
    // Trong lúc await, widget cha có thể đã gỡ widget này khỏi cây.
    if (!mounted) return;
    _timer?.cancel();
    setState(() {
      _cancelling = false;
      _cancelled = true;
    });
  }

  String get _message {
    if (_cancelled) return 'Đã huỷ tìm tài xế';
    if (_cancelFailed) return 'Chưa huỷ được, hãy thử lại';
    if (_remaining == 0) return 'Chưa tìm được tài xế';
    return 'Đang tìm tài xế, còn $_remaining giây';
  }

  @override
  Widget build(BuildContext context) {
    final canCancel = !_cancelled && !_cancelling && widget.onCancel != null;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(_message),
        const SizedBox(height: 8),
        LinearProgressIndicator(value: _remaining / widget.seconds),
        const SizedBox(height: 8),
        TextButton(
          onPressed: canCancel ? _cancel : null,
          child: const Text('Huỷ tìm'),
        ),
      ],
    );
  }
}
