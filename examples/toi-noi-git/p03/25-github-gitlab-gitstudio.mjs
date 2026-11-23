// GIT-025 · GitHub và GitLab trong GitStudio
// Mô phỏng cách GitHub để lộ pull request dưới dạng ref refs/pull/<số>/head, và lấy về để thử.

import { WAITING_FEE_V1 } from '../lib/history3.mjs';
import { checkRefs3, P } from './chung.mjs';

export default {
  id: 'GIT-025',
  start: 3,
  end: 3,
  run(s, { refs3 }) {
    s.say('Lan đẩy một nhánh; "GitHub" (remote của lab) gắn cho pull request số 12 một ref riêng');
    s.cd('../lan/bao-gia', { show: false });
    s.git('switch -q -c thu/pr');
    s.write('src/BaoGia/WaitingFee.cs', WAITING_FEE_V1);
    s.git('add src/BaoGia/WaitingFee.cs');
    s.git('commit -q -m "Thử phí chờ"', { as: 'lan', at: '2026-11-20T15:00:00+07:00' });
    s.git('push -q origin thu/pr');
    const head = s.value('rev-parse HEAD');
    s.cd('../../remote/bao-gia.git', { show: false });
    s.git(`update-ref refs/pull/12/head ${head}`, { quiet: true });
    s.cd('../../bao-gia', { show: false });
    s.blank();
    s.say('Phía Minh: refs/pull/* không nằm trong refspec mặc định, phải fetch riêng');
    s.git('ls-remote origin "refs/pull/*"');
    s.git('fetch origin pull/12/head:pr-12');
    s.git('log --oneline -1 pr-12');
    s.git('config --get remote.origin.fetch');
    s.blank();
    s.say('Dọn: xoá nhánh thử và ref của pull request');
    s.git('branch -q -D pr-12');
    s.git('push -q origin --delete thu/pr');
    s.cd('../remote/bao-gia.git', { show: false });
    s.git('update-ref -d refs/pull/12/head', { quiet: true });
    s.cd('../../lan/bao-gia', { show: false });
    s.git('switch -q main');
    s.git('branch -q -D thu/pr');
    s.git('fetch -q --prune');
    s.cd('../../bao-gia', { show: false });
    s.check('remote không còn ref pull request', !s.value('ls-remote origin').includes('refs/pull'));
    void P;
    checkRefs3(s, refs3(3));
  },
};
