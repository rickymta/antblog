/// Giả lập lỗi tầng mạng cho test: [FaultyAdapter] bọc một `HttpClientAdapter`
/// và chèn từ chối kết nối, quá hạn kết nối, server treo hoặc mất response
/// theo kịch bản. Không phụ thuộc vào adapter nào bên trong.
library;

export 'src/faulty_adapter.dart' show FaultyAdapter, NetworkFault;
