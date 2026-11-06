// Chương trình nhỏ để đo `--obfuscate` và tách symbol (bài viết, mục 5).
// Không dùng trong app: test/obfuscation_test.dart biên dịch và chạy nó.

enum TripPhase { searching, accepted }

class QuoteCalculator {
  int fareFor(int meters) {
    if (meters > 100000) {
      throw StateError('quá xa');
    }
    return 12000 + meters;
  }
}

// Một chuỗi hằng trông như bí mật. Chuỗi thật không bao giờ nằm trong app.
const demoSecret = 'demo-khoa-api-gia-lap-0001';

void main() {
  print('type=${QuoteCalculator().runtimeType}');
  print('enum=${TripPhase.searching}');
  print('secret=$demoSecret');
  try {
    QuoteCalculator().fareFor(200000);
  } catch (error, stack) {
    print('error=$error');
    print(stack);
  }
}
