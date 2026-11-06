// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'trip_dto.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$TripDto {

 String get id;@ServiceKindConverter() ServiceKind get serviceKind;@TripStatusConverter() TripStatus get status;@NonNegativeIntConverter() int get distanceMeters;@JsonKey(name: 'fare')@WholeVndConverter() int get fareVnd;@VndCurrencyConverter() String get currency; String? get driverId;/// Thời điểm đặt, luôn UTC.
@UtcInstantConverter() DateTime get requestedAt;/// Số phiên bản của chuyến trên server, dùng làm `expectedVersion`.
@NonNegativeIntConverter() int get version;
/// Create a copy of TripDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TripDtoCopyWith<TripDto> get copyWith => _$TripDtoCopyWithImpl<TripDto>(this as TripDto, _$identity);

  /// Serializes this TripDto to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  final _this = this as TripDto;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TripDto&&(identical(other.id, _this.id) || other.id == _this.id)&&(identical(other.serviceKind, _this.serviceKind) || other.serviceKind == _this.serviceKind)&&(identical(other.status, _this.status) || other.status == _this.status)&&(identical(other.distanceMeters, _this.distanceMeters) || other.distanceMeters == _this.distanceMeters)&&(identical(other.fareVnd, _this.fareVnd) || other.fareVnd == _this.fareVnd)&&(identical(other.currency, _this.currency) || other.currency == _this.currency)&&(identical(other.driverId, _this.driverId) || other.driverId == _this.driverId)&&(identical(other.requestedAt, _this.requestedAt) || other.requestedAt == _this.requestedAt)&&(identical(other.version, _this.version) || other.version == _this.version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
  final _this = this as TripDto;
  return Object.hash(runtimeType,_this.id,_this.serviceKind,_this.status,_this.distanceMeters,_this.fareVnd,_this.currency,_this.driverId,_this.requestedAt,_this.version);
}

@override
String toString() {
  final _this = this as TripDto;
  return 'TripDto(id: ${_this.id}, serviceKind: ${_this.serviceKind}, status: ${_this.status}, distanceMeters: ${_this.distanceMeters}, fareVnd: ${_this.fareVnd}, currency: ${_this.currency}, driverId: ${_this.driverId}, requestedAt: ${_this.requestedAt}, version: ${_this.version})';
}


}

/// @nodoc
abstract mixin class $TripDtoCopyWith<$Res>  {
  factory $TripDtoCopyWith(TripDto value, $Res Function(TripDto) _then) = _$TripDtoCopyWithImpl;
@useResult
$Res call({
 String id,@ServiceKindConverter() ServiceKind serviceKind,@TripStatusConverter() TripStatus status,@NonNegativeIntConverter() int distanceMeters,@JsonKey(name: 'fare')@WholeVndConverter() int fareVnd,@VndCurrencyConverter() String currency, String? driverId,@UtcInstantConverter() DateTime requestedAt,@NonNegativeIntConverter() int version
});




}
/// @nodoc
class _$TripDtoCopyWithImpl<$Res>
    implements $TripDtoCopyWith<$Res> {
  _$TripDtoCopyWithImpl(this._self, this._then);

  final TripDto _self;
  final $Res Function(TripDto) _then;

/// Create a copy of TripDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? serviceKind = null,Object? status = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,Object? driverId = freezed,Object? requestedAt = null,Object? version = null,}) {
  return _then(TripDto(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as TripStatus,distanceMeters: null == distanceMeters ? _self.distanceMeters : distanceMeters // ignore: cast_nullable_to_non_nullable
as int,fareVnd: null == fareVnd ? _self.fareVnd : fareVnd // ignore: cast_nullable_to_non_nullable
as int,currency: null == currency ? _self.currency : currency // ignore: cast_nullable_to_non_nullable
as String,driverId: freezed == driverId ? _self.driverId : driverId // ignore: cast_nullable_to_non_nullable
as String?,requestedAt: null == requestedAt ? _self.requestedAt : requestedAt // ignore: cast_nullable_to_non_nullable
as DateTime,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [TripDto].
extension TripDtoPatterns on TripDto {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TripDto value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TripDto() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TripDto value)  $default,){
final _that = this;
switch (_that) {
case _TripDto():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TripDto value)?  $default,){
final _that = this;
switch (_that) {
case _TripDto() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id, @ServiceKindConverter()  ServiceKind serviceKind, @TripStatusConverter()  TripStatus status, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency,  String? driverId, @UtcInstantConverter()  DateTime requestedAt, @NonNegativeIntConverter()  int version)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TripDto() when $default != null:
return $default(_that.id,_that.serviceKind,_that.status,_that.distanceMeters,_that.fareVnd,_that.currency,_that.driverId,_that.requestedAt,_that.version);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id, @ServiceKindConverter()  ServiceKind serviceKind, @TripStatusConverter()  TripStatus status, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency,  String? driverId, @UtcInstantConverter()  DateTime requestedAt, @NonNegativeIntConverter()  int version)  $default,) {final _that = this;
switch (_that) {
case _TripDto():
return $default(_that.id,_that.serviceKind,_that.status,_that.distanceMeters,_that.fareVnd,_that.currency,_that.driverId,_that.requestedAt,_that.version);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id, @ServiceKindConverter()  ServiceKind serviceKind, @TripStatusConverter()  TripStatus status, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency,  String? driverId, @UtcInstantConverter()  DateTime requestedAt, @NonNegativeIntConverter()  int version)?  $default,) {final _that = this;
switch (_that) {
case _TripDto() when $default != null:
return $default(_that.id,_that.serviceKind,_that.status,_that.distanceMeters,_that.fareVnd,_that.currency,_that.driverId,_that.requestedAt,_that.version);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _TripDto extends TripDto {
  const _TripDto({required this.id, @ServiceKindConverter() required this.serviceKind, @TripStatusConverter() required this.status, @NonNegativeIntConverter() required this.distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter() required this.fareVnd, @VndCurrencyConverter() required this.currency, this.driverId, @UtcInstantConverter() required this.requestedAt, @NonNegativeIntConverter() required this.version}): super._();
  factory _TripDto.fromJson(Map<String, dynamic> json) => _$TripDtoFromJson(json);

@override final  String id;
@override@ServiceKindConverter() final  ServiceKind serviceKind;
@override@TripStatusConverter() final  TripStatus status;
@override@NonNegativeIntConverter() final  int distanceMeters;
@override@JsonKey(name: 'fare')@WholeVndConverter() final  int fareVnd;
@override@VndCurrencyConverter() final  String currency;
@override final  String? driverId;
/// Thời điểm đặt, luôn UTC.
@override@UtcInstantConverter() final  DateTime requestedAt;
/// Số phiên bản của chuyến trên server, dùng làm `expectedVersion`.
@override@NonNegativeIntConverter() final  int version;

/// Create a copy of TripDto
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TripDtoCopyWith<_TripDto> get copyWith => __$TripDtoCopyWithImpl<_TripDto>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$TripDtoToJson(this, );
}

@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is _TripDto&&(identical(other.id, id) || other.id == id)&&(identical(other.serviceKind, serviceKind) || other.serviceKind == serviceKind)&&(identical(other.status, status) || other.status == status)&&(identical(other.distanceMeters, distanceMeters) || other.distanceMeters == distanceMeters)&&(identical(other.fareVnd, fareVnd) || other.fareVnd == fareVnd)&&(identical(other.currency, currency) || other.currency == currency)&&(identical(other.driverId, driverId) || other.driverId == driverId)&&(identical(other.requestedAt, requestedAt) || other.requestedAt == requestedAt)&&(identical(other.version, version) || other.version == version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
    return Object.hash(runtimeType,id,serviceKind,status,distanceMeters,fareVnd,currency,driverId,requestedAt,version);
}

@override
String toString() {
    return 'TripDto(id: $id, serviceKind: $serviceKind, status: $status, distanceMeters: $distanceMeters, fareVnd: $fareVnd, currency: $currency, driverId: $driverId, requestedAt: $requestedAt, version: $version)';
}


}

/// @nodoc
abstract mixin class _$TripDtoCopyWith<$Res> implements $TripDtoCopyWith<$Res> {
  factory _$TripDtoCopyWith(_TripDto value, $Res Function(_TripDto) _then) = __$TripDtoCopyWithImpl;
@override @useResult
$Res call({
 String id,@ServiceKindConverter() ServiceKind serviceKind,@TripStatusConverter() TripStatus status,@NonNegativeIntConverter() int distanceMeters,@JsonKey(name: 'fare')@WholeVndConverter() int fareVnd,@VndCurrencyConverter() String currency, String? driverId,@UtcInstantConverter() DateTime requestedAt,@NonNegativeIntConverter() int version
});




}
/// @nodoc
class __$TripDtoCopyWithImpl<$Res>
    implements _$TripDtoCopyWith<$Res> {
  __$TripDtoCopyWithImpl(this._self, this._then);

  final _TripDto _self;
  final $Res Function(_TripDto) _then;

/// Create a copy of TripDto
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? serviceKind = null,Object? status = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,Object? driverId = freezed,Object? requestedAt = null,Object? version = null,}) {
  return _then(_TripDto(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as TripStatus,distanceMeters: null == distanceMeters ? _self.distanceMeters : distanceMeters // ignore: cast_nullable_to_non_nullable
as int,fareVnd: null == fareVnd ? _self.fareVnd : fareVnd // ignore: cast_nullable_to_non_nullable
as int,currency: null == currency ? _self.currency : currency // ignore: cast_nullable_to_non_nullable
as String,driverId: freezed == driverId ? _self.driverId : driverId // ignore: cast_nullable_to_non_nullable
as String?,requestedAt: null == requestedAt ? _self.requestedAt : requestedAt // ignore: cast_nullable_to_non_nullable
as DateTime,version: null == version ? _self.version : version // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on
