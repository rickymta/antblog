// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'quote_dto.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_QuoteDto _$QuoteDtoFromJson(Map<String, dynamic> json) =>
    $checkedCreate('_QuoteDto', json, ($checkedConvert) {
      final val = _QuoteDto(
        serviceKind: $checkedConvert(
          'serviceKind',
          (v) => const ServiceKindConverter().fromJson(v),
        ),
        distanceMeters: $checkedConvert(
          'distanceMeters',
          (v) => const NonNegativeIntConverter().fromJson(v),
        ),
        fareVnd: $checkedConvert(
          'fare',
          (v) => const WholeVndConverter().fromJson(v),
        ),
        currency: $checkedConvert(
          'currency',
          (v) => const VndCurrencyConverter().fromJson(v),
        ),
      );
      return val;
    }, fieldKeyMap: const {'fareVnd': 'fare'});

Map<String, dynamic> _$QuoteDtoToJson(_QuoteDto instance) => <String, dynamic>{
  'serviceKind': const ServiceKindConverter().toJson(instance.serviceKind),
  'distanceMeters': const NonNegativeIntConverter().toJson(
    instance.distanceMeters,
  ),
  'fare': const WholeVndConverter().toJson(instance.fareVnd),
  'currency': const VndCurrencyConverter().toJson(instance.currency),
};
