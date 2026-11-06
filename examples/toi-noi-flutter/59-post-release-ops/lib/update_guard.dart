import 'dart:async';

import 'package:flutter/material.dart';

import 'app_version.dart';
import 'force_update_screen.dart';
import 'min_version_policy.dart';
import 'store_platform.dart';
import 'update_gate.dart';
import 'url_opener.dart';

/// Bọc nội dung app, đặt ở `MaterialApp.builder` để bao cả Navigator. Đặt ở
/// `home:` thì route nào được push trong lúc chờ chính sách sẽ nằm trên guard
/// và màn hình chặn nằm sau nó.
///
/// Trong lúc chờ server, và khi server lỗi, người dùng thấy [child]: ta không
/// bắt người dùng đợi một lời cho phép. Khi có `UpdateForce`, [child] bị gỡ hẳn
/// khỏi cây.
///
/// Không có Overlay ở tầng này (Overlay nằm trong Navigator), nên banner không
/// dùng tooltip.
class UpdateGuard extends StatefulWidget {
  const UpdateGuard({
    super.key,
    required this.gate,
    required this.current,
    required this.platform,
    required this.opener,
    required this.child,
  });

  final UpdateGate gate;
  final AppVersion current;
  final StorePlatform platform;
  final UrlOpener opener;
  final Widget child;

  @override
  State<UpdateGuard> createState() => _UpdateGuardState();
}

class _UpdateGuardState extends State<UpdateGuard> {
  UpdateDecision? _decision;
  bool _bannerDismissed = false;

  @override
  void initState() {
    super.initState();
    unawaited(_check());
  }

  Future<void> _check() async {
    final decision = await widget.gate.check(
      current: widget.current,
      platform: widget.platform,
    );
    if (mounted) setState(() => _decision = decision);
  }

  Future<void> _openStore(Uri uri) async {
    try {
      await widget.opener.open(uri);
    } on Object {
      // Banner chỉ là lời nhắc: lỗi mở link không đáng làm hỏng app.
    }
  }

  @override
  Widget build(BuildContext context) {
    final decision = _decision;
    return switch (decision) {
      UpdateForce() => ForceUpdateScreen(
        message: decision.message,
        storeUri: decision.storeUri,
        opener: widget.opener,
      ),
      UpdateRecommend() when !_bannerDismissed => Column(
        children: [
          Material(
            color: Theme.of(context).colorScheme.secondaryContainer,
            child: SafeArea(
              bottom: false,
              child: ListTile(
                title: Text(decision.message),
                trailing: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    TextButton(
                      onPressed: () => unawaited(_openStore(decision.storeUri)),
                      child: const Text('Cập nhật'),
                    ),
                    TextButton(
                      onPressed: () => setState(() => _bannerDismissed = true),
                      child: const Text('Để sau'),
                    ),
                  ],
                ),
              ),
            ),
          ),
          Expanded(child: widget.child),
        ],
      ),
      _ => widget.child,
    };
  }
}
