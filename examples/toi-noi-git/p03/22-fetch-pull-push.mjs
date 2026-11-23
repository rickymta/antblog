// GIT-022 · Fetch, pull và push

import { WAIT_DOC_L1 } from '../lib/history3.mjs';
import { checkRefs3 } from './chung.mjs';

export default {
  id: 'GIT-022',
  start: 1,
  end: 2,
  run(s, { refs3 }) {
    s.say('Lan sửa tài liệu phí chờ, commit và push');
    s.cd('../lan/bao-gia');
    s.write('docs/phi-cho.md', WAIT_DOC_L1(s.read('docs/phi-cho.md')));
    s.git('commit -am "Không tính phí chờ khi khách huỷ trước khi xe tới"', { as: 'lan', at: '2026-11-20T09:00:00+07:00' });
    s.git('push');
    s.blank();
    s.say('Phía Minh: status chưa biết gì, vì chưa liên lạc với remote');
    s.cd('../../bao-gia');
    s.git('status -sb');
    s.git('fetch');
    s.git('status -sb');
    s.git('log --oneline main..origin/main');
    s.git('diff --stat main origin/main');
    s.blank();
    s.say('Fetch chỉ cập nhật origin/main; pull = fetch + đưa thay đổi vào nhánh hiện tại');
    s.git('pull --ff-only');
    s.git('status -sb');
    s.check('main của Minh trùng origin/main', s.value('rev-parse main') === s.value('rev-parse origin/main'));
    checkRefs3(s, refs3(2));
  },
};
