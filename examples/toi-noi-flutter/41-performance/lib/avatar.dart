import 'package:flutter/widgets.dart';

/// Kích thước giải mã (pixel vật lý) cho ảnh hiển thị [logicalSize] pixel
/// logic trên màn hình có [devicePixelRatio].
int decodeSizeFor(double logicalSize, double devicePixelRatio) =>
    (logicalSize * devicePixelRatio).ceil();

/// Bọc [source] để engine chỉ giải mã ở [logicalSize] pixel logic.
///
/// Không phóng to quá kích thước gốc (`allowUpscaling` mặc định `false`).
ImageProvider<Object> avatarProvider(
  ImageProvider<Object> source, {
  required double logicalSize,
  required double devicePixelRatio,
}) => ResizeImage(source, width: decodeSizeFor(logicalSize, devicePixelRatio));

/// Ảnh đại diện tài xế hiển thị [size] x [size] pixel logic.
class DriverAvatar extends StatelessWidget {
  /// Tạo ảnh đại diện từ [source].
  const DriverAvatar({required this.source, this.size = 48, super.key});

  /// Nguồn ảnh gốc, thường là ảnh lớn từ server.
  final ImageProvider<Object> source;

  /// Cạnh hiển thị, pixel logic.
  final double size;

  @override
  Widget build(BuildContext context) {
    final dpr = MediaQuery.devicePixelRatioOf(context);
    return Image(
      image: avatarProvider(source, logicalSize: size, devicePixelRatio: dpr),
      width: size,
      height: size,
      fit: BoxFit.cover,
    );
  }
}
