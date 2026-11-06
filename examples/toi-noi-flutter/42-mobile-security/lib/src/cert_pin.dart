import 'dart:convert';
import 'dart:typed_data';

import 'package:crypto/crypto.dart';

import 'der.dart';

/// Hai cách định danh một chứng chỉ để ghim.
enum PinKind {
  /// SHA-256 của `SubjectPublicKeyInfo`: đổi khi đổi khoá, giữ nguyên khi
  /// cấp lại chứng chỉ trên cùng một khoá.
  spki('spki-sha256'),

  /// SHA-256 của toàn bộ chứng chỉ DER: đổi mỗi lần cấp lại, kể cả cùng khoá.
  certificate('cert-sha256');

  const PinKind(this.prefix);

  final String prefix;
}

/// Một giá trị ghim, dạng chuỗi `spki-sha256/<base64>` hoặc
/// `cert-sha256/<base64>`.
final class CertPin {
  const CertPin._(this.kind, this.hash);

  /// Tính ghim [kind] của chứng chỉ có bytes DER là [der].
  factory CertPin.of(Uint8List der, {PinKind kind = PinKind.spki}) {
    final input = switch (kind) {
      PinKind.spki => extractSubjectPublicKeyInfo(der),
      PinKind.certificate => der,
    };
    return CertPin._(kind, base64.encode(sha256.convert(input).bytes));
  }

  /// Đọc lại chuỗi đã lưu trong cấu hình. Ném [FormatException] nếu sai dạng.
  factory CertPin.parse(String text) {
    final slash = text.indexOf('/');
    if (slash < 0) {
      throw FormatException('Ghim phải có dạng "<loại>/<base64>"', text);
    }
    final prefix = text.substring(0, slash);
    final kind = PinKind.values.where((k) => k.prefix == prefix).firstOrNull;
    if (kind == null) {
      throw FormatException('Loại ghim không biết: $prefix', text);
    }
    final hash = text.substring(slash + 1);
    final bytes = base64.decode(hash);
    if (bytes.length != 32) {
      throw FormatException('SHA-256 phải dài 32 byte, nhận ${bytes.length}');
    }
    return CertPin._(kind, hash);
  }

  final PinKind kind;

  /// SHA-256 mã hoá base64.
  final String hash;

  /// Chứng chỉ [der] có khớp ghim này không.
  bool matches(Uint8List der) => CertPin.of(der, kind: kind) == this;

  @override
  bool operator ==(Object other) =>
      other is CertPin && other.kind == kind && other.hash == hash;

  @override
  int get hashCode => Object.hash(kind, hash);

  @override
  String toString() => '${kind.prefix}/$hash';
}

/// Chính sách ghim: mỗi host có một tập ghim; host ngoài danh sách bị từ chối.
final class PinPolicy {
  /// Mặc định mỗi host cần ít nhất 2 ghim (khoá đang dùng và khoá dự phòng
  /// giữ ngoại tuyến) để một lần đổi khoá không khoá chết app.
  PinPolicy(
    Map<String, Set<CertPin>> pinsByHost, {
    int minPinsPerHost = 2,
    DateTime Function()? now,
  }) : _pinsByHost = {
         for (final entry in pinsByHost.entries)
           entry.key.toLowerCase(): Set.unmodifiable(entry.value),
       },
       _now = now ?? DateTime.now {
    for (final entry in _pinsByHost.entries) {
      if (entry.value.length < minPinsPerHost) {
        throw ArgumentError.value(
          entry.value.length,
          'pinsByHost[${entry.key}]',
          'cần ít nhất $minPinsPerHost ghim để còn đường rotation',
        );
      }
    }
  }

  final Map<String, Set<CertPin>> _pinsByHost;
  final DateTime Function() _now;

  /// Cho phép [host] kết nối khi chứng chỉ lá [der] khớp một ghim và còn hạn.
  ///
  /// [startValidity] và [endValidity] lấy từ `X509Certificate`: chứng chỉ hết
  /// hạn không được chấp nhận chỉ vì khoá còn khớp.
  bool accepts(
    String host,
    Uint8List der, {
    required DateTime startValidity,
    required DateTime endValidity,
  }) {
    final pins = _pinsByHost[host.toLowerCase()];
    if (pins == null) return false;
    final now = _now();
    if (now.isBefore(startValidity) || now.isAfter(endValidity)) return false;
    return pins.any((pin) => pin.matches(der));
  }
}
