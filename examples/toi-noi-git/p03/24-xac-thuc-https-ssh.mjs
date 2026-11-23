// GIT-024 · Xác thực: HTTPS token, credential helper và SSH key
// Lab không có máy chủ thật; bài cho thấy git chọn cách xác thực theo URL, và cấu hình liên quan.

import { checkRefs3, P } from './chung.mjs';

export default {
  id: 'GIT-024',
  start: 3,
  end: 3,
  run(s, { refs3 }) {
    s.say('Cùng một repository trên GitHub, hai dạng URL, hai cách xác thực');
    s.git('remote add github https://github.com/toinoi/bao-gia.git');
    s.git('remote -v');
    s.git('remote set-url github git@github.com:toinoi/bao-gia.git');
    s.git('remote get-url github');
    s.git('remote remove github');
    s.blank();
    s.say('Credential helper: lab không đặt, nên git sẽ hỏi (và lab tắt việc hỏi)');
    s.git('config --show-scope --get credential.helper', { expectFail: true });
    s.git('config --get core.sshCommand', { expectFail: true });
    s.blank();
    s.say('Viết tắt URL bằng insteadOf: toinoi:<repo> thay cho đường dẫn đầy đủ');
    const base = P.remote.replace(/bao-gia\.git$/, '');
    s.git(`config url.${base}.insteadOf toinoi:`, { show: 'config url.<lab>/work/remote/.insteadOf toinoi:' });
    s.git('ls-remote --heads toinoi:bao-gia.git');
    s.git(`config --remove-section url.${base}`, { show: 'config --remove-section url.<lab>/work/remote/' });
    s.check('đã gỡ cấu hình insteadOf', s.git('config --get-regexp ^url', { quiet: true, expectFail: true }).stdout === '');
    checkRefs3(s, refs3(3));
  },
};
