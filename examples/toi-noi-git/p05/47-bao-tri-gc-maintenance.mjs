// GIT-047 · Bảo trì: count-objects, gc, fsck, commit-graph, maintenance

import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { checkRefs5, P } from './chung.mjs';

/** Một luồng nén để kích thước pack giống nhau trên mọi máy (lab ẩn tuỳ chọn này khỏi lệnh in ra). */
const ONE_THREAD = { env: { GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'pack.threads', GIT_CONFIG_VALUE_0: '1' } };

export default {
  id: 'GIT-047',
  start: 5,
  end: 5,
  run(s, { refs5 }) {
    s.say('Sau bốn phần, kho object toàn object rời (loose): mỗi object một file nén');
    s.git('count-objects -v');
    s.blank();
    s.say('fsck: kiểm tra mọi object. dangling = object không còn ref nào trỏ tới (commit thử, reset, rebase ở Phần 4)');
    const fsck = s.git('fsck --full --no-progress', { quiet: true }).stdout.split('\n').filter(Boolean);
    s.print('$ git fsck --full --no-progress');
    const kinds = {};
    for (const l of fsck) kinds[l.split(' ').slice(0, 2).join(' ')] = (kinds[l.split(' ').slice(0, 2).join(' ')] ?? 0) + 1;
    for (const [k, n] of Object.entries(kinds)) s.print(`${k} … (${n} dòng)`);
    s.blank();
    s.say('gc: đóng gói object rời thành pack, nén delta, đóng gói ref; object không còn tham chiếu được giữ 2 tuần rồi mới xoá');
    s.git('gc -q', ONE_THREAD);
    s.git('count-objects -v');
    s.git('fsck --full --no-progress --no-dangling');
    s.blank();
    s.say('commit-graph: chỉ mục giúp log, đồ thị, blame, merge-base chạy nhanh trên lịch sử dài');
    s.git('commit-graph write --reachable --changed-paths');
    s.git('commit-graph verify');
    s.check('đã có commit-graph', existsSync(join(P.minh, '.git', 'objects', 'info', 'commit-graph')) || existsSync(join(P.minh, '.git', 'objects', 'info', 'commit-graphs')));
    s.blank();
    s.say('maintenance run: chạy từng việc bảo trì theo tên (git maintenance start sẽ đặt lịch chạy định kỳ)');
    s.git('maintenance run --task=pack-refs');
    s.git('maintenance run --task=commit-graph');
    s.say('Dọn object LFS cũ đã có trên remote (xem trước)');
    s.git('lfs prune --dry-run');
    checkRefs5(s, refs5(5));
  },
};
