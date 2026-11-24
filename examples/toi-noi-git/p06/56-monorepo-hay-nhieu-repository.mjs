// GIT-056 · Monorepo hay nhiều repository

import { ADR_REPOS, CODEOWNERS } from '../lib/history6.mjs';
import { checkRefs6 } from './chung.mjs';

const M = (t) => ({ as: 'minh', at: `2026-12-03T${t}:00+07:00` });

export default {
  id: 'GIT-056',
  start: 5,
  end: 6,
  run(s, { refs6 }) {
    s.say('Dữ liệu cho quyết định: thay đổi từ 1.2.0 rơi vào thư mục nào, ai viết');
    s.git('diff --dirstat=files,5 v1.2.0 main');
    s.git('shortlog -sn --no-merges v1.2.0..main');
    s.blank();
    s.say('Hai repository có từng chung một commit nào không? (đổi cả hai cùng lúc là dấu hiệu nên gộp)');
    s.git('-C ../app-tai-xe log --format="%ad %s" --date=short');
    s.git('log --format="%ad %s" --date=short --since="2026-12-02T00:00+07:00" --until="2026-12-02T23:59+07:00" main');
    s.blank();
    s.say('CODEOWNERS: ai phải review thay đổi ở từng khu vực');
    s.write('.github/CODEOWNERS', CODEOWNERS);
    s.git('add .github/CODEOWNERS');
    s.git('commit -q -m "chore: CODEOWNERS cho từng khu vực"', M('14:00'));
    s.blank();
    s.say('Ghi quyết định (ADR) cạnh code để người sau biết vì sao');
    s.write('docs/quyet-dinh/0001-hai-repository.md', ADR_REPOS);
    s.git('add docs/quyet-dinh');
    s.git('commit -q -m "docs(quyet-dinh): giữ hai repository, gộp thư viện tiền tệ vào báo giá"', M('14:30'));
    s.git('push -q');
    s.git('log --oneline -3');
    s.check('có CODEOWNERS và ADR', s.exists('.github/CODEOWNERS') && s.exists('docs/quyet-dinh/0001-hai-repository.md'));
    checkRefs6(s, refs6(6));
  },
};
