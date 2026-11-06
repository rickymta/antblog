/// Định nghĩa một cờ: khoá, giá trị mặc định **do app sở hữu**, cách đọc giá
/// trị thô từ server, và có phải kill switch hay không.
final class FlagDef<T extends Object> {
  const FlagDef._(this.key, this.defaultValue, this._parse, this.killSwitch);

  /// Cờ bật/tắt. Chấp nhận `true`/`false` (bool hoặc chuỗi, không phân biệt
  /// hoa thường); mọi giá trị khác bị coi là không hợp lệ.
  static FlagDef<bool> boolean(
    String key, {
    required bool defaultValue,
    bool killSwitch = false,
  }) => FlagDef._(key, defaultValue, _parseBool, killSwitch);

  /// Cờ số nguyên trong khoảng [min, max]; ngoài khoảng là không hợp lệ.
  static FlagDef<int> intInRange(
    String key, {
    required int defaultValue,
    required int min,
    required int max,
  }) {
    assert(defaultValue >= min && defaultValue <= max);
    return FlagDef._(key, defaultValue, (raw) {
      final value = switch (raw) {
        int() => raw,
        String() => int.tryParse(raw.trim()),
        _ => null,
      };
      return value != null && value >= min && value <= max ? value : null;
    }, false);
  }

  final String key;
  final T defaultValue;
  final bool killSwitch;
  final T? Function(Object? raw) _parse;

  /// Giá trị hợp lệ hoặc `null` nếu [raw] không đọc được.
  T? parse(Object? raw) => _parse(raw);

  static bool? _parseBool(Object? raw) => switch (raw) {
    bool() => raw,
    String() => switch (raw.trim().toLowerCase()) {
      'true' => true,
      'false' => false,
      _ => null,
    },
    _ => null,
  };
}

/// Các cờ của Tới Nơi. Mặc định là trạng thái an toàn khi không biết gì:
/// tính năng mới tắt, chức năng đang chạy thì bật.
abstract final class Flags {
  /// Kill switch: tắt lựa chọn thanh toán VíXanh nếu cổng lỗi hàng loạt.
  static final vixanhPayment = FlagDef.boolean(
    'payment_vixanh_enabled',
    defaultValue: true,
    killSwitch: true,
  );

  /// Tính năng mới, chưa phát cho mọi người.
  static final newRouting = FlagDef.boolean(
    'driver_new_routing_enabled',
    defaultValue: false,
  );

  /// Chu kỳ làm mới báo giá (giây).
  static final quoteRefreshSeconds = FlagDef.intInRange(
    'quote_refresh_seconds',
    defaultValue: 30,
    min: 10,
    max: 300,
  );

  static final List<FlagDef<Object>> all = [
    vixanhPayment,
    newRouting,
    quoteRefreshSeconds,
  ];
}

/// Ranh giới tới dịch vụ cấu hình từ xa. Trả các giá trị **do server đặt**
/// (khoá nào server không đặt thì không có mặt). Ném lỗi khi thất bại.
abstract interface class RemoteFlagSource {
  Future<Map<String, Object?>> fetch();
}

/// Bộ nhớ giữ giá trị đã tải lần trước, để lần mở app sau có giá trị ngay.
abstract interface class FlagCache {
  Future<Map<String, Object?>?> read();
  Future<void> write(Map<String, Object?> values);
}

/// Đọc cờ theo bốn lớp: mặc định trong app < cache < giá trị mới tải. Không
/// bao giờ ném lỗi ra ngoài.
///
/// - [load] (lúc khởi động) nạp cache, nên cả phiên dùng một bộ giá trị ổn định.
/// - [refresh] tải bản mới, ghi cache; chỉ **kill switch** có hiệu lực ngay,
///   cờ thường chờ lần mở app sau.
final class FeatureFlags {
  FeatureFlags({
    required this._source,
    required this._cache,
    List<FlagDef<Object>>? catalog,
  }) : _catalog = catalog ?? Flags.all;

  final RemoteFlagSource _source;
  final FlagCache _cache;
  final List<FlagDef<Object>> _catalog;
  final Map<String, Object> _active = {};

  /// Giá trị đang có hiệu lực của [flag].
  T value<T extends Object>(FlagDef<T> flag) {
    final active = _active[flag.key];
    return active is T ? active : flag.defaultValue;
  }

  Future<void> load() async {
    Map<String, Object?>? cached;
    try {
      cached = await _cache.read();
    } on Object {
      cached = null;
    }
    _active
      ..clear()
      ..addAll(_validated(cached));
  }

  /// Trả `true` nếu tải thành công.
  Future<bool> refresh() async {
    final Map<String, Object?> fetched;
    try {
      fetched = await _source.fetch();
    } on Object {
      return false;
    }
    final valid = _validated(fetched);
    try {
      await _cache.write(valid);
    } on Object {
      // Không ghi được cache: lần mở sau dùng mặc định hoặc cache cũ.
    }
    for (final flag in _catalog.where((f) => f.killSwitch)) {
      final next = valid[flag.key];
      if (next == null) {
        _active.remove(flag.key);
      } else {
        _active[flag.key] = next;
      }
    }
    return true;
  }

  /// Giữ lại đúng các khoá trong danh mục có giá trị đọc được.
  Map<String, Object> _validated(Map<String, Object?>? raw) {
    if (raw == null) return const {};
    return {
      for (final flag in _catalog)
        if (flag.parse(raw[flag.key]) case final Object value) flag.key: value,
    };
  }
}
