// GIT-053 · Workspace và Launchpad: nhiều repository cùng lúc

import { STEPS } from '../lib/history6.mjs';
import { checkRefs6, P } from './chung.mjs';

export default {
  id: 'GIT-053',
  start: 2,
  end: 3,
  /** `npm run setup -- 53`: dựng sẵn cả ba repository để mở workspace trong GitStudio. */
  prepare: () => STEPS[2].run(P),
  run(s, { refs6 }) {
    s.say('Lab dựng thêm hai repository cạnh bao-gia: app-tai-xe (Flutter, có việc dở chưa commit)');
    s.say('và tien-te (bản clone thư viện của Minh, Lan vừa push thêm một commit)');
    STEPS[2].run(P);
    s.cd('..');
    s.ls();
    s.blank();
    s.say('Từng repository một: nhánh, đồng bộ với upstream, thay đổi chưa commit');
    s.git('-C bao-gia status -sb');
    s.git('-C app-tai-xe status -sb');
    s.git('-C tien-te status -sb');
    s.blank();
    s.say('"Fetch tất cả" của workspace = fetch từng repository');
    for (const r of ['bao-gia', 'app-tai-xe', 'tien-te']) s.git(`-C ${r} fetch -q --tags --prune --all`);
    s.git('-C tien-te rev-list --left-right --count main...origin/main');
    s.blank();
    s.say('"Pull tất cả" chỉ fast-forward repository đang chậm hơn upstream, không tạo merge commit');
    s.git('-C tien-te pull --ff-only');
    s.git('-C tien-te log --oneline -3');
    s.blank();
    s.say('Việc dở trong app tài xế (tab WIP của Launchpad)');
    s.git('-C app-tai-xe diff --stat');
    s.cd('bao-gia', { show: false });
    s.check('tien-te đã khớp origin/main', s.value('-C ../tien-te rev-list --count main..origin/main') === '0');
    checkRefs6(s, refs6(3));
  },
};
