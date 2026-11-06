// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'providers.dart';

// **************************************************************************
// RiverpodGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint, type=warning
/// Cổng tới backend. Cố ý không có giá trị mặc định: `main()` và test ghi đè
/// bằng `quoteSourceProvider.overrideWithValue(...)`.

@ProviderFor(quoteSource)
final quoteSourceProvider = QuoteSourceProvider._();

/// Cổng tới backend. Cố ý không có giá trị mặc định: `main()` và test ghi đè
/// bằng `quoteSourceProvider.overrideWithValue(...)`.

final class QuoteSourceProvider
    extends $FunctionalProvider<QuoteSource, QuoteSource, QuoteSource>
    with $Provider<QuoteSource> {
  /// Cổng tới backend. Cố ý không có giá trị mặc định: `main()` và test ghi đè
  /// bằng `quoteSourceProvider.overrideWithValue(...)`.
  QuoteSourceProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'quoteSourceProvider',
        isAutoDispose: false,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$quoteSourceHash();

  @$internal
  @override
  $ProviderElement<QuoteSource> $createElement($ProviderPointer pointer) =>
      $ProviderElement(pointer);

  @override
  QuoteSource create(Ref ref) {
    return quoteSource(ref);
  }

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(QuoteSource value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<QuoteSource>(value),
    );
  }
}

String _$quoteSourceHash() => r'52bd4ec5d89a4e8a0905cce0055baff464b9bc5e';

/// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
/// rời màn hình thì state bị bỏ.

@ProviderFor(QuoteController)
final quoteControllerProvider = QuoteControllerProvider._();

/// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
/// rời màn hình thì state bị bỏ.
final class QuoteControllerProvider
    extends $NotifierProvider<QuoteController, QuoteScreenState> {
  /// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
  /// rời màn hình thì state bị bỏ.
  QuoteControllerProvider._()
    : super(
        from: null,
        argument: null,
        retry: null,
        name: r'quoteControllerProvider',
        isAutoDispose: true,
        dependencies: null,
        $allTransitiveDependencies: null,
      );

  @override
  String debugGetCreateSourceHash() => _$quoteControllerHash();

  @$internal
  @override
  QuoteController create() => QuoteController();

  /// {@macro riverpod.override_with_value}
  Override overrideWithValue(QuoteScreenState value) {
    return $ProviderOverride(
      origin: this,
      providerOverride: $SyncValueProvider<QuoteScreenState>(value),
    );
  }
}

String _$quoteControllerHash() => r'049b03a06e15ce8196db53948ec0ba5cb8e462de';

/// State của màn hình báo giá. Không có `keepAlive` nên là auto-dispose:
/// rời màn hình thì state bị bỏ.

abstract class _$QuoteController extends $Notifier<QuoteScreenState> {
  QuoteScreenState build();
  @$mustCallSuper
  @override
  WhenComplete runBuild() {
    final ref = this.ref as $Ref<QuoteScreenState, QuoteScreenState>;
    final element =
        ref.element
            as $ClassProviderElement<
              AnyNotifier<QuoteScreenState, QuoteScreenState>,
              QuoteScreenState,
              Object?,
              Object?
            >;
    return element.handleCreate(ref, build);
  }
}

/// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
/// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
/// riêng (family).

@ProviderFor(tripSnapshot)
final tripSnapshotProvider = TripSnapshotFamily._();

/// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
/// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
/// riêng (family).

final class TripSnapshotProvider
    extends
        $FunctionalProvider<
          AsyncValue<TripSnapshot>,
          TripSnapshot,
          FutureOr<TripSnapshot>
        >
    with $FutureModifier<TripSnapshot>, $FutureProvider<TripSnapshot> {
  /// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
  /// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
  /// riêng (family).
  TripSnapshotProvider._({
    required TripSnapshotFamily super.from,
    required String super.argument,
  }) : super(
         retry: null,
         name: r'tripSnapshotProvider',
         isAutoDispose: true,
         dependencies: null,
         $allTransitiveDependencies: null,
       );

  @override
  String debugGetCreateSourceHash() => _$tripSnapshotHash();

  @override
  String toString() {
    return r'tripSnapshotProvider'
        ''
        '($argument)';
  }

  @$internal
  @override
  $FutureProviderElement<TripSnapshot> $createElement(
    $ProviderPointer pointer,
  ) => $FutureProviderElement(pointer);

  @override
  FutureOr<TripSnapshot> create(Ref ref) {
    final argument = this.argument as String;
    return tripSnapshot(ref, argument);
  }

  @override
  bool operator ==(Object other) {
    return other is TripSnapshotProvider && other.argument == argument;
  }

  @override
  int get hashCode {
    return argument.hashCode;
  }
}

String _$tripSnapshotHash() => r'db83915f7c2a0bc043d440add065e883d7090f34';

/// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
/// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
/// riêng (family).

final class TripSnapshotFamily extends $Family
    with $FunctionalFamilyOverride<FutureOr<TripSnapshot>, String> {
  TripSnapshotFamily._()
    : super(
        retry: null,
        name: r'tripSnapshotProvider',
        dependencies: null,
        $allTransitiveDependencies: null,
        isAutoDispose: true,
      );

  /// Một chuyến theo mã. Tham số của hàm (`id`) trở thành tham số của provider:
  /// `tripSnapshotProvider('T-1')` và `tripSnapshotProvider('T-2')` có state
  /// riêng (family).

  TripSnapshotProvider call(String id) =>
      TripSnapshotProvider._(argument: id, from: this);

  @override
  String toString() => r'tripSnapshotProvider';
}
