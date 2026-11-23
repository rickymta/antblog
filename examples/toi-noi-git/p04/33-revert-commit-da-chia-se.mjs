// GIT-033 · Revert: hoàn tác commit đã chia sẻ

import { WAITING_MAX_50 } from '../lib/history4.mjs';
import { checkRefs4 } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-11-24T${t}:00+07:00` });

export default {
  id: 'GIT-033',
  start: 0,
  end: 1,
  run(s, { refs4 }) {
    s.say('Minh tăng phí chờ tối đa lên 50.000 đ và push');
    s.write('src/BaoGia/WaitingFee.cs', WAITING_MAX_50(s.read('src/BaoGia/WaitingFee.cs')));
    s.git('commit -am "Tăng phí chờ tối đa lên 50.000 đ"', M('09:00'));
    s.git('push');
    s.blank();
    s.say('Bộ phận vận hành chưa duyệt mức mới: hoàn tác bằng một commit đảo ngược, không viết lại main');
    s.git('revert --no-edit HEAD', M('09:30'));
    s.git('log -2 --format="%h %s%n%b"');
    s.git('show --stat --format="%h %s" HEAD');
    s.git('push');
    s.check('phí chờ tối đa trở lại 30.000 đ', s.read('src/BaoGia/WaitingFee.cs').includes('30000m);'));
    s.blank();
    s.say('Revert một merge commit phải chọn nhánh cha giữ lại (-m 1 = nhánh đang đứng khi merge)');
    s.git('switch -c thu/revert-merge');
    s.git('revert --no-edit d263a4d', { ...M('10:00'), expectFail: true });
    s.git('revert --no-edit -m 1 d263a4d', M('10:00'));
    s.git('show --stat --format="%h %s" HEAD');
    s.git('switch -q main');
    s.git('branch -q -D thu/revert-merge');
    checkRefs4(s, refs4(1));
  },
};
