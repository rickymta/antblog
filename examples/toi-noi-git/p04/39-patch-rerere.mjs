// GIT-039 · Patch, rerere và merge lại nhiều lần

import { DOC_LAN, DOC_RESOLVED, LAN_BRANCH, NIGHT_BRANCH } from '../lib/history4.mjs';
import { checkRefs4 } from './chung.mjs';

export default {
  id: 'GIT-039',
  start: 5,
  end: 6,
  run(s, { refs4 }) {
    s.say('Lan chưa có quyền push: sửa tài liệu trên nhánh riêng rồi xuất patch gửi Minh');
    s.cd('../lan/bao-gia');
    s.git('fetch -q upstream');
    s.git(`switch -c ${LAN_BRANCH} upstream/main`);
    s.write('docs/phi-cho.md', DOC_LAN(s.read('docs/phi-cho.md')));
    s.git('commit -q -am "Ghi rõ phí chờ tối đa tính cho mỗi chuyến"', { as: 'lan', at: '2026-11-25T11:00:00+07:00' });
    const patch = s.git('format-patch -1 -o ../patches').stdout.trim();
    const name = patch.split('/').pop();
    s.git(`switch -q main`);
    s.git(`branch -q -D ${LAN_BRANCH}`);
    s.blank();
    s.say('Minh xem patch, thử áp, rồi git am: tác giả vẫn là Lan, Minh là người commit');
    s.cd('../../bao-gia');
    s.git(`apply --stat ../lan/patches/${name}`);
    s.git(`apply --check ../lan/patches/${name}`);
    s.git(`am ../lan/patches/${name}`, { committedAt: '2026-11-25T11:30:00+07:00' });
    s.git('log -1 --format=fuller');
    s.git('push');
    s.blank();
    s.say('rerere: bật, thử merge main vào nhánh tính năng; Git ghi lại xung đột và cách giải quyết');
    s.git('config rerere.enabled true');
    s.git(`switch -q ${NIGHT_BRANCH}`);
    s.git('merge main', { as: 'minh', at: '2026-11-25T13:00:00+07:00', expectFail: true });
    s.git('diff');
    s.write('docs/phi-cho.md', DOC_RESOLVED(s.read('docs/phi-cho.md')));
    s.git('add docs/phi-cho.md');
    s.git('merge --continue', { as: 'minh', at: '2026-11-25T13:00:00+07:00' });
    s.ls('.git/rr-cache');
    s.say('Merge thử chỉ để ghi nhớ; bỏ nó, nhánh tính năng giữ nguyên hai commit đã dọn');
    s.git('reset -q --hard HEAD~1');
    s.git('switch -q main');
    checkRefs4(s, refs4(6));
  },
};
