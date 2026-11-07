import 'dart:io';

import 'package:toi_noi_51/version_check_cli.dart';

void main(List<String> args) {
  exitCode = runVersionCheck(args, stdout, stderr);
}
