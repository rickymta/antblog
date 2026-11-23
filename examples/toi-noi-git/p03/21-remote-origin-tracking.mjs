// GIT-021 · Remote, origin và nhánh remote-tracking

import { checkRefs3, P, R_SHOW } from './chung.mjs';

export default {
  id: 'GIT-021',
  start: 0,
  end: 1,
  run(s, { refs3 }) {
    s.say('Repository của Minh chưa có remote nào');
    s.git('remote -v');
    s.blank();
    s.say('Tạo remote chung của nhóm (lab dùng một bare repository thay cho GitHub)');
    s.git(`init --bare -b main ${P.remote}`, { show: `init --bare -b main ${R_SHOW}` });
    s.git(`remote add origin ${P.remote}`, { show: `remote add origin ${R_SHOW}` });
    s.git('remote -v');
    s.blank();
    s.say('Push lần đầu, đặt upstream cho main; đẩy cả nhánh phát hành và tag');
    s.git('push -u origin main');
    s.git('push origin phat-hanh/1.0');
    s.git('push origin --tags');
    s.git('branch -vv');
    s.git('branch -r');
    s.git('ls-remote origin');
    s.blank();
    s.say('Lan clone repository chung về máy mình');
    s.git(`clone ${P.remote} ${P.lan}`, { show: `clone ${R_SHOW} ../lan/bao-gia` });
    s.cd('../lan/bao-gia');
    s.git('config user.name "Lê Lan"');
    s.git('config user.email lan@toinoi.test');
    s.git('branch -a');
    s.git('remote show origin');
    s.cd('../../bao-gia', { show: false });
    checkRefs3(s, refs3(1));
  },
};
