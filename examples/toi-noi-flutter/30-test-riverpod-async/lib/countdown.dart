import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Tài xế có bao lâu để nhận một cuốc (minh hoạ).
const offerWindow = Duration(seconds: 15);

/// Hạn trả lời cho một cuốc được đề nghị lúc [offeredAt].
DateTime offerDeadline(DateTime offeredAt) => offeredAt.add(offerWindow);

/// Còn bao lâu nữa tới [deadline], phát lại mỗi [tick]; phát `Duration.zero`
/// rồi đóng khi hết hạn.
///
/// Phần còn lại luôn tính từ `clock.now()` so với [deadline], không đếm số lần
/// tick: máy ngủ hay Timer bị trễ thì lần phát kế tiếp vẫn đúng.
Stream<Duration> remainingUntil(
  DateTime deadline, {
  Duration tick = const Duration(seconds: 1),
}) {
  late final StreamController<Duration> controller;
  Timer? timer;

  void emit() {
    final remaining = deadline.difference(clock.now());
    if (remaining <= Duration.zero) {
      timer?.cancel();
      controller.add(Duration.zero);
      controller.close();
    } else {
      controller.add(remaining);
    }
  }

  controller = StreamController<Duration>(
    onListen: () {
      emit();
      if (!controller.isClosed) {
        timer = Timer.periodic(tick, (_) => emit());
      }
    },
    onCancel: () => timer?.cancel(),
  );
  return controller.stream;
}

/// Đếm ngược tới một hạn, dùng chung cho hạn báo giá và hạn nhận cuốc.
final countdownProvider = StreamProvider.autoDispose.family<Duration, DateTime>(
  (ref, deadline) => remainingUntil(deadline),
);

/// "0:15" cho 15 giây; làm tròn lên giây để "0:00" chỉ hiện khi đã hết hạn.
String formatRemaining(Duration remaining) {
  final seconds = (remaining.inMilliseconds / 1000).ceil();
  final minutes = seconds ~/ 60;
  final rest = (seconds % 60).toString().padLeft(2, '0');
  return '$minutes:$rest';
}
