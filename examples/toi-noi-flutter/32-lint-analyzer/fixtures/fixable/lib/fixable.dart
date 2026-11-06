import 'dart:math';
import 'dart:async';

class Quote {
  Quote(int vnd) : vnd = vnd;

  final int vnd;

  String label() {
    var text = "${this.vnd} VND";
    return text;
  }
}

Future<int> jitter() async => Random(1).nextInt(3);

Future<void> idle() => Future<void>.delayed(Duration.zero);
