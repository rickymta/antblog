// GIT-002 · Cài Git và GitStudio, cấu hình danh tính commit
// Cấu hình "toàn cục" của bài này nằm trong work/home/.gitconfig (không đụng cấu hình máy bạn).

import { join } from 'node:path';
import { WORK } from '../lib/lab.mjs';

export default {
  id: 'GIT-002',
  start: null,
  end: null,
  run(s) {
    const g = { globalConfig: join(WORK, 'home', '.gitconfig') };
    s.cd('.', { show: false });
    s.write('home/.gitconfig', '');
    s.say('Danh tính dùng cho mọi repository trên máy');
    s.git('config --global user.name "Trần Minh"', g);
    s.git('config --global user.email minh@toinoi.test', g);
    s.git('config --global init.defaultBranch main', g);
    s.git('config --global --get user.name', g);
    s.git('config --global --list', g);
    s.blank();
    s.say('Repository cá nhân dùng email khác: cấu hình riêng (local) thắng cấu hình toàn cục');
    s.git('init -q du-an-ca-nhan', g);
    s.cd('du-an-ca-nhan');
    s.git('config user.email minh.ca-nhan@example.com', g);
    s.git('config --show-scope --get user.email', g);
    s.git('config --show-scope --get user.name', g);
    s.git('config --show-origin --get user.email', g);
    s.blank();
    s.say('Commit thử để thấy danh tính được ghi vào commit');
    s.write('ghi-chu.md', '# Ghi chú\n');
    s.git('add ghi-chu.md', g);
    s.git('commit -q -m "Ghi chú đầu tiên"', g);
    s.git('log -1 --format="%an <%ae>"', g);
    s.check('local email thắng global', s.value('log -1 --format=%ae', g) === 'minh.ca-nhan@example.com');
    s.check('tên lấy từ global', s.value('log -1 --format=%an', g) === 'Trần Minh');
    s.check('nhánh khởi tạo là main', s.value('branch --show-current', g) === 'main');
  },
};
