import 'package:flutter/material.dart';
import 'package:toi_noi_core/toi_noi_core.dart';
import 'package:toi_noi_ui/toi_noi_ui.dart';

import 'fare_readers.dart';
import 'quote_scope.dart';
import 'trip_book.dart';

/// Màn hình báo giá. `StatelessWidget`: state sống trong [QuoteDraft], không
/// còn `_kind` và `_km` trong `State` như ở bài 16.
class QuoteScreen extends StatelessWidget {
  const QuoteScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(ToiNoiSpacing.md),
      children: const [
        _ServiceChips(),
        SizedBox(height: ToiNoiSpacing.lg),
        _KmSlider(),
        SizedBox(height: ToiNoiSpacing.md),
        Card(
          child: Padding(
            padding: EdgeInsets.all(ToiNoiSpacing.md),
            child: FareFromValue(),
          ),
        ),
        SizedBox(height: ToiNoiSpacing.lg),
        _BookButton(),
      ],
    );
  }
}

class _ServiceChips extends StatelessWidget {
  const _ServiceChips();

  @override
  Widget build(BuildContext context) {
    // `of`: chip phải vẽ lại khi dịch vụ đổi.
    final draft = QuoteScope.of(context);
    return Wrap(
      spacing: ToiNoiSpacing.sm,
      children: [
        for (final kind in ServiceKind.values)
          ChoiceChip(
            label: Text(kind.label),
            selected: kind == draft.service,
            onSelected: (_) => draft.selectService(kind),
          ),
      ],
    );
  }
}

class _KmSlider extends StatelessWidget {
  const _KmSlider();

  @override
  Widget build(BuildContext context) {
    final draft = QuoteScope.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Quãng đường: ${draft.km.toStringAsFixed(0)} km'),
        Slider(
          value: draft.km,
          min: 1,
          max: 20,
          divisions: 19,
          semanticFormatterCallback: (value) => '${value.round()} km',
          onChanged: draft.setKm,
        ),
      ],
    );
  }
}

class _BookButton extends StatelessWidget {
  const _BookButton();

  @override
  Widget build(BuildContext context) {
    return FilledButton(
      // `read`: chỉ lấy draft lúc bấm nút, nút không cần dựng lại khi draft đổi.
      onPressed: () {
        final draft = QuoteScope.read(context);
        final trip = TripBookScope.read(context).book(draft.service, draft.km);
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text('Đã gửi yêu cầu ${trip.id}')));
      },
      child: const Text('Đặt chuyến'),
    );
  }
}
