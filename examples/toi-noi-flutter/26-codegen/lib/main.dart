import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'providers.dart';
import 'quote_screen.dart';
import 'quote_source.dart';

/// Địa chỉ fake API của lab (`dart run bin/server.dart` trong `tools/fake_api`).
/// iOS Simulator gọi được `localhost`; Android emulator dùng `10.0.2.2`.
const _baseUrl = 'http://localhost:8787';

void main() {
  runApp(
    ProviderScope(
      overrides: [
        quoteSourceProvider.overrideWithValue(
          DioQuoteSource(Dio(BaseOptions(baseUrl: _baseUrl))),
        ),
      ],
      child: const MaterialApp(home: QuoteScreen()),
    ),
  );
}
