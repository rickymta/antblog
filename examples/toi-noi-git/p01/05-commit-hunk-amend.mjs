// GIT-005 · Commit tốt: stage theo hunk và theo dòng, nội dung commit, amend

import { CALC_TWO_CHANGES, TESTS_V2 } from '../lib/history.mjs';
import { writeText } from '../lib/lab.mjs';

export default {
  id: 'GIT-005',
  start: 5,
  end: 7,
  prepare(dir) {
    // Đầu bài trong GitStudio: một file có hai thay đổi không liên quan.
    writeText(dir, 'src/BaoGia/FareCalculator.cs', CALC_TWO_CHANGES);
  },
  run(s, { hashes }) {
    s.say('Minh sửa FareCalculator.cs hai chỗ: thông báo lỗi và làm tròn cước');
    s.write('src/BaoGia/FareCalculator.cs', CALC_TWO_CHANGES);
    s.git('diff');
    s.blank();
    s.say('Stage riêng hunk thứ nhất: trả lời y cho hunk 1, n cho hunk 2');
    s.git('add -p src/BaoGia/FareCalculator.cs', { input: 'y\nn\n' });
    s.git('status --short');
    s.git('diff --staged --stat');
    s.git('commit -m "Báo lỗi rõ hơn khi quãng đường âm"', { as: 'minh', at: '2026-11-05T09:00:00+07:00' });
    s.check('commit thứ nhất trùng c6', s.value('rev-parse HEAD') === hashes.get('c6'));
    s.blank();
    s.say('Hunk còn lại thành commit thứ hai, có summary và description');
    s.git('add src/BaoGia/FareCalculator.cs');
    s.git(
      'commit -m "Làm tròn cước lên bội số 1.000 đ" -m "Khách trả tiền mặt không phải trả tiền lẻ, tài xế không phải thối."',
      { as: 'minh', at: '2026-11-05T09:05:00+07:00' },
    );
    const before = s.value('rev-parse --short HEAD');
    s.blank();
    s.say('Quên test: thêm test rồi amend vào commit vừa tạo');
    s.write('tests/BaoGia.Tests/FareCalculatorTests.cs', TESTS_V2);
    s.git('add tests/BaoGia.Tests/FareCalculatorTests.cs');
    s.git('commit --amend --no-edit', { as: 'minh', at: '2026-11-05T09:05:00+07:00', committedAt: '2026-11-05T09:12:00+07:00' });
    s.git('log --oneline -3');
    s.git('show --stat --format=fuller HEAD');
    s.say(`Mã hash trước amend: ${before}`);
    s.check('amend tạo commit mới (hash khác)', s.value('rev-parse --short HEAD') !== before);
    s.check('commit sau amend trùng c7', s.value('rev-parse HEAD') === hashes.get('c7'));
    s.check('commit trước amend vẫn còn trong reflog', s.value('reflog --format=%h').split('\n').includes(before));
  },
};
