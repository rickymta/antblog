/// Mô hình chi phí phút của một lần phát hành, tính bằng phần nghìn USD
/// (milli-USD) để không dùng số thực. Giá là số kiểm tại 11/2026 trên trang
/// chính thức của từng nhà cung cấp; thời gian job là giả định minh hoạ.
class RunnerRate {
  const RunnerRate(this.name, this.milliUsdPerMinute);

  final String name;
  final int milliUsdPerMinute;

  int cost(int minutes) => milliUsdPerMinute * minutes;
}

/// docs.github.com, "About billing for GitHub Actions": runner chuẩn.
const githubLinux = RunnerRate('GitHub Linux 2 nhân', 6);
const githubMacos = RunnerRate('GitHub macOS 3 hoặc 4 nhân', 62);

/// codemagic.io/pricing, gói pay as you go.
const codemagicMacM2 = RunnerRate('Codemagic Mac mini M2', 95);
const codemagicLinux = RunnerRate('Codemagic Linux X2', 45);

/// Thời gian job minh hoạ (phút). Chưa đo trên runner thật.
const illustrativeAndroidMinutes = 15;
const illustrativeIosMinutes = 30;

/// Tiền included mỗi tháng của GitHub Free cho repository riêng, milli-USD
/// (docs.github.com, kiểm tại 11/2026). Repository công khai: runner chuẩn miễn phí.
const githubFreeIncludedMilliUsd = 12000;

/// Chi phí một lần phát hành (Android + iOS) khi hai job chạy trên [linux] và
/// [mac] với số phút đã cho.
int releaseCost({
  required RunnerRate linux,
  required RunnerRate mac,
  int androidMinutes = illustrativeAndroidMinutes,
  int iosMinutes = illustrativeIosMinutes,
}) => linux.cost(androidMinutes) + mac.cost(iosMinutes);

/// Số lần phát hành trọn vẹn trong [budgetMilliUsd] (chia nguyên).
int releasesWithin(int budgetMilliUsd, int costPerRelease) {
  if (costPerRelease <= 0) throw ArgumentError('costPerRelease phải dương');
  return budgetMilliUsd ~/ costPerRelease;
}

/// Một giá trị milli-USD dạng "1,95 USD" (dấu phẩy thập phân kiểu Việt Nam).
String formatUsd(int milliUsd) {
  final cents = (milliUsd + 5) ~/ 10;
  final whole = cents ~/ 100;
  final frac = (cents % 100).toString().padLeft(2, '0');
  return '$whole,$frac USD';
}
