import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:toi_noi_api/toi_noi_api.dart' show ApiFailure;
import 'package:toi_noi_core/toi_noi_core.dart';

part 'quote_state.freezed.dart';

/// Thứ khách yêu cầu báo giá: dịch vụ và quãng đường theo km.
@freezed
abstract class QuoteInput with _$QuoteInput {
  const factory QuoteInput({
    required ServiceKind serviceKind,
    required double distanceKm,
  }) = _QuoteInput;
}

/// Trạng thái của màn hình báo giá: bốn khả năng loại trừ nhau.
///
/// `sealed` nên `switch` trên giá trị này bắt buộc xử lý đủ bốn nhánh; thêm
/// nhánh thứ năm là mọi `switch` chưa xử lý nó thành lỗi biên dịch. Mỗi nhánh
/// chỉ mang dữ liệu có nghĩa ở nhánh đó: không có `fare` khi đang tải, không
/// có `failure` khi đã có giá.
@freezed
sealed class QuoteScreenState with _$QuoteScreenState {
  /// Chưa hỏi gì.
  const factory QuoteScreenState.idle() = QuoteIdle;

  /// Đang chờ server.
  const factory QuoteScreenState.loading({required QuoteInput input}) =
      QuoteLoading;

  /// Đã có giá.
  const factory QuoteScreenState.ready({
    required QuoteInput input,
    required Money fare,
  }) = QuoteReady;

  /// Thất bại, đã phân loại.
  const factory QuoteScreenState.failed({
    required QuoteInput input,
    required ApiFailure failure,
  }) = QuoteFailed;
}
