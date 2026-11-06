// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'trip_dto.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_TripDto _$TripDtoFromJson(Map<String, dynamic> json) =>
    $checkedCreate('_TripDto', json, ($checkedConvert) {
      final val = _TripDto(
        id: $checkedConvert('id', (v) => v as String),
        serviceKind: $checkedConvert(
          'serviceKind',
          (v) => const ServiceKindConverter().fromJson(v),
        ),
        status: $checkedConvert(
          'status',
          (v) => const TripStatusConverter().fromJson(v),
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
        driverId: $checkedConvert('driverId', (v) => v as String?),
        requestedAt: $checkedConvert(
          'requestedAt',
          (v) => const UtcInstantConverter().fromJson(v),
        ),
        version: $checkedConvert(
          'version',
          (v) => const NonNegativeIntConverter().fromJson(v),
        ),
      );
      return val;
    }, fieldKeyMap: const {'fareVnd': 'fare'});

Map<String, dynamic> _$TripDtoToJson(_TripDto instance) => <String, dynamic>{
  'id': instance.id,
  'serviceKind': const ServiceKindConverter().toJson(instance.serviceKind),
  'status': const TripStatusConverter().toJson(instance.status),
  'distanceMeters': const NonNegativeIntConverter().toJson(
    instance.distanceMeters,
  ),
  'fare': const WholeVndConverter().toJson(instance.fareVnd),
  'currency': const VndCurrencyConverter().toJson(instance.currency),
  'driverId': instance.driverId,
  'requestedAt': const UtcInstantConverter().toJson(instance.requestedAt),
  'version': const NonNegativeIntConverter().toJson(instance.version),
};
