// Chứng minh giá trị define nằm trong binary (bài viết, mục 3).
//
//   dart compile exe -DDEMO_VALUE=gia-tri-thu tool/embed_demo.dart -o /tmp/embed_demo
//   env -i /tmp/embed_demo                       # chạy KHÔNG có biến môi trường nào
//   grep -a -c gia-tri-thu /tmp/embed_demo       # chuỗi nằm ngay trong file binary
const _value = String.fromEnvironment('DEMO_VALUE');

// ignore: avoid_print
void main() => print('value=$_value');
