// GIT-046 · Repository lớn: partial clone, shallow clone, sparse checkout

import { checkRefs5, REMOTE_SHOW, REMOTE_URL } from './chung.mjs';

/** Clone trong bài này không tải file LFS (GIT-045) để tập trung vào object Git. */
const NO_LFS = { env: { GIT_LFS_SKIP_SMUDGE: '1', GIT_CONFIG_COUNT: '1', GIT_CONFIG_KEY_0: 'pack.threads', GIT_CONFIG_VALUE_0: '1' } };
const missingBlobs = (s) =>
  s
    .value('rev-list --objects --all --missing=print')
    .split('\n')
    .filter((l) => l.startsWith('?')).length;

export default {
  id: 'GIT-046',
  start: 5,
  end: 5,
  run(s, { refs5 }) {
    s.say('Máy chủ phải cho phép lọc object khi clone (GitHub, GitLab bật sẵn)');
    s.cd('../remote/bao-gia.git', { show: false });
    s.git('config uploadpack.allowFilter true');
    s.cd('../..', { show: false });
    s.blank();
    s.say('Clone đủ: mọi commit, mọi phiên bản của mọi file');
    s.git(`clone -q ${REMOTE_URL} clone-du`, { ...NO_LFS, show: `clone -q ${REMOTE_SHOW} clone-du` });
    s.cd('clone-du');
    s.git('count-objects -v');
    s.cd('..');
    s.blank();
    s.say('Partial clone --filter=blob:none: đủ commit và tree, blob (nội dung file) chỉ tải khi cần');
    s.git(`clone -q --filter=blob:none ${REMOTE_URL} clone-nhe`, { ...NO_LFS, show: `clone -q --filter=blob:none ${REMOTE_SHOW} clone-nhe` });
    s.cd('clone-nhe');
    s.git('rev-list --count HEAD');
    s.git('count-objects -v');
    s.print('$ git rev-list --objects --all --missing=print | Select-String "^[?]" | Measure-Object | % Count');
    s.print(String(missingBlobs(s)));
    s.say('Xem một file ở bản cũ: Git tải đúng blob đó từ remote rồi in ra');
    const old = s.git('show v1.0.0:src/BaoGia/FareCalculator.cs', { ...NO_LFS, quiet: true }).stdout;
    s.print('$ git show v1.0.0:src/BaoGia/FareCalculator.cs | Select-Object -First 3');
    s.print(old.split('\n').slice(0, 3).join('\n'));
    s.print('$ git rev-list --objects --all --missing=print | Select-String "^[?]" | Measure-Object | % Count');
    s.print(String(missingBlobs(s)));
    s.cd('..');
    s.blank();
    s.say('Shallow clone --depth 1: chỉ commit mới nhất (hợp với CI chỉ cần build)');
    s.git(`clone -q --depth 1 ${REMOTE_URL} clone-nong`, { ...NO_LFS, show: `clone -q --depth 1 ${REMOTE_SHOW} clone-nong` });
    s.cd('clone-nong');
    s.git('log --oneline');
    s.git('rev-parse --is-shallow-repository');
    s.git('fetch -q --deepen 2', NO_LFS);
    s.git('log --oneline');
    s.cd('..');
    s.blank();
    s.say('Sparse checkout: chỉ đưa vài thư mục ra working tree (kết hợp partial clone)');
    s.git(`clone -q --sparse --filter=blob:none ${REMOTE_URL} clone-thua`, { ...NO_LFS, show: `clone -q --sparse --filter=blob:none ${REMOTE_SHOW} clone-thua` });
    s.cd('clone-thua');
    s.ls('.');
    s.git('sparse-checkout set docs', NO_LFS);
    s.git('sparse-checkout list');
    s.ls('.');
    s.ls('docs');
    s.cd('../bao-gia', { show: false });
    checkRefs5(s, refs5(5));
  },
};
