// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'naive_trip_dto.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_NaiveTripDto _$NaiveTripDtoFromJson(Map<String, dynamic> json) =>
    _NaiveTripDto(
      id: json['id'] as String,
      serviceKind: $enumDecode(_$ServiceKindEnumMap, json['serviceKind']),
      status: $enumDecode(_$TripStatusEnumMap, json['status']),
      distanceMeters: (json['distanceMeters'] as num).toInt(),
      fareVnd: (json['fare'] as num).toInt(),
      currency: json['currency'] as String,
      driverId: json['driverId'] as String?,
      requestedAt: DateTime.parse(json['requestedAt'] as String),
      version: (json['version'] as num).toInt(),
    );

Map<String, dynamic> _$NaiveTripDtoToJson(_NaiveTripDto instance) =>
    <String, dynamic>{
      'id': instance.id,
      'serviceKind': _$ServiceKindEnumMap[instance.serviceKind]!,
      'status': _$TripStatusEnumMap[instance.status]!,
      'distanceMeters': instance.distanceMeters,
      'fare': instance.fareVnd,
      'currency': instance.currency,
      'driverId': instance.driverId,
      'requestedAt': instance.requestedAt.toIso8601String(),
      'version': instance.version,
    };

const _$ServiceKindEnumMap = {
  ServiceKind.ride: 'ride',
  ServiceKind.food: 'food',
  ServiceKind.parcel: 'parcel',
};

const _$TripStatusEnumMap = {
  TripStatus.requested: 'requested',
  TripStatus.searching: 'searching',
  TripStatus.accepted: 'accepted',
  TripStatus.arriving: 'arriving',
  TripStatus.pickedUp: 'pickedUp',
  TripStatus.inTransit: 'inTransit',
  TripStatus.completed: 'completed',
  TripStatus.cancelled: 'cancelled',
};
