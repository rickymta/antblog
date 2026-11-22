// GIT-011 · Nhánh là gì: con trỏ di động trên đồ thị

import { FEATURE, SETTINGS_PEAK } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

export default {
  id: 'GIT-011',
  start: 0,
  end: 1,
  run(s, { refs2 }) {
    s.say('Repository đang có một nhánh');
    s.git('branch');
    s.git('branch --show-current');
    s.blank();
    s.say('Tạo nhánh tính năng và chuyển sang đó');
    s.git(`switch -c ${FEATURE}`);
    s.git('branch');
    s.say(`.git/HEAD chứa: ${s.read('.git/HEAD').trim()}`);
    s.git(`rev-parse main ${FEATURE}`);
    s.check('nhánh mới trỏ cùng commit với main', s.value('rev-parse main') === s.value(`rev-parse ${FEATURE}`));
    s.blank();
    s.say('Commit trên nhánh tính năng: chỉ nhánh này tiến lên');
    s.write('src/BaoGia/appsettings.json', SETTINGS_PEAK);
    s.git('add src/BaoGia/appsettings.json');
    s.git('commit -m "Thêm hệ số phụ phí giờ cao điểm vào bảng giá"', { as: 'minh', at: '2026-11-16T09:00:00+07:00' });
    s.git('log --oneline --graph --all -3');
    s.blank();
    s.say('Về main: working tree trở về đúng commit của main');
    s.git('switch main');
    s.git('grep -c Multiplier -- src/BaoGia/appsettings.json', { expectFail: true });
    s.git(`switch ${FEATURE}`);
    s.git('grep -c Multiplier -- src/BaoGia/appsettings.json');
    s.blank();
    s.say('Tạo, đổi tên, xoá một nhánh thử');
    s.git('branch thu-nghiem main');
    s.git('branch -m thu-nghiem thu-nghiem-gio-cao-diem');
    s.git('branch -v');
    s.git('branch -d thu-nghiem-gio-cao-diem');
    s.blank();
    s.say('Xoá nhánh có commit chưa merge: Git từ chối');
    s.git('switch main');
    s.git(`branch -d ${FEATURE}`, { expectFail: true });
    s.git(`switch ${FEATURE}`);
    checkRefs(s, refs2(1));
  },
};
