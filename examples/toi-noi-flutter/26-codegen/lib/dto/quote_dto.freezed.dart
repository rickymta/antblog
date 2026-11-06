// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'quote_dto.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$QuoteDto {

@ServiceKindConverter() ServiceKind get serviceKind;/// Quãng đường theo mét nguyên: dạng truyền và lưu.
@NonNegativeIntConverter() int get distanceMeters;/// Cước theo số đồng. Khoá JSON là `fare`.
@JsonKey(name: 'fare')@WholeVndConverter() int get fareVnd;@VndCurrencyConverter() String get currency;
/// Create a copy of QuoteDto
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuoteDtoCopyWith<QuoteDto> get copyWith => _$QuoteDtoCopyWithImpl<QuoteDto>(this as QuoteDto, _$identity);

  /// Serializes this QuoteDto to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  final _this = this as QuoteDto;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteDto&&(identical(other.serviceKind, _this.serviceKind) || other.serviceKind == _this.serviceKind)&&(identical(other.distanceMeters, _this.distanceMeters) || other.distanceMeters == _this.distanceMeters)&&(identical(other.fareVnd, _this.fareVnd) || other.fareVnd == _this.fareVnd)&&(identical(other.currency, _this.currency) || other.currency == _this.currency));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
  final _this = this as QuoteDto;
  return Object.hash(runtimeType,_this.serviceKind,_this.distanceMeters,_this.fareVnd,_this.currency);
}

@override
String toString() {
  final _this = this as QuoteDto;
  return 'QuoteDto(serviceKind: ${_this.serviceKind}, distanceMeters: ${_this.distanceMeters}, fareVnd: ${_this.fareVnd}, currency: ${_this.currency})';
}


}

/// @nodoc
abstract mixin class $QuoteDtoCopyWith<$Res>  {
  factory $QuoteDtoCopyWith(QuoteDto value, $Res Function(QuoteDto) _then) = _$QuoteDtoCopyWithImpl;
@useResult
$Res call({
@ServiceKindConverter() ServiceKind serviceKind,@NonNegativeIntConverter() int distanceMeters,@JsonKey(name: 'fare')@WholeVndConverter() int fareVnd,@VndCurrencyConverter() String currency
});




}
/// @nodoc
class _$QuoteDtoCopyWithImpl<$Res>
    implements $QuoteDtoCopyWith<$Res> {
  _$QuoteDtoCopyWithImpl(this._self, this._then);

  final QuoteDto _self;
  final $Res Function(QuoteDto) _then;

/// Create a copy of QuoteDto
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? serviceKind = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,}) {
  return _then(QuoteDto(
serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,distanceMeters: null == distanceMeters ? _self.distanceMeters : distanceMeters // ignore: cast_nullable_to_non_nullable
as int,fareVnd: null == fareVnd ? _self.fareVnd : fareVnd // ignore: cast_nullable_to_non_nullable
as int,currency: null == currency ? _self.currency : currency // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [QuoteDto].
extension QuoteDtoPatterns on QuoteDto {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _QuoteDto value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _QuoteDto() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _QuoteDto value)  $default,){
final _that = this;
switch (_that) {
case _QuoteDto():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _QuoteDto value)?  $default,){
final _that = this;
switch (_that) {
case _QuoteDto() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function(@ServiceKindConverter()  ServiceKind serviceKind, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _QuoteDto() when $default != null:
return $default(_that.serviceKind,_that.distanceMeters,_that.fareVnd,_that.currency);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function(@ServiceKindConverter()  ServiceKind serviceKind, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency)  $default,) {final _that = this;
switch (_that) {
case _QuoteDto():
return $default(_that.serviceKind,_that.distanceMeters,_that.fareVnd,_that.currency);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function(@ServiceKindConverter()  ServiceKind serviceKind, @NonNegativeIntConverter()  int distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter()  int fareVnd, @VndCurrencyConverter()  String currency)?  $default,) {final _that = this;
switch (_that) {
case _QuoteDto() when $default != null:
return $default(_that.serviceKind,_that.distanceMeters,_that.fareVnd,_that.currency);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _QuoteDto extends QuoteDto {
  const _QuoteDto({@ServiceKindConverter() required this.serviceKind, @NonNegativeIntConverter() required this.distanceMeters, @JsonKey(name: 'fare')@WholeVndConverter() required this.fareVnd, @VndCurrencyConverter() required this.currency}): super._();
  factory _QuoteDto.fromJson(Map<String, dynamic> json) => _$QuoteDtoFromJson(json);

@override@ServiceKindConverter() final  ServiceKind serviceKind;
/// Quãng đường theo mét nguyên: dạng truyền và lưu.
@override@NonNegativeIntConverter() final  int distanceMeters;
/// Cước theo số đồng. Khoá JSON là `fare`.
@override@JsonKey(name: 'fare')@WholeVndConverter() final  int fareVnd;
@override@VndCurrencyConverter() final  String currency;

/// Create a copy of QuoteDto
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$QuoteDtoCopyWith<_QuoteDto> get copyWith => __$QuoteDtoCopyWithImpl<_QuoteDto>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$QuoteDtoToJson(this, );
}

@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is _QuoteDto&&(identical(other.serviceKind, serviceKind) || other.serviceKind == serviceKind)&&(identical(other.distanceMeters, distanceMeters) || other.distanceMeters == distanceMeters)&&(identical(other.fareVnd, fareVnd) || other.fareVnd == fareVnd)&&(identical(other.currency, currency) || other.currency == currency));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode {
    return Object.hash(runtimeType,serviceKind,distanceMeters,fareVnd,currency);
}

@override
String toString() {
    return 'QuoteDto(serviceKind: $serviceKind, distanceMeters: $distanceMeters, fareVnd: $fareVnd, currency: $currency)';
}


}

/// @nodoc
abstract mixin class _$QuoteDtoCopyWith<$Res> implements $QuoteDtoCopyWith<$Res> {
  factory _$QuoteDtoCopyWith(_QuoteDto value, $Res Function(_QuoteDto) _then) = __$QuoteDtoCopyWithImpl;
@override @useResult
$Res call({
@ServiceKindConverter() ServiceKind serviceKind,@NonNegativeIntConverter() int distanceMeters,@JsonKey(name: 'fare')@WholeVndConverter() int fareVnd,@VndCurrencyConverter() String currency
});




}
/// @nodoc
class __$QuoteDtoCopyWithImpl<$Res>
    implements _$QuoteDtoCopyWith<$Res> {
  __$QuoteDtoCopyWithImpl(this._self, this._then);

  final _QuoteDto _self;
  final $Res Function(_QuoteDto) _then;

/// Create a copy of QuoteDto
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? serviceKind = null,Object? distanceMeters = null,Object? fareVnd = null,Object? currency = null,}) {
  return _then(_QuoteDto(
serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,distanceMeters: null == distanceMeters ? _self.distanceMeters : distanceMeters // ignore: cast_nullable_to_non_nullable
as int,fareVnd: null == fareVnd ? _self.fareVnd : fareVnd // ignore: cast_nullable_to_non_nullable
as int,currency: null == currency ? _self.currency : currency // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
