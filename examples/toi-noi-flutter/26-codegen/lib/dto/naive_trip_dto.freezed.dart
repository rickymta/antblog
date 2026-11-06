// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'naive_trip_dto.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$NaiveTripDto {

 String get id; ServiceKind get serviceKind; TripStatus get status; int get distanceMeters;@JsonKey(name: 'fare') int get fareVnd; String get currency; String? get driverId; DateTime get requestedAt; int get version;
/// Create a copy of NaiveTripDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NaiveTripDtoCopyWith<NaiveTripDto> get copyWith => _$NaiveTripDtoCopyWithImpl<NaiveTripDto>(this as NaiveTripDto, _$identity);

  /// Serializes this NaiveTripDto to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  final _this = this as NaiveTripDto;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NaiveTripDto&&(identical(other.id, _this.id) || other.id == _this.id)&&(identical(other.serviceKind, _this.serviceKind) || other.serviceKind == _this.serviceKind)&&(identical(other.status, _this.status) || other.status == _this.status)&&(identical(other.distanceMeters, _this.distanceMeters) || other.distanceMeters == _this.distanceMeters)&&(identical(other.fareVnd, _this.fareVnd) || other.fareVnd == _this.fareVnd)&&(identical(other.currency, _this.currency) || other.currency == _this.currency)&&(identical(other.driverId, _this.driverId) || other.driverId == _this.driverId)&&(identical(other.requestedAt, _this.requestedAt) || other.requestedAt == _this.requestedAt)&&(identical(other.version, _this.version) || other.version == _this.version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
  final _this = this as NaiveTripDto;
  return Object.hash(runtimeType,_this.id,_this.serviceKind,_this.status,_this.distanceMeters,_this.fareVnd,_this.currency,_this.driverId,_this.requestedAt,_this.version);
}

@override
String toString() {
  final _this = this as NaiveTripDto;
  return 'NaiveTripDto(id: ${_this.id}, serviceKind: ${_this.serviceKind}, status: ${_this.status}, distanceMeters: ${_this.distanceMeters}, fareVnd: ${_this.fareVnd}, currency: ${_this.currency}, driverId: ${_this.driverId}, requestedAt: ${_this.requestedAt}, version: ${_this.version})';
}


}

/// @nodoc
abstract mixin class $NaiveTripDtoCopyWith<$Res>  {
  factory $NaiveTripDtoCopyWith(NaiveTripDto value, $Res Function(NaiveTripDto) _then) = _$NaiveTripDtoCopyWithImpl;
@useResult
$Res call({
 String id, ServiceKind serviceKind, TripStatus status, int distanceMeters,@JsonKey(name: 'fare') int fareVnd, String currency, String? driverId, DateTime requestedAt, int version
});




}
/// @nodoc
class _$NaiveTripDtoCopyWithImpl<$Res>
    implements $NaiveTripDtoCopyWith<$Res> {
  _$NaiveTripDtoCopyWithImpl(this._self, this._then);

  final NaiveTripDto _self;
  final $Res Function(NaiveTripDto) _then;

/// Create a copy of NaiveTripDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? serviceKind = null,Object? status = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,Object? driverId = freezed,Object? requestedAt = null,Object? version = null,}) {
  return _then(NaiveTripDto(
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


/// Adds pattern-matching-related methods to [NaiveTripDto].
extension NaiveTripDtoPatterns on NaiveTripDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NaiveTripDto value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NaiveTripDto() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NaiveTripDto value)  $default,){
final _that = this;
switch (_that) {
case _NaiveTripDto():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NaiveTripDto value)?  $default,){
final _that = this;
switch (_that) {
case _NaiveTripDto() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  ServiceKind serviceKind,  TripStatus status,  int distanceMeters, @JsonKey(name: 'fare')  int fareVnd,  String currency,  String? driverId,  DateTime requestedAt,  int version)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NaiveTripDto() when $default != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  ServiceKind serviceKind,  TripStatus status,  int distanceMeters, @JsonKey(name: 'fare')  int fareVnd,  String currency,  String? driverId,  DateTime requestedAt,  int version)  $default,) {final _that = this;
switch (_that) {
case _NaiveTripDto():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  ServiceKind serviceKind,  TripStatus status,  int distanceMeters, @JsonKey(name: 'fare')  int fareVnd,  String currency,  String? driverId,  DateTime requestedAt,  int version)?  $default,) {final _that = this;
switch (_that) {
case _NaiveTripDto() when $default != null:
return $default(_that.id,_that.serviceKind,_that.status,_that.distanceMeters,_that.fareVnd,_that.currency,_that.driverId,_that.requestedAt,_that.version);case _:
  return null;

}
}

}

/// @nodoc

@JsonSerializable(checked: false)
class _NaiveTripDto implements NaiveTripDto {
  const _NaiveTripDto({required this.id, required this.serviceKind, required this.status, required this.distanceMeters, @JsonKey(name: 'fare') required this.fareVnd, required this.currency, this.driverId, required this.requestedAt, required this.version});
  factory _NaiveTripDto.fromJson(Map<String, dynamic> json) => _$NaiveTripDtoFromJson(json);

@override final  String id;
@override final  ServiceKind serviceKind;
@override final  TripStatus status;
@override final  int distanceMeters;
@override@JsonKey(name: 'fare') final  int fareVnd;
@override final  String currency;
@override final  String? driverId;
@override final  DateTime requestedAt;
@override final  int version;

/// Create a copy of NaiveTripDto
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NaiveTripDtoCopyWith<_NaiveTripDto> get copyWith => __$NaiveTripDtoCopyWithImpl<_NaiveTripDto>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$NaiveTripDtoToJson(this, );
}

@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is _NaiveTripDto&&(identical(other.id, id) || other.id == id)&&(identical(other.serviceKind, serviceKind) || other.serviceKind == serviceKind)&&(identical(other.status, status) || other.status == status)&&(identical(other.distanceMeters, distanceMeters) || other.distanceMeters == distanceMeters)&&(identical(other.fareVnd, fareVnd) || other.fareVnd == fareVnd)&&(identical(other.currency, currency) || other.currency == currency)&&(identical(other.driverId, driverId) || other.driverId == driverId)&&(identical(other.requestedAt, requestedAt) || other.requestedAt == requestedAt)&&(identical(other.version, version) || other.version == version));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
    return Object.hash(runtimeType,id,serviceKind,status,distanceMeters,fareVnd,currency,driverId,requestedAt,version);
}

@override
String toString() {
    return 'NaiveTripDto(id: $id, serviceKind: $serviceKind, status: $status, distanceMeters: $distanceMeters, fareVnd: $fareVnd, currency: $currency, driverId: $driverId, requestedAt: $requestedAt, version: $version)';
}


}

/// @nodoc
abstract mixin class _$NaiveTripDtoCopyWith<$Res> implements $NaiveTripDtoCopyWith<$Res> {
  factory _$NaiveTripDtoCopyWith(_NaiveTripDto value, $Res Function(_NaiveTripDto) _then) = __$NaiveTripDtoCopyWithImpl;
@override @useResult
$Res call({
 String id, ServiceKind serviceKind, TripStatus status, int distanceMeters,@JsonKey(name: 'fare') int fareVnd, String currency, String? driverId, DateTime requestedAt, int version
});




}
/// @nodoc
class __$NaiveTripDtoCopyWithImpl<$Res>
    implements _$NaiveTripDtoCopyWith<$Res> {
  __$NaiveTripDtoCopyWithImpl(this._self, this._then);

  final _NaiveTripDto _self;
  final $Res Function(_NaiveTripDto) _then;

/// Create a copy of NaiveTripDto
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? serviceKind = null,Object? status = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,Object? driverId = freezed,Object? requestedAt = null,Object? version = null,}) {
  return _then(_NaiveTripDto(
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
