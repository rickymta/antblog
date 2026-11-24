// GIT-048 · Ký commit và tag bằng khoá SSH

import { ALLOWED_SIGNERS, SIGN_KEY } from '../lib/history5.mjs';
import { LAB, readText } from '../lib/lab.mjs';
import { checkRefs5 } from './chung.mjs';

const KEY_SHOW = '<lab>/fixtures/khoa-ky-demo.pub';
const MINH = (t) => ({ as: 'minh', at: `2026-11-28T${t}:00+07:00` });

export default {
  id: 'GIT-048',
  start: 5,
  end: 6,
  run(s, { refs5 }) {
    s.say('Ký bằng khoá SSH: định dạng ssh, khoá ký là public key (private key nằm cạnh nó hoặc trong ssh-agent)');
    s.git('config gpg.format ssh');
    s.git(`config user.signingkey ${SIGN_KEY}.pub`, { show: `config user.signingkey ${KEY_SHOW}` });
    s.blank();
    s.say('Danh sách người ký tin cậy, commit cùng repository để cả nhóm xác minh được');
    s.write('.allowed_signers', ALLOWED_SIGNERS(readText(LAB, 'fixtures/khoa-ky-demo.pub')));
    s.print('$ Get-Content .allowed_signers');
    s.print(s.read('.allowed_signers').trimEnd());
    s.git('config gpg.ssh.allowedSignersFile .allowed_signers');
    s.git('add .allowed_signers');
    s.git('commit -S -m "Thêm danh sách khoá ký của nhóm"', MINH('09:00'));
    s.git('log --show-signature -1');
    s.git('verify-commit HEAD');
    s.say('Commit cũ không ký: verify-commit không in gì và thoát mã 1');
    s.git('verify-commit HEAD~1', { expectFail: true });
    s.blank();
    s.say('Tag phát hành có chữ ký');
    s.git('tag -s v1.2.0 -m "Phát hành 1.2.0: phụ phí đêm, phí chờ theo giây"', MINH('09:30'));
    s.git('tag -v v1.2.0');
    s.git('push -q origin main v1.2.0');
    s.check('tag v1.2.0 có chữ ký hợp lệ', s.git('tag -v v1.2.0', { quiet: true }).status === 0);
    checkRefs5(s, refs5(6));
  },
};
