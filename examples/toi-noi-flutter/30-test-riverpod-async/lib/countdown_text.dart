import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'countdown.dart';

/// Chữ đếm ngược tới [deadline]: "0:15", "0:14", ... rồi "0:00".
class CountdownText extends ConsumerWidget {
  const CountdownText({super.key, required this.deadline});

  final DateTime deadline;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final remaining = ref.watch(countdownProvider(deadline));
    return Text(
      remaining.hasValue ? formatRemaining(remaining.requireValue) : '--:--',
      textDirection: TextDirection.ltr,
    );
  }
}
