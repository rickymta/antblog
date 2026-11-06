/// Công cụ test cho code dùng `dio`: chạy một shelf `Handler` (thường là
/// `createFakeApiHandler()`) ngay trong tiến trình, không socket.
///
/// ```dart
/// final adapter = InProcessAdapter(createFakeApiHandler());
/// final client = ToiNoiApiClient(
///   baseUrl: Uri.parse('http://fake.test'),
///   adapter: adapter,
/// );
/// ```
library;

export 'src/in_process_adapter.dart' show InProcessAdapter, RecordedRequest;
