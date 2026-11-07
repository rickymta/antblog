import 'dart:async';

import 'package:toi_noi_core/toi_noi_core.dart';

import 'checkout_launcher.dart';
import 'payment_gateway.dart';
import 'payment_models.dart';
import 'payment_return.dart';
import 'payment_store.dart';

/// Vì sao ta chưa chốt được kết quả.
enum PendingReason {
  /// Mất mạng hoặc server lỗi lúc hỏi.
  unreachable,

  /// Hỏi được, nhưng hết lượt hỏi mà server vẫn nói `pending`.
  stillPending,

  /// Server từ chối trả lời về phiên này (4xx).
  rejected,
}

/// Các trạng thái của một lần thanh toán ở phía app. Chỉ [PaymentFinished] là
/// kết quả; mọi trạng thái còn lại là "chưa biết".
sealed class PaymentState {
  const PaymentState();

  /// Phiên của server nếu đã tạo xong.
  PaymentSession? get session => null;

  List<Object?> get _props;

  @override
  bool operator ==(Object other) {
    if (other is! PaymentState || other.runtimeType != runtimeType) {
      return false;
    }
    final a = _props;
    final b = other._props;
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  @override
  int get hashCode => Object.hash(runtimeType, Object.hashAll(_props));
}

final class PaymentIdle extends PaymentState {
  const PaymentIdle();

  @override
  List<Object?> get _props => const [];
}

/// Đang xin server một phiên.
final class PaymentCreating extends PaymentState {
  const PaymentCreating(this.request);

  final PaymentRequest request;

  @override
  List<Object?> get _props => [request];
}

/// Chưa có phiên. [cause] là `null` khi app bị đóng giữa lúc đang xin.
final class PaymentCreateFailed extends PaymentState {
  const PaymentCreateFailed(this.request, [this.cause]);

  final PaymentRequest request;
  final PaymentGatewayException? cause;

  bool get retryable => cause?.retryable ?? true;

  @override
  List<Object?> get _props => [request, cause?.kind, cause?.message];
}

/// Đã mở (hoặc cố mở) trang cổng; đợi khách quay về.
final class PaymentAwaitingCheckout extends PaymentState {
  const PaymentAwaitingCheckout(this.session, {this.launchFailed = false});

  @override
  final PaymentSession session;

  /// Không mở được trình duyệt: khách cần bấm mở lại.
  final bool launchFailed;

  @override
  List<Object?> get _props => [session, launchFailed];
}

/// Đang hỏi server.
final class PaymentConfirming extends PaymentState {
  const PaymentConfirming(this.session);

  @override
  final PaymentSession session;

  @override
  List<Object?> get _props => [session];
}

/// "Đang chờ xác nhận": chưa kết luận gì, không phải thất bại.
final class PaymentPendingConfirmation extends PaymentState {
  const PaymentPendingConfirmation(this.session, this.reason);

  @override
  final PaymentSession session;
  final PendingReason reason;

  @override
  List<Object?> get _props => [session, reason];
}

/// Server đã chốt. Bốn lớp con tương ứng bốn trạng thái cuối của server.
sealed class PaymentFinished extends PaymentState {
  const PaymentFinished(this.session);

  @override
  final PaymentSession session;

  @override
  List<Object?> get _props => [session];
}

final class PaymentSucceeded extends PaymentFinished {
  const PaymentSucceeded(super.session);
}

final class PaymentFailed extends PaymentFinished {
  const PaymentFailed(super.session);
}

final class PaymentCancelled extends PaymentFinished {
  const PaymentCancelled(super.session);
}

final class PaymentExpired extends PaymentFinished {
  const PaymentExpired(super.session);
}

/// Kết quả của việc đưa một link vào [PaymentFlow.handleReturn].
enum ReturnHandling { notPaymentLink, unknownSession, handled }

/// Lịch hỏi server: hỏi ngay một lần, rồi chờ [waits] giữa các lần hỏi tiếp.
/// Số lần hỏi tối đa là `waits.length + 1`.
final class PollPolicy {
  const PollPolicy({required this.waits});

  /// Tổng chờ tối đa 15 giây; con số minh hoạ.
  static const standard = PollPolicy(
    waits: [
      Duration(seconds: 1),
      Duration(seconds: 2),
      Duration(seconds: 4),
      Duration(seconds: 8),
    ],
  );

  final List<Duration> waits;
}

Future<void> _sleep(Duration d) => Future<void>.delayed(d);

/// Máy trạng thái thanh toán qua cổng bên ngoài. Dart thuần: không biết
/// Flutter, Riverpod hay plugin; mọi thứ bên ngoài đi qua interface.
final class PaymentFlow {
  PaymentFlow({
    required this._gateway,
    required this._launcher,
    required this._store,
    required this._newKey,
    required this._now,
    this.poll = PollPolicy.standard,
    this._delay = _sleep,
  });

  final PaymentGateway _gateway;
  final CheckoutLauncher _launcher;
  final PaymentStore _store;
  final String Function() _newKey;
  final DateTime Function() _now;
  final Future<void> Function(Duration) _delay;
  final PollPolicy poll;

  final _changes = StreamController<PaymentState>.broadcast();
  PaymentState _state = const PaymentIdle();
  Future<void>? _restored;
  Future<void>? _confirming;
  bool _starting = false;

  PaymentState get state => _state;
  Stream<PaymentState> get changes => _changes.stream;

  void dispose() => unawaited(_changes.close());

  void _set(PaymentState next) {
    if (next == _state) return;
    _state = next;
    if (!_changes.isClosed) _changes.add(next);
  }

  /// Nạp lại lần thanh toán dang dở từ kho (sau khi app bị kill). Chạy một
  /// lần; gọi nhiều lần trả cùng một Future. Nếu đã có phiên thì hỏi server
  /// luôn, vì khách có thể đã trả xong trong lúc app không chạy.
  Future<void> restore() => _restored ??= _restore();

  Future<void> _restore() async {
    final pending = await _store.load();
    if (pending == null) return;
    final session = pending.session;
    if (session == null) {
      _set(PaymentCreateFailed(pending.request));
      return;
    }
    await _confirm(session);
  }

  bool get _canStart => switch (_state) {
    PaymentIdle() ||
    PaymentCreateFailed() ||
    PaymentFailed() ||
    PaymentCancelled() ||
    PaymentExpired() => true,
    _ => false,
  };

  /// Bắt đầu thanh toán một chuyến.
  ///
  /// Khoá idempotency chỉ được dùng lại khi lần trước chưa nhận được phiên
  /// (mất mạng, app bị đóng) cho cùng chuyến và số tiền. Sau một kết quả
  /// cuối không thành công, khách thử lại là một lần thanh toán mới và có khoá
  /// mới.
  Future<void> start({required String tripId, required Money amount}) async {
    if (_starting) return;
    _starting = true;
    try {
      await restore();
      if (!_canStart) return;

      final stored = await _store.load();
      final reuse =
          stored != null &&
          stored.session == null &&
          stored.request.tripId == tripId &&
          stored.request.amount == amount;
      final request = reuse
          ? stored.request
          : PaymentRequest(
              tripId: tripId,
              amount: amount,
              idempotencyKey: _newKey(),
            );
      await _store.save(PendingPayment(request));
      _set(PaymentCreating(request));

      final PaymentSession session;
      try {
        session = await _gateway.createSession(
          tripId: tripId,
          amount: amount,
          idempotencyKey: request.idempotencyKey,
        );
      } on PaymentGatewayException catch (e) {
        if (!e.retryable) await _store.clear();
        _set(PaymentCreateFailed(request, e));
        return;
      }
      await _store.save(PendingPayment(request, session));
      await _openCheckout(session);
    } finally {
      _starting = false;
    }
  }

  Future<void> _openCheckout(PaymentSession session) async {
    _set(PaymentAwaitingCheckout(session));
    final opened = await _launcher.open(session.checkoutUrl);
    if (!opened && _state is PaymentAwaitingCheckout) {
      _set(PaymentAwaitingCheckout(session, launchFailed: true));
    }
  }

  /// Mở lại trang cổng cho phiên đang chờ. Quá hạn thì hỏi server thay vì mở.
  Future<void> reopenCheckout() async {
    final session = _waitingSession;
    if (session == null) return;
    if (!_now().isBefore(session.expiresAt)) {
      await _confirm(session);
      return;
    }
    await _openCheckout(session);
  }

  /// Hỏi lại server cho phiên đang chờ (nút "Kiểm tra lại", app trở lại
  /// foreground, có mạng lại).
  Future<void> refresh() async {
    final session = _waitingSession;
    if (session != null) await _confirm(session);
  }

  PaymentSession? get _waitingSession => switch (_state) {
    PaymentAwaitingCheckout(:final session) => session,
    PaymentPendingConfirmation(:final session) => session,
    _ => null,
  };

  /// Nhận link quay về từ cổng. Tham số của link chỉ dùng để tìm đúng phiên;
  /// kết quả luôn do server quyết.
  Future<ReturnHandling> handleReturn(Uri link) async {
    final ret = PaymentReturn.tryParse(link);
    if (ret == null) return ReturnHandling.notPaymentLink;
    await restore();
    final session = _state.session;
    if (session == null || session.id != ret.sessionId) {
      return ReturnHandling.unknownSession;
    }
    switch (_state) {
      case PaymentAwaitingCheckout() || PaymentPendingConfirmation():
        // Tab trình duyệt (nếu đóng được) che màn hình kết quả: đóng trước.
        await _launcher.close();
        await _confirm(session);
      case PaymentConfirming():
        await _confirming;
      default:
        break; // đã chốt: link thứ hai không làm gì
    }
    return ReturnHandling.handled;
  }

  Future<void> _confirm(PaymentSession session) =>
      _confirming ??= _runConfirm(session).whenComplete(() {
        _confirming = null;
      });

  Future<void> _runConfirm(PaymentSession session) async {
    _set(PaymentConfirming(session));
    for (var check = 0; ; check++) {
      final PaymentStatus status;
      try {
        status = await _gateway.fetchStatus(session.id);
      } on PaymentGatewayException catch (e) {
        _set(
          PaymentPendingConfirmation(
            session,
            e.retryable ? PendingReason.unreachable : PendingReason.rejected,
          ),
        );
        return;
      }
      if (status.isFinal) {
        _set(switch (status) {
          PaymentStatus.succeeded => PaymentSucceeded(session),
          PaymentStatus.failed => PaymentFailed(session),
          PaymentStatus.cancelled => PaymentCancelled(session),
          PaymentStatus.expired => PaymentExpired(session),
          PaymentStatus.pending => throw StateError('không thể tới đây'),
        });
        await _store.clear();
        return;
      }
      if (check >= poll.waits.length) {
        _set(PaymentPendingConfirmation(session, PendingReason.stillPending));
        return;
      }
      await _delay(poll.waits[check]);
    }
  }
}
