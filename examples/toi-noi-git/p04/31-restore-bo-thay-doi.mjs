// GIT-031 · Bỏ thay đổi trong working tree và staging area

import { checkRefs4 } from './chung.mjs';

export default {
  id: 'GIT-031',
  start: 0,
  end: 0,
  run(s, { refs4 }) {
    s.say('Minh thử sửa hai file rồi đổi ý');
    s.write('src/BaoGia/FareCalculator.cs', s.read('src/BaoGia/FareCalculator.cs').replace('// Chưa gồm phí cầu đường, khuyến mãi.', '// Chưa gồm phí cầu đường, khuyến mãi, phí chờ.'));
    s.write('README.md', s.read('README.md') + '\nGhi chú thử: chưa chắc giữ.\n');
    s.git('status -s');
    s.git('restore src/BaoGia/FareCalculator.cs');
    s.git('status -s');
    s.blank();
    s.say('Lỡ stage README.md: đưa ra khỏi staging area, thay đổi vẫn còn trong file');
    s.git('add README.md');
    s.git('status -s');
    s.git('restore --staged README.md');
    s.git('status -s');
    s.git('restore README.md');
    s.blank();
    s.say('Bỏ một phần: sửa hai chỗ trong WaitingFee.cs, restore -p hỏi từng hunk (y = bỏ, n = giữ)');
    s.write(
      'src/BaoGia/WaitingFee.cs',
      s
        .read('src/BaoGia/WaitingFee.cs')
        .replace('    private const int FreeMinutes = 5;', '    private const int FreeMinutes = 10;')
        .replace('        return Math.Min(billable * 1000m, 30000m);', '        // TODO: xem lại mức tối đa\n        return Math.Min(billable * 1000m, 30000m);'),
    );
    s.git('restore -p src/BaoGia/WaitingFee.cs', { input: 'y\nn\n' });
    s.git('diff');
    s.git('restore src/BaoGia/WaitingFee.cs');
    s.blank();
    s.say('Lấy một file như ở bản phát hành v1.0.0 mà không đổi HEAD, rồi trả lại');
    s.git('restore --source=v1.0.0 src/BaoGia/FareCalculator.cs');
    s.git('diff --stat');
    s.git('restore src/BaoGia/FareCalculator.cs');
    s.blank();
    s.say('File mới chưa track: restore không đụng tới, dùng clean (xem trước bằng -n)');
    s.write('ghi-chu-tam.txt', 'Nháp\n');
    s.git('restore .');
    s.git('status -s');
    s.git('clean -n');
    s.git('clean -f');
    s.git('status -s');
    s.check('working tree sạch như lúc đầu', s.value('status --porcelain') === '');
    checkRefs4(s, refs4(0));
  },
};
