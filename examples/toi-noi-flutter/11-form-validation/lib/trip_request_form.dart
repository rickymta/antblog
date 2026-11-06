import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'trip_request_data.dart';
import 'validators.dart';

/// Form nhập điểm đón, điểm đến và khoảng cách (nhập tay, tạm thay cho bản
/// đồ ở bài sau). Khi hợp lệ, gọi [onSubmit] với dữ liệu đã chuẩn hoá.
class TripRequestForm extends StatefulWidget {
  const TripRequestForm({super.key, required this.onSubmit});

  final ValueChanged<TripRequestData> onSubmit;

  @override
  State<TripRequestForm> createState() => _TripRequestFormState();
}

class _TripRequestFormState extends State<TripRequestForm> {
  final _formKey = GlobalKey<FormState>();

  final _pickupKey = GlobalKey<FormFieldState<String>>();
  final _destinationKey = GlobalKey<FormFieldState<String>>();
  final _distanceKey = GlobalKey<FormFieldState<String>>();

  final _pickupController = TextEditingController();
  final _destinationController = TextEditingController();
  final _distanceController = TextEditingController();

  final _pickupFocus = FocusNode();
  final _destinationFocus = FocusNode();
  final _distanceFocus = FocusNode();

  @override
  void dispose() {
    _pickupController.dispose();
    _destinationController.dispose();
    _distanceController.dispose();
    _pickupFocus.dispose();
    _destinationFocus.dispose();
    _distanceFocus.dispose();
    super.dispose();
  }

  void _submit() {
    if (!_formKey.currentState!.validate()) {
      _focusFirstInvalid();
      return;
    }
    // validate() vừa qua nên parseDistanceKm không thể trả null.
    final data = TripRequestData(
      pickup: normalizeAddress(_pickupController.text),
      destination: normalizeAddress(_destinationController.text),
      distanceKm: parseDistanceKm(_distanceController.text)!,
    );
    FocusScope.of(context).unfocus();
    widget.onSubmit(data);
  }

  void _focusFirstInvalid() {
    final fields = [
      (_pickupKey, _pickupFocus),
      (_destinationKey, _destinationFocus),
      (_distanceKey, _distanceFocus),
    ];
    for (final (key, focus) in fields) {
      if (key.currentState?.hasError ?? false) {
        focus.requestFocus();
        return;
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          TextFormField(
            key: _pickupKey,
            controller: _pickupController,
            focusNode: _pickupFocus,
            decoration: const InputDecoration(
              labelText: 'Điểm đón',
              prefixIcon: Icon(Icons.trip_origin),
            ),
            keyboardType: TextInputType.streetAddress,
            textInputAction: TextInputAction.next,
            textCapitalization: TextCapitalization.words,
            autovalidateMode: AutovalidateMode.onUserInteractionIfError,
            validator: validatePickup,
          ),
          const SizedBox(height: 16),
          TextFormField(
            key: _destinationKey,
            controller: _destinationController,
            focusNode: _destinationFocus,
            decoration: const InputDecoration(
              labelText: 'Điểm đến',
              prefixIcon: Icon(Icons.place),
            ),
            keyboardType: TextInputType.streetAddress,
            textInputAction: TextInputAction.next,
            textCapitalization: TextCapitalization.words,
            autovalidateMode: AutovalidateMode.onUserInteractionIfError,
            // Đọc điểm đón tại thời điểm kiểm, không chụp giá trị cũ.
            validator: (value) =>
                validateDestination(value, pickup: _pickupController.text),
          ),
          const SizedBox(height: 16),
          TextFormField(
            key: _distanceKey,
            controller: _distanceController,
            focusNode: _distanceFocus,
            decoration: const InputDecoration(
              labelText: 'Khoảng cách (km)',
              helperText: 'Tạm nhập tay; bài bản đồ sẽ tính tự động',
              prefixIcon: Icon(Icons.straighten),
            ),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            // Bộ lọc chỉ giúp gõ đỡ nhầm, không thay validator: dán vào vẫn
            // có thể ra "1,2,3".
            inputFormatters: [
              FilteringTextInputFormatter.allow(RegExp(r'[0-9.,]')),
            ],
            textInputAction: TextInputAction.done,
            autovalidateMode: AutovalidateMode.onUserInteractionIfError,
            validator: validateDistance,
            onFieldSubmitted: (_) => _submit(),
          ),
          const SizedBox(height: 24),
          FilledButton(onPressed: _submit, child: const Text('Tìm tài xế')),
        ],
      ),
    );
  }
}
