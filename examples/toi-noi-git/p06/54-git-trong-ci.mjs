// GIT-054 · Git trong CI: clone nông, tag, phiên bản và quyền

import { CI_YML, PHIEN_BAN_SH } from '../lib/history6.mjs';
import { checkRefs6, NO_LFS, REMOTE_SHOW, REMOTE_URL, sh } from './chung.mjs';

export default {
  id: 'GIT-054',
  start: 3,
  end: 4,
  run(s, { refs6 }) {
    s.say('Workflow CI và script tính phiên bản từ tag');
    s.write('.github/workflows/ci.yml', CI_YML);
    s.write('scripts/phien-ban.sh', PHIEN_BAN_SH);
    s.git('add .github scripts/phien-ban.sh');
    s.git('update-index --chmod=+x scripts/phien-ban.sh');
    s.git('commit -q -m "ci: build, test và tính phiên bản từ tag trên mỗi push"', { as: 'minh', at: '2026-12-02T14:00:00+07:00' });
    s.git('push -q');
    sh(s, ['scripts/phien-ban.sh']);
    s.blank();
    s.say('Máy CI clone như actions/checkout mặc định (fetch-depth: 1): một commit, không tag');
    s.cd('..');
    s.git(`clone -q --depth 1 ${REMOTE_URL} ci-nong`, { ...NO_LFS, show: `clone -q --depth 1 ${REMOTE_SHOW} ci-nong` });
    s.cd('ci-nong');
    s.git('log --oneline');
    s.git('tag -l');
    sh(s, ['scripts/phien-ban.sh'], { expectFail: true });
    s.blank();
    s.say('Sửa: lấy đủ lịch sử và tag (tương đương fetch-depth: 0)');
    s.git('fetch -q --unshallow --tags', NO_LFS);
    sh(s, ['scripts/phien-ban.sh']);
    s.cd('..');
    s.blank();
    s.say('Hoặc partial clone: đủ commit và tag, nội dung file cũ chỉ tải khi cần (GIT-046).');
    s.say('Máy chủ phải cho phép lọc object (GitHub, GitLab bật sẵn; remote của lab bật bằng lệnh dưới)');
    s.git('-C remote/bao-gia.git config uploadpack.allowFilter true');
    s.git(`clone -q --filter=blob:none ${REMOTE_URL} ci-nhe`, { ...NO_LFS, show: `clone -q --filter=blob:none ${REMOTE_SHOW} ci-nhe` });
    s.cd('ci-nhe');
    s.git('describe --tags --match "v[0-9]*"');
    s.say('Job chạy cho tag (on: push: tags) thì đứng đúng commit của tag: phiên bản không có hậu tố dev');
    s.git('describe --tags --match "v[0-9]*" v1.3.1');
    s.cd('../bao-gia');
    s.check('phiên bản của main là bản dev sau 1.3.1', /^1\.3\.2-dev\.\d+\+g[0-9a-f]{7}$/.test(s.value('describe --tags --match "v[0-9]*" --long').replace(/^v1\.3\.1-(\d+)-g([0-9a-f]+)$/, '1.3.2-dev.$1+g$2')));
    checkRefs6(s, refs6(4));
  },
};
