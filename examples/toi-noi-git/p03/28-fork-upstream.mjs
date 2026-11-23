// GIT-028 · Fork và đóng góp vào repository của người khác

import { FORK_BRANCH, FORK_DOC, README_FORK_UPSTREAM } from '../lib/history3.mjs';
import { checkRefs3, FORK_SHOW, P } from './chung.mjs';

export default {
  id: 'GIT-028',
  start: 5,
  end: 6,
  run(s, { refs3 }) {
    s.say('"Fork" trên GitHub = một bản sao repository trên máy chủ, thuộc tài khoản của Lan');
    s.git(`clone --bare ${P.remote} ${P.fork}`, { show: `clone --bare <lab>/work/remote/bao-gia.git ${FORK_SHOW}` });
    s.blank();
    s.say('Lan: repository chung thành upstream, fork thành origin');
    s.cd('../lan/bao-gia');
    s.git('remote rename origin upstream');
    s.git(`remote add origin ${P.fork}`, { show: `remote add origin ${FORK_SHOW}` });
    s.git('fetch -q origin');
    s.git('remote -v');
    s.git(`switch -c ${FORK_BRANCH}`);
    s.write('docs/phi-cho.md', FORK_DOC(s.read('docs/phi-cho.md')));
    s.git('commit -am "Nói rõ phí chờ tính thêm mỗi phút"', { as: 'lan', at: '2026-11-22T09:00:00+07:00' });
    s.git(`push -u origin ${FORK_BRANCH}`);
    s.blank();
    s.say('Trong lúc đó main của repository chung có commit mới');
    s.cd('../../bao-gia');
    s.write('README.md', README_FORK_UPSTREAM(s.read('README.md')));
    s.git('commit -q -am "Hướng dẫn báo lỗi qua issue"', { as: 'minh', at: '2026-11-22T09:30:00+07:00' });
    s.git('push -q');
    s.blank();
    s.say('Lan đồng bộ fork với upstream trước khi mở pull request');
    s.cd('../lan/bao-gia');
    s.git('fetch upstream');
    s.git('rebase upstream/main', { as: 'lan', at: '2026-11-22T10:00:00+07:00' });
    s.git('push --force-with-lease');
    s.blank();
    s.say('Minh nhận pull request từ fork: thêm remote của fork, fetch, merge --no-ff');
    s.cd('../../bao-gia');
    s.git(`remote add lan-fork ${P.fork}`, { show: `remote add lan-fork ${FORK_SHOW}` });
    s.git('fetch lan-fork');
    s.git(`merge --no-ff --no-edit lan-fork/${FORK_BRANCH}`, { as: 'minh', at: '2026-11-22T11:00:00+07:00' });
    s.git('push -q');
    s.git('remote remove lan-fork');
    s.git('log --oneline --graph -4');
    s.blank();
    s.say('Lan cập nhật main của mình và của fork, xoá nhánh đã merge');
    s.cd('../lan/bao-gia');
    s.git('switch -q main');
    s.git('pull --ff-only upstream main');
    s.git('push -q origin main');
    s.git(`push -q origin --delete ${FORK_BRANCH}`);
    s.git(`branch -d ${FORK_BRANCH}`);
    s.cd('../../bao-gia', { show: false });
    checkRefs3(s, refs3(6));
  },
};
