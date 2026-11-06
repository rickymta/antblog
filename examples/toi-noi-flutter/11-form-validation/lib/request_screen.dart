import 'package:flutter/material.dart';

import 'trip_request_data.dart';
import 'trip_request_form.dart';

/// Màn hình đặt xe: một [TripRequestForm] trong [SingleChildScrollView].
///
/// [resizeToAvoidBottomInset] chỉ để minh hoạ và test: đặt `false` thì
/// `Scaffold` không thu nhỏ body khi bàn phím hiện.
class RequestScreen extends StatelessWidget {
  const RequestScreen({super.key, this.resizeToAvoidBottomInset = true});

  final bool resizeToAvoidBottomInset;

  void _onSubmit(BuildContext context, TripRequestData data) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'Đã nhận: ${data.pickup} → ${data.destination}, '
          '${data.distanceKm} km, cước ước tính ${data.estimatedFare}',
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Đặt xe')),
      resizeToAvoidBottomInset: resizeToAvoidBottomInset,
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          keyboardDismissBehavior: ScrollViewKeyboardDismissBehavior.onDrag,
          child: TripRequestForm(onSubmit: (data) => _onSubmit(context, data)),
        ),
      ),
    );
  }
}
