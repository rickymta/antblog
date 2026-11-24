// GIT-043 · .gitattributes: line ending, diff và merge theo loại file

import { GITATTRIBUTES_P05 } from '../lib/history5.mjs';
import { checkRefs5 } from './chung.mjs';

export default {
  id: 'GIT-043',
  start: 2,
  end: 3,
  run(s, { refs5 }) {
    s.say('.gitattributes hiện có (từ GIT-007): xuống dòng và file nhị phân');
    s.print('$ Get-Content .gitattributes');
    s.print(s.read('.gitattributes').trimEnd());
    s.git('check-attr -a -- src/BaoGia/FareCalculator.cs build.cmd docs/logo.png');
    s.git('ls-files --eol -- README.md src/BaoGia/FareCalculator.cs');
    s.blank();
    s.say('Sửa một dòng giữa hàm Quote: tiêu đề hunk mặc định chỉ đoán theo dòng không thụt lề');
    const original = s.read('src/BaoGia/FareCalculator.cs');
    s.write('src/BaoGia/FareCalculator.cs', original.replace('        // Chưa gồm phí cầu đường, khuyến mãi.', '        // Chưa gồm phí cầu đường, khuyến mãi, phụ phí đêm.'));
    s.git('diff -U1');
    s.blank();
    s.say('Khai báo diff=csharp (và thêm vài quy tắc khác): tiêu đề hunk là tên hàm');
    s.write('.gitattributes', GITATTRIBUTES_P05(s.read('.gitattributes')));
    s.git('diff -U1 -- src');
    s.write('src/BaoGia/FareCalculator.cs', original);
    s.git('check-attr diff merge -- src/BaoGia/FareCalculator.cs CHANGELOG.md docs/bang-gia.xlsx');
    s.blank();
    s.say('Sau khi đổi quy tắc xuống dòng: chuẩn hoá lại file đã commit (ở đây không có gì cần đổi)');
    s.git('add --renormalize .');
    s.git('status -s');
    s.git('commit -q -am "Khai báo diff và merge theo loại file"', { as: 'minh', at: '2026-11-27T09:00:00+07:00' });
    s.git('push -q');
    checkRefs5(s, refs5(3));
  },
};
