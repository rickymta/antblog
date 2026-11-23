// GIT-038 · Rebase --onto và tách nhánh lỡ tạo sai gốc

import { NIGHT_BRANCH, ROUND_BRANCH, WAITING_SECONDS } from '../lib/history4.mjs';
import { checkRefs4 } from './chung.mjs';

export default {
  id: 'GIT-038',
  start: 4,
  end: 5,
  run(s, { refs4 }) {
    s.say('Minh tạo nhánh sửa lỗi phí chờ trong lúc vẫn đứng ở nhánh tính năng');
    s.git('branch --show-current');
    s.git(`switch -c ${ROUND_BRANCH}`);
    s.write('src/BaoGia/WaitingFee.cs', WAITING_SECONDS(s.read('src/BaoGia/WaitingFee.cs')));
    s.git('commit -am "Tính phí chờ từ số giây, làm tròn lên phút"', { as: 'minh', at: '2026-11-25T10:00:00+07:00' });
    s.say('Merge nhánh này vào main sẽ kéo theo cả hai commit phụ phí đêm chưa review:');
    s.git(`log --oneline main..${ROUND_BRANCH}`);
    s.blank();
    s.say('rebase --onto <gốc mới> <gốc cũ> <nhánh>: chép các commit sau gốc cũ lên gốc mới');
    s.git(`rebase --onto main ${NIGHT_BRANCH} ${ROUND_BRANCH}`, { committedAt: '2026-11-25T10:05:00+07:00' });
    s.git(`log --oneline main..${ROUND_BRANCH}`);
    s.git(`log --oneline main..${NIGHT_BRANCH}`);
    s.blank();
    s.say('Sửa lỗi nhỏ, đã review: đưa vào main bằng fast-forward, push, xoá nhánh');
    s.git('switch main');
    s.git(`merge --ff-only ${ROUND_BRANCH}`);
    s.git('push');
    s.git(`branch -d ${ROUND_BRANCH}`);
    checkRefs4(s, refs4(5));
  },
};
