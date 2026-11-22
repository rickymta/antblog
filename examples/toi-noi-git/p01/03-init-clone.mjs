// GIT-003 · Tạo repository: init, clone và mở trong GitStudio

import { README } from '../lib/history.mjs';

export default {
  id: 'GIT-003',
  start: null,
  end: 1,
  run(s, { hashes }) {
    s.cd('.', { show: false });
    s.git('init -b main bao-gia');
    s.cd('bao-gia');
    s.ls('.git');
    s.git('status');
    s.blank();
    s.say('Danh tính riêng của repository lab');
    s.git('config user.name "Trần Minh"');
    s.git('config user.email minh@toinoi.test');
    s.blank();
    s.say('Tạo README.md rồi commit đầu tiên');
    s.write('README.md', README);
    s.git('status --short');
    s.git('add README.md');
    s.git('commit -m "Khởi tạo dịch vụ báo giá"', { as: 'minh', at: '2026-11-02T08:30:00+07:00' });
    s.git('log --oneline');
    s.check('commit đầu tiên trùng c1 của lịch sử chuẩn', s.value('rev-parse HEAD') === hashes.get('c1'));
    s.blank();
    s.say('Clone repository vừa tạo sang thư mục khác');
    s.cd('..');
    s.git('clone bao-gia bao-gia-ban-sao');
    s.cd('bao-gia-ban-sao');
    s.git('log --oneline');
    s.git('remote -v');
    s.git('status');
    s.check('bản clone có cùng commit', s.value('rev-parse HEAD') === hashes.get('c1'));
    s.check('bản clone có remote origin', s.value('remote') === 'origin');
  },
};
