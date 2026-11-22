// GIT-016 · Tag và phiên bản phát hành

import { TAG_MSG } from '../lib/history2.mjs';
import { checkRefs } from './chung.mjs';

const at = (t) => ({ as: 'minh', at: `2026-11-17T${t}:00+07:00` });

export default {
  id: 'GIT-016',
  start: 7,
  end: 8,
  run(s, { refs2, hashes }) {
    s.git('tag');
    s.blank();
    s.say('Lightweight tag: chỉ là một cái tên trỏ thẳng vào commit');
    s.git('tag thu-nghiem');
    s.git('cat-file -t thu-nghiem');
    s.git('tag -d thu-nghiem');
    s.blank();
    s.say('Annotated tag: một object riêng có người tạo, thời điểm, lời nhắn');
    s.git(`tag -a v1.0.0 -m "${TAG_MSG['v1.0.0']}" ${hashes.get('c10').slice(0, 7)}`, at('11:00'));
    s.git(`tag -a v1.1.0 -m "${TAG_MSG['v1.1.0']}"`, at('11:05'));
    s.git('tag -n');
    s.git('cat-file -t v1.1.0');
    s.git('cat-file -p v1.1.0');
    s.check('v1.0.0 trỏ vào commit cuối Phần 1', s.value('rev-parse v1.0.0^{commit}') === hashes.get('c10'));
    s.blank();
    s.say('Mô tả một commit bằng tag gần nhất');
    s.git('describe');
    s.git('describe HEAD~1');
    s.git('log --oneline --decorate -2');
    checkRefs(s, refs2(8));
  },
};
