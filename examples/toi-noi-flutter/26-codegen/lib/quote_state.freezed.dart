// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'quote_state.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$QuoteInput {

 ServiceKind get serviceKind; double get distanceKm;
/// Create a copy of QuoteInput
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuoteInputCopyWith<QuoteInput> get copyWith => _$QuoteInputCopyWithImpl<QuoteInput>(this as QuoteInput, _$identity);



@override
bool operator ==(Object other) {
  final _this = this as QuoteInput;
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteInput&&(identical(other.serviceKind, _this.serviceKind) || other.serviceKind == _this.serviceKind)&&(identical(other.distanceKm, _this.distanceKm) || other.distanceKm == _this.distanceKm));
}


@override
int get hashCode {
  final _this = this as QuoteInput;
  return Object.hash(runtimeType,_this.serviceKind,_this.distanceKm);
}

@override
String toString() {
  final _this = this as QuoteInput;
  return 'QuoteInput(serviceKind: ${_this.serviceKind}, distanceKm: ${_this.distanceKm})';
}


}

/// @nodoc
abstract mixin class $QuoteInputCopyWith<$Res>  {
  factory $QuoteInputCopyWith(QuoteInput value, $Res Function(QuoteInput) _then) = _$QuoteInputCopyWithImpl;
@useResult
$Res call({
 ServiceKind serviceKind, double distanceKm
});




}
/// @nodoc
class _$QuoteInputCopyWithImpl<$Res>
    implements $QuoteInputCopyWith<$Res> {
  _$QuoteInputCopyWithImpl(this._self, this._then);

  final QuoteInput _self;
  final $Res Function(QuoteInput) _then;

/// Create a copy of QuoteInput
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? serviceKind = null,Object? distanceKm = null,}) {
  return _then(QuoteInput(
serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,distanceKm: null == distanceKm ? _self.distanceKm : distanceKm // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

}


/// Adds pattern-matching-related methods to [QuoteInput].
extension QuoteInputPatterns on QuoteInput {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _QuoteInput value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _QuoteInput() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _QuoteInput value)  $default,){
final _that = this;
switch (_that) {
case _QuoteInput():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _QuoteInput value)?  $default,){
final _that = this;
switch (_that) {
case _QuoteInput() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ServiceKind serviceKind,  double distanceKm)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _QuoteInput() when $default != null:
return $default(_that.serviceKind,_that.distanceKm);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ServiceKind serviceKind,  double distanceKm)  $default,) {final _that = this;
switch (_that) {
case _QuoteInput():
return $default(_that.serviceKind,_that.distanceKm);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ServiceKind serviceKind,  double distanceKm)?  $default,) {final _that = this;
switch (_that) {
case _QuoteInput() when $default != null:
return $default(_that.serviceKind,_that.distanceKm);case _:
  return null;

}
}

}

/// @nodoc


class _QuoteInput implements QuoteInput {
  const _QuoteInput({required this.serviceKind, required this.distanceKm});
  

@override final  ServiceKind serviceKind;
@override final  double distanceKm;

/// Create a copy of QuoteInput
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$QuoteInputCopyWith<_QuoteInput> get copyWith => __$QuoteInputCopyWithImpl<_QuoteInput>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is _QuoteInput&&(identical(other.serviceKind, serviceKind) || other.serviceKind == serviceKind)&&(identical(other.distanceKm, distanceKm) || other.distanceKm == distanceKm));
}


@override
int get hashCode {
    return Object.hash(runtimeType,serviceKind,distanceKm);
}

@override
String toString() {
    return 'QuoteInput(serviceKind: $serviceKind, distanceKm: $distanceKm)';
}


}

/// @nodoc
abstract mixin class _$QuoteInputCopyWith<$Res> implements $QuoteInputCopyWith<$Res> {
  factory _$QuoteInputCopyWith(_QuoteInput value, $Res Function(_QuoteInput) _then) = __$QuoteInputCopyWithImpl;
@override @useResult
$Res call({
 ServiceKind serviceKind, double distanceKm
});




}
/// @nodoc
class __$QuoteInputCopyWithImpl<$Res>
    implements _$QuoteInputCopyWith<$Res> {
  __$QuoteInputCopyWithImpl(this._self, this._then);

  final _QuoteInput _self;
  final $Res Function(_QuoteInput) _then;

/// Create a copy of QuoteInput
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? serviceKind = null,Object? distanceKm = null,}) {
  return _then(_QuoteInput(
serviceKind: null == serviceKind ? _self.serviceKind : serviceKind // ignore: cast_nullable_to_non_nullable
as ServiceKind,distanceKm: null == distanceKm ? _self.distanceKm : distanceKm // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc
mixin _$QuoteScreenState {





@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteScreenState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'QuoteScreenState()';
}


}

/// @nodoc
class $QuoteScreenStateCopyWith<$Res>  {
$QuoteScreenStateCopyWith(QuoteScreenState _, $Res Function(QuoteScreenState) __);
}


/// Adds pattern-matching-related methods to [QuoteScreenState].
extension QuoteScreenStatePatterns on QuoteScreenState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( QuoteIdle value)?  idle,TResult Function( QuoteLoading value)?  loading,TResult Function( QuoteReady value)?  ready,TResult Function( QuoteFailed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case QuoteIdle() when idle != null:
return idle(_that);case QuoteLoading() when loading != null:
return loading(_that);case QuoteReady() when ready != null:
return ready(_that);case QuoteFailed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( QuoteIdle value)  idle,required TResult Function( QuoteLoading value)  loading,required TResult Function( QuoteReady value)  ready,required TResult Function( QuoteFailed value)  failed,}){
final _that = this;
switch (_that) {
case QuoteIdle():
return idle(_that);case QuoteLoading():
return loading(_that);case QuoteReady():
return ready(_that);case QuoteFailed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( QuoteIdle value)?  idle,TResult? Function( QuoteLoading value)?  loading,TResult? Function( QuoteReady value)?  ready,TResult? Function( QuoteFailed value)?  failed,}){
final _that = this;
switch (_that) {
case QuoteIdle() when idle != null:
return idle(_that);case QuoteLoading() when loading != null:
return loading(_that);case QuoteReady() when ready != null:
return ready(_that);case QuoteFailed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  idle,TResult Function( QuoteInput input)?  loading,TResult Function( QuoteInput input,  Money fare)?  ready,TResult Function( QuoteInput input,  ApiFailure failure)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case QuoteIdle() when idle != null:
return idle();case QuoteLoading() when loading != null:
return loading(_that.input);case QuoteReady() when ready != null:
return ready(_that.input,_that.fare);case QuoteFailed() when failed != null:
return failed(_that.input,_that.failure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  idle,required TResult Function( QuoteInput input)  loading,required TResult Function( QuoteInput input,  Money fare)  ready,required TResult Function( QuoteInput input,  ApiFailure failure)  failed,}) {final _that = this;
switch (_that) {
case QuoteIdle():
return idle();case QuoteLoading():
return loading(_that.input);case QuoteReady():
return ready(_that.input,_that.fare);case QuoteFailed():
return failed(_that.input,_that.failure);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  idle,TResult? Function( QuoteInput input)?  loading,TResult? Function( QuoteInput input,  Money fare)?  ready,TResult? Function( QuoteInput input,  ApiFailure failure)?  failed,}) {final _that = this;
switch (_that) {
case QuoteIdle() when idle != null:
return idle();case QuoteLoading() when loading != null:
return loading(_that.input);case QuoteReady() when ready != null:
return ready(_that.input,_that.fare);case QuoteFailed() when failed != null:
return failed(_that.input,_that.failure);case _:
  return null;

}
}

}

/// @nodoc


class QuoteIdle implements QuoteScreenState {
  const QuoteIdle();
  






@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteIdle);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
    return 'QuoteScreenState.idle()';
}


}




/// @nodoc


class QuoteLoading implements QuoteScreenState {
  const QuoteLoading({required this.input});
  

 final  QuoteInput input;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuoteLoadingCopyWith<QuoteLoading> get copyWith => _$QuoteLoadingCopyWithImpl<QuoteLoading>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteLoading&&(identical(other.input, input) || other.input == input));
}


@override
int get hashCode {
    return Object.hash(runtimeType,input);
}

@override
String toString() {
    return 'QuoteScreenState.loading(input: $input)';
}


}

/// @nodoc
abstract mixin class $QuoteLoadingCopyWith<$Res> implements $QuoteScreenStateCopyWith<$Res> {
  factory $QuoteLoadingCopyWith(QuoteLoading value, $Res Function(QuoteLoading) _then) = _$QuoteLoadingCopyWithImpl;
@useResult
$Res call({
 QuoteInput input
});


$QuoteInputCopyWith<$Res> get input;

}
/// @nodoc
class _$QuoteLoadingCopyWithImpl<$Res>
    implements $QuoteLoadingCopyWith<$Res> {
  _$QuoteLoadingCopyWithImpl(this._self, this._then);

  final QuoteLoading _self;
  final $Res Function(QuoteLoading) _then;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? input = null,}) {
  return _then(QuoteLoading(
input: null == input ? _self.input : input // ignore: cast_nullable_to_non_nullable
as QuoteInput,
  ));
}

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$QuoteInputCopyWith<$Res> get input {
  
  return $QuoteInputCopyWith<$Res>(_self.input, (value) {
    return _then(_self.copyWith(input: value));
  });
}
}

/// @nodoc


class QuoteReady implements QuoteScreenState {
  const QuoteReady({required this.input, required this.fare});
  

 final  QuoteInput input;
 final  Money fare;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuoteReadyCopyWith<QuoteReady> get copyWith => _$QuoteReadyCopyWithImpl<QuoteReady>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteReady&&(identical(other.input, input) || other.input == input)&&(identical(other.fare, fare) || other.fare == fare));
}


@override
int get hashCode {
    return Object.hash(runtimeType,input,fare);
}

@override
String toString() {
    return 'QuoteScreenState.ready(input: $input, fare: $fare)';
}


}

/// @nodoc
abstract mixin class $QuoteReadyCopyWith<$Res> implements $QuoteScreenStateCopyWith<$Res> {
  factory $QuoteReadyCopyWith(QuoteReady value, $Res Function(QuoteReady) _then) = _$QuoteReadyCopyWithImpl;
@useResult
$Res call({
 QuoteInput input, Money fare
});


$QuoteInputCopyWith<$Res> get input;

}
/// @nodoc
class _$QuoteReadyCopyWithImpl<$Res>
    implements $QuoteReadyCopyWith<$Res> {
  _$QuoteReadyCopyWithImpl(this._self, this._then);

  final QuoteReady _self;
  final $Res Function(QuoteReady) _then;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? input = null,Object? fare = null,}) {
  return _then(QuoteReady(
input: null == input ? _self.input : input // ignore: cast_nullable_to_non_nullable
as QuoteInput,fare: null == fare ? _self.fare : fare // ignore: cast_nullable_to_non_nullable
as Money,
  ));
}

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$QuoteInputCopyWith<$Res> get input {
  
  return $QuoteInputCopyWith<$Res>(_self.input, (value) {
    return _then(_self.copyWith(input: value));
  });
}
}

/// @nodoc


class QuoteFailed implements QuoteScreenState {
  const QuoteFailed({required this.input, required this.failure});
  

 final  QuoteInput input;
 final  ApiFailure failure;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuoteFailedCopyWith<QuoteFailed> get copyWith => _$QuoteFailedCopyWithImpl<QuoteFailed>(this, _$identity);



@override
bool operator ==(Object other) {
    return identical(this, other) || (other.runtimeType == runtimeType&&other is QuoteFailed&&(identical(other.input, input) || other.input == input)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode {
    return Object.hash(runtimeType,input,failure);
}

@override
String toString() {
    return 'QuoteScreenState.failed(input: $input, failure: $failure)';
}


}

/// @nodoc
abstract mixin class $QuoteFailedCopyWith<$Res> implements $QuoteScreenStateCopyWith<$Res> {
  factory $QuoteFailedCopyWith(QuoteFailed value, $Res Function(QuoteFailed) _then) = _$QuoteFailedCopyWithImpl;
@useResult
$Res call({
 QuoteInput input, ApiFailure failure
});


$QuoteInputCopyWith<$Res> get input;

}
/// @nodoc
class _$QuoteFailedCopyWithImpl<$Res>
    implements $QuoteFailedCopyWith<$Res> {
  _$QuoteFailedCopyWithImpl(this._self, this._then);

  final QuoteFailed _self;
  final $Res Function(QuoteFailed) _then;

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? input = null,Object? failure = null,}) {
  return _then(QuoteFailed(
input: null == input ? _self.input : input // ignore: cast_nullable_to_non_nullable
as QuoteInput,failure: null == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as ApiFailure,
  ));
}

/// Create a copy of QuoteScreenState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$QuoteInputCopyWith<$Res> get input {
  
  return $QuoteInputCopyWith<$Res>(_self.input, (value) {
    return _then(_self.copyWith(input: value));
  });
}
}

// dart format on
