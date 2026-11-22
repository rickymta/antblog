// GIT-018 · Rebase cơ bản: viết lại nhánh lên đầu main

import { README_BOTH, README_HOUR, README_WAIT, WAIT_DOC, WAITING } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

const LAN = (t) => ({ as: 'lan', at: `2026-11-18T${t}:00+07:00` });

export default {
  id: 'GIT-018',
  start: 10,
  end: 11,
  run(s, { refs2 }) {
    s.say('Lan làm phí chờ trên nhánh tạo từ v1.1.0');
    s.git(`switch -c ${WAITING} v1.1.0`);
    s.write('README.md', README_WAIT);
    s.git('commit -am "Nhắc thời gian chờ trong mô tả dịch vụ"', LAN('10:00'));
    s.write('docs/phi-cho.md', WAIT_DOC);
    s.git('add docs/phi-cho.md');
    s.git('commit -m "Thêm tài liệu phí chờ"', LAN('11:00'));
    s.blank();
    s.say('Trong lúc đó, Minh sửa cùng dòng README trên main');
    s.git('switch main');
    s.write('README.md', README_HOUR);
    s.git('commit -am "Nhắc giờ đặt chuyến trong mô tả dịch vụ"', { as: 'minh', at: '2026-11-18T15:00:00+07:00' });
    s.git(`log --oneline --graph main ${WAITING} -6`);
    s.blank();
    s.say('Rebase nhánh của Lan lên đầu main: dừng ở commit đụng README');
    s.git(`switch ${WAITING}`);
    s.git('rebase main', { ...LAN('16:00'), expectFail: true });
    s.git('status');
    s.git('diff');
    s.blank();
    s.say('Giải quyết: giữ cả "giờ đặt" lẫn "thời gian chờ", rồi đi tiếp');
    s.write('README.md', README_BOTH);
    s.git('add README.md');
    s.git('rebase --continue', LAN('16:00'));
    s.git('log --oneline --graph -4');
    s.git('log -2 --format="%h %an | viết %ad | commit %cd | %s" --date=format:%d/%m_%H:%M');
    s.blank();
    s.say('Sau rebase, main chỉ cần fast-forward');
    s.git('switch main');
    s.git(`merge --ff-only ${WAITING}`);
    s.git(`branch -d ${WAITING}`);
    s.git('log --oneline --graph -5');
    checkRefs(s, refs2(11));
  },
};
