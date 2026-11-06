import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'build_log.dart';

/// Thẻ báo giá. Mỗi lần build ghi nhãn `quote-card`.
class QuoteCard extends StatelessWidget {
  /// Tạo thẻ báo giá cho [fare].
  const QuoteCard({required this.fare, super.key});

  /// Cước hiển thị.
  final Money fare;

  @override
  Widget build(BuildContext context) {
    BuildLog.hit('quote-card');
    return Card(
      child: ListTile(
        title: MoneyText(fare),
        subtitle: const Text('Giá tạm tính'),
      ),
    );
  }
}

/// Chữ ETA, tự nghe [ticker]: chỉ nó rebuild mỗi nhịp.
class EtaText extends StatelessWidget {
  /// Tạo chữ ETA nghe [ticker] (số giây còn lại).
  const EtaText(this.ticker, {super.key});

  /// Số giây còn lại.
  final ValueNotifier<int> ticker;

  @override
  Widget build(BuildContext context) => ValueListenableBuilder<int>(
    valueListenable: ticker,
    builder: (context, seconds, _) {
      BuildLog.hit('eta');
      return Text('Tài xế tới sau $seconds giây');
    },
  );
}

/// Cách 1 (chưa tối ưu): `setState` ở gốc màn hình mỗi nhịp.
///
/// Thẻ báo giá được dựng bằng giá trị chạy lúc runtime nên không `const`
/// được; footer thì `const` và không bị gọi lại `build`.
class NaiveTripScreen extends StatefulWidget {
  /// Tạo màn hình; [ticker] đổi giá trị mỗi nhịp đếm ngược.
  const NaiveTripScreen({required this.ticker, required this.fare, super.key});

  /// Nguồn nhịp đếm ngược.
  final ValueNotifier<int> ticker;

  /// Cước của chuyến.
  final Money fare;

  @override
  State<NaiveTripScreen> createState() => _NaiveTripScreenState();
}

class _NaiveTripScreenState extends State<NaiveTripScreen> {
  @override
  void initState() {
    super.initState();
    widget.ticker.addListener(_onTick);
  }

  @override
  void dispose() {
    widget.ticker.removeListener(_onTick);
    super.dispose();
  }

  void _onTick() => setState(() {});

  @override
  Widget build(BuildContext context) {
    BuildLog.hit('screen');
    return Column(
      children: [
        Text('Tài xế tới sau ${widget.ticker.value} giây'),
        QuoteCard(fare: widget.fare), // widget mới mỗi lần build
        const BuildProbe('const-footer'), // cùng một instance mỗi lần
        BuildProbe('plain-footer'), // ignore: prefer_const_constructors
      ],
    );
  }
}

/// Cách 2: tách phần đổi theo nhịp thành widget riêng ([EtaText]).
///
/// Màn hình là `StatelessWidget` không nghe gì, nên nó và thẻ báo giá không
/// build lại.
class SplitTripScreen extends StatelessWidget {
  /// Tạo màn hình đã tách.
  const SplitTripScreen({required this.ticker, required this.fare, super.key});

  /// Nguồn nhịp đếm ngược.
  final ValueNotifier<int> ticker;

  /// Cước của chuyến.
  final Money fare;

  @override
  Widget build(BuildContext context) {
    BuildLog.hit('screen');
    return Column(
      children: [
        EtaText(ticker),
        QuoteCard(fare: fare),
      ],
    );
  }
}

/// Cách 3: giữ `setState`/builder ở gốc nhưng truyền phần tĩnh qua tham số
/// `child` của builder, để framework gặp lại đúng instance cũ và dừng rebuild.
class ChildParamTripScreen extends StatelessWidget {
  /// Tạo màn hình dùng `child` của builder.
  const ChildParamTripScreen({
    required this.ticker,
    required this.fare,
    super.key,
  });

  /// Nguồn nhịp đếm ngược.
  final ValueNotifier<int> ticker;

  /// Cước của chuyến.
  final Money fare;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: ticker,
    // Dựng một lần trong build của màn hình, không dựng trong builder.
    child: QuoteCard(fare: fare),
    builder: (context, child) {
      BuildLog.hit('builder');
      return Column(children: [Text('Còn ${ticker.value} giây'), child!]);
    },
  );
}
