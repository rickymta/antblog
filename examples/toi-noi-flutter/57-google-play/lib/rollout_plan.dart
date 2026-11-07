/// Kế hoạch staged rollout lên track production của Google Play.
///
/// Phần này thuần Dart và không gọi Play. Các con số (bước phần trăm, ngưỡng
/// crash-free, cỡ mẫu) là MINH HOẠ của Tới Nơi, không phải luật của Google.
library;

/// Giá trị `versionCode` lớn nhất Google Play cho phép (trang trợ giúp Play
/// Console về version code, kiểm tại 11/2026).
const int maxVersionCode = 2100000000;

/// Một lần đo sau khi một bước rollout đã chạy đủ lâu.
final class RolloutObservation {
  RolloutObservation({required this.users, required this.crashFreeUsersBp}) {
    if (users < 0) {
      throw ArgumentError.value(users, 'users', 'không được âm');
    }
    if (crashFreeUsersBp < 0 || crashFreeUsersBp > 10000) {
      throw ArgumentError.value(
        crashFreeUsersBp,
        'crashFreeUsersBp',
        'phải nằm trong 0..10000 (phần vạn)',
      );
    }
  }

  /// Số người dùng đã nhận bản này trong khung đo.
  final int users;

  /// Tỉ lệ người dùng không gặp crash, tính bằng phần vạn: 9950 = 99,50%.
  /// Dùng số nguyên để so sánh ngưỡng không dính sai số `double`.
  final int crashFreeUsersBp;
}

/// Luật dừng và tiến của một rollout (số minh hoạ).
final class RolloutPolicy {
  const RolloutPolicy({
    this.steps = const [1, 5, 20, 50, 100],
    this.minUsers = 500,
    this.advanceAtBp = 9950,
    this.haltBelowBp = 9900,
  });

  /// Các mức phần trăm, tăng ngặt, mức cuối là 100.
  final List<int> steps;

  /// Dưới cỡ mẫu này thì chưa kết luận gì: giữ nguyên bước.
  final int minUsers;

  /// Crash-free từ mức này trở lên (và đủ mẫu) thì tiến bước kế.
  final int advanceAtBp;

  /// Crash-free thấp hơn mức này (và đủ mẫu) thì dừng rollout.
  final int haltBelowBp;

  /// Kiểm ràng buộc của luật. `RolloutPlan.start` luôn gọi hàm này.
  void validate() {
    if (steps.isEmpty) throw ArgumentError.value(steps, 'steps', 'đang rỗng');
    if (minUsers < 1) {
      throw ArgumentError.value(minUsers, 'minUsers', 'phải dương');
    }
    for (var i = 0; i < steps.length; i++) {
      if (steps[i] < 1 || steps[i] > 100) {
        throw ArgumentError.value(steps[i], 'steps', 'phải nằm trong 1..100');
      }
      if (i > 0 && steps[i] <= steps[i - 1]) {
        throw ArgumentError.value(steps, 'steps', 'phải tăng ngặt');
      }
    }
    if (steps.last != 100) {
      throw ArgumentError.value(steps, 'steps', 'bước cuối phải là 100');
    }
    if (haltBelowBp >= advanceAtBp) {
      throw ArgumentError(
        'haltBelowBp phải nhỏ hơn advanceAtBp, để còn vùng giữ',
      );
    }
  }
}

enum RolloutAction { advance, hold, halt }

enum RolloutStatus { inProgress, halted, completed }

/// Trạng thái bất biến của một rollout. Mọi thao tác trả về bản mới.
final class RolloutPlan {
  RolloutPlan._({
    required this.versionCode,
    required this.previousVersionCode,
    required this.policy,
    required this.stepIndex,
    required this.status,
  });

  /// Bắt đầu rollout cho bản [versionCode].
  ///
  /// [previousVersionCode] là versionCode đang chạy ở production; `null` nghĩa
  /// là bản đầu tiên của app. Play chỉ cho staged rollout với bản cập nhật,
  /// không cho lần xuất bản đầu (trang "Release app updates with staged
  /// rollouts", kiểm tại 11/2026), nên bản đầu đi thẳng tới 100%.
  factory RolloutPlan.start({
    required int versionCode,
    int? previousVersionCode,
    RolloutPolicy policy = const RolloutPolicy(),
  }) {
    policy.validate();
    _checkVersionCode(versionCode);
    if (previousVersionCode != null) {
      _checkVersionCode(previousVersionCode);
      if (versionCode <= previousVersionCode) {
        throw ArgumentError.value(
          versionCode,
          'versionCode',
          'phải lớn hơn $previousVersionCode đang ở production',
        );
      }
    }
    final startsAtEnd = previousVersionCode == null || policy.steps.length == 1;
    return RolloutPlan._(
      versionCode: versionCode,
      previousVersionCode: previousVersionCode,
      policy: policy,
      stepIndex: startsAtEnd ? policy.steps.length - 1 : 0,
      status: startsAtEnd ? RolloutStatus.completed : RolloutStatus.inProgress,
    );
  }

  final int versionCode;
  final int? previousVersionCode;
  final RolloutPolicy policy;
  final int stepIndex;
  final RolloutStatus status;

  /// Phần trăm người dùng đủ điều kiện nhận bản này.
  int get percent => policy.steps[stepIndex];

  /// Quyết định cho một lần đo, không đổi trạng thái.
  RolloutAction decide(RolloutObservation observation) {
    _requireInProgress();
    if (observation.users < policy.minUsers) return RolloutAction.hold;
    if (observation.crashFreeUsersBp < policy.haltBelowBp) {
      return RolloutAction.halt;
    }
    if (observation.crashFreeUsersBp >= policy.advanceAtBp) {
      return RolloutAction.advance;
    }
    return RolloutAction.hold;
  }

  /// Áp quyết định của [decide] và trả về kế hoạch mới.
  RolloutPlan observe(RolloutObservation observation) =>
      switch (decide(observation)) {
        RolloutAction.hold => this,
        RolloutAction.halt => halt(),
        RolloutAction.advance => _advance(),
      };

  /// Dừng tay (nút Halt rollout), cả khi rollout đang chạy lẫn khi đã 100%.
  ///
  /// Đang chạy: Play ngừng phát cho thêm người, người đã nhận giữ bản đó.
  /// Đã 100%: trang "Halting a fully rolled-out release" nói bản hoàn chỉnh
  /// trước đó được phát lại cho người dùng mới và người chưa cài bản này; ai
  /// đã cập nhật thì không bị hạ bản. Không halt được lần phát hành đầu của
  /// một track (không có bản trước để quay lại), nên lab từ chối khi
  /// [previousVersionCode] là `null`. Internal không mô hình hoá ở đây.
  RolloutPlan halt() {
    if (status == RolloutStatus.halted) {
      throw StateError('rollout đã dừng rồi');
    }
    if (previousVersionCode == null) {
      throw StateError('không halt được lần phát hành đầu của một track');
    }
    return _copy(status: RolloutStatus.halted);
  }

  /// Tiếp tục một rollout đã dừng ở mức [percent] (một trong `policy.steps`),
  /// mặc định là mức đang giữ. Play cho resume ở mức bất kỳ; lab chỉ nhận mức
  /// nằm trong bậc thang để lịch rollout còn đo được. Chỉ nên làm khi đã
  /// chắc bản không lỗi; nếu có lỗi thì dùng [fixForward].
  RolloutPlan resume({int? percent}) {
    if (status != RolloutStatus.halted) {
      throw StateError('chỉ resume được rollout đang dừng, hiện là $status');
    }
    final index = percent == null ? stepIndex : policy.steps.indexOf(percent);
    if (index < 0) {
      throw ArgumentError.value(
        percent,
        'percent',
        'phải là một trong ${policy.steps}',
      );
    }
    return _copy(
      stepIndex: index,
      status: index == policy.steps.length - 1
          ? RolloutStatus.completed
          : RolloutStatus.inProgress,
    );
  }

  /// Cách sửa lỗi theo Play: phát bản mới đã sửa, versionCode cao hơn.
  /// Không có thao tác nào "quay về" versionCode thấp hơn.
  RolloutPlan fixForward({required int newVersionCode}) => RolloutPlan.start(
    versionCode: newVersionCode,
    previousVersionCode: versionCode,
    policy: policy,
  );

  RolloutPlan _advance() => _copy(
    stepIndex: stepIndex + 1,
    status: stepIndex + 1 == policy.steps.length - 1
        ? RolloutStatus.completed
        : RolloutStatus.inProgress,
  );

  RolloutPlan _copy({int? stepIndex, RolloutStatus? status}) => RolloutPlan._(
    versionCode: versionCode,
    previousVersionCode: previousVersionCode,
    policy: policy,
    stepIndex: stepIndex ?? this.stepIndex,
    status: status ?? this.status,
  );

  void _requireInProgress() {
    if (status != RolloutStatus.inProgress) {
      throw StateError('rollout không còn chạy, hiện là $status');
    }
  }

  static void _checkVersionCode(int value) {
    if (value < 1 || value > maxVersionCode) {
      throw ArgumentError.value(
        value,
        'versionCode',
        'phải nằm trong 1..$maxVersionCode',
      );
    }
  }
}
