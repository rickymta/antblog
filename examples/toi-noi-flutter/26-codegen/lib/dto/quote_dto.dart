import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:toi_noi_core/toi_noi_core.dart';

import 'converters.dart';
import 'decode.dart';

part 'quote_dto.freezed.dart';
part 'quote_dto.g.dart';

/// Kết quả báo giá của server, viết lại bằng `freezed` + `json_serializable`.
///
/// Cùng hợp đồng JSON với `QuoteDto` viết tay của `toi_noi_api`: bốn khoá
/// `serviceKind`, `distanceMeters`, `fare`, `currency`.
@freezed
abstract class QuoteDto with _$QuoteDto {
  const QuoteDto._();

  const factory QuoteDto({
    @ServiceKindConverter() required ServiceKind serviceKind,

    /// Quãng đường theo mét nguyên: dạng truyền và lưu.
    @NonNegativeIntConverter() required int distanceMeters,

    /// Cước theo số đồng. Khoá JSON là `fare`.
    @JsonKey(name: 'fare') @WholeVndConverter() required int fareVnd,
    @VndCurrencyConverter() required String currency,
  }) = _QuoteDto;

  factory QuoteDto.fromJson(Object? json) =>
      decodeDto('QuoteDto', json, _$QuoteDtoFromJson);

  /// Cước dưới dạng [Money] của domain.
  Money toMoney() => Money.fromVnd(fareVnd);
}
