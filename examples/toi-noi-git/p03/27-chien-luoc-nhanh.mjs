// GIT-027 · Chiến lược nhánh; bảo vệ main bằng hook pre-receive trên remote

import { PROTECT_HOOK } from '../lib/history3.mjs';
import { writeText } from '../lib/lab.mjs';
import { checkRefs3, P } from './chung.mjs';

export default {
  id: 'GIT-027',
  start: 4,
  end: 5,
  run(s, { refs3 }) {
    s.say('Cài hook pre-receive vào remote (trên GitHub/GitLab: Branch protection / Protected branches)');
    writeText(P.remote, 'hooks/pre-receive', PROTECT_HOOK);
    s.print('# hooks/pre-receive của remote:');
    for (const line of PROTECT_HOOK.trimEnd().split('\n')) s.print(`#   ${line}`);
    s.blank();
    s.say('Lan thử viết lại lịch sử main trên remote');
    s.cd('../lan/bao-gia');
    s.git('push --force origin HEAD~1:main', { expectFail: true });
    s.blank();
    s.say('Lan thử xoá tag phát hành');
    s.git('push origin --delete v1.0.0', { expectFail: true });
    s.blank();
    s.say('Push bình thường (fast-forward) vẫn được: thử bằng một nhánh rồi xoá');
    s.git('push origin HEAD:refs/heads/thu/hook');
    s.git('push -q origin --delete thu/hook');
    s.cd('../../bao-gia', { show: false });
    s.check('main trên remote không đổi', s.value('ls-remote origin refs/heads/main').split('\t')[0] === s.value('rev-parse main'));
    checkRefs3(s, refs3(5));
  },
};
