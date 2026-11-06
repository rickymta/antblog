import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'quote_draft.dart';
import 'quote_scope.dart';

/// Bộ đếm số lần build theo tên, để test đo ranh giới rebuild.
class BuildCounts {
  final Map<String, int> _counts = {};

  int operator [](String name) => _counts[name] ?? 0;

  void hit(String name) => _counts[name] = this[name] + 1;
}

/// Cách 1: `QuoteScope.of`. Đăng ký phụ thuộc vào *cả* draft, nên dựng lại
/// khi bất cứ thứ gì trong draft đổi, kể cả dịch vụ.
class FareFromScope extends StatelessWidget {
  const FareFromScope({super.key, this.counts});

  final BuildCounts? counts;

  @override
  Widget build(BuildContext context) {
    counts?.hit('scope');
    return MoneyText(QuoteScope.of(context).fare);
  }
}

/// Cách 2: `QuoteScope.read` rồi chỉ nghe `fareListenable`. Widget này không
/// phụ thuộc vào scope; chỉ phần `builder` chạy lại, và chỉ khi cước đổi.
/// Phần [caption] truyền qua `child` được dựng đúng một lần.
class FareFromValue extends StatelessWidget {
  const FareFromValue({super.key, this.counts});

  final BuildCounts? counts;

  @override
  Widget build(BuildContext context) {
    counts?.hit('value:widget');
    final draft = QuoteScope.read(context);
    return ValueListenableBuilder<Money>(
      valueListenable: draft.fareListenable,
      child: _Caption(counts),
      builder: (context, fare, caption) {
        counts?.hit('value');
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [caption!, MoneyText(fare)],
        );
      },
    );
  }
}

/// Cách 3: truyền *cả notifier* vào widget rồi `ListenableBuilder` trên nó.
/// Builder chạy lại mỗi lần draft báo, dù thứ đổi không liên quan tới cước.
class FareFromWholeNotifier extends StatelessWidget {
  const FareFromWholeNotifier({super.key, required this.draft, this.counts});

  final QuoteDraft draft;
  final BuildCounts? counts;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: draft,
      builder: (context, _) {
        counts?.hit('whole');
        return MoneyText(draft.fare);
      },
    );
  }
}

class _Caption extends StatelessWidget {
  const _Caption(this.counts);

  final BuildCounts? counts;

  @override
  Widget build(BuildContext context) {
    counts?.hit('caption');
    return Text('Cước dự kiến', style: Theme.of(context).textTheme.labelLarge);
  }
}
