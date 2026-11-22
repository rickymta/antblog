// GIT-009 · Truy vết thay đổi: blame, lịch sử file và tìm theo nội dung

export default {
  id: 'GIT-009',
  start: 10,
  end: 10,
  run(s, { hashes }) {
    s.say('Câu hỏi của bộ phận vận hành: ai đổi đơn giá mỗi km của ô tô 4 chỗ, khi nào, vì sao?');
    s.git('blame -L 3,7 src/BaoGia/appsettings.json');
    s.blank();
    s.say('Đọc commit được blame chỉ ra: description trả lời "vì sao"');
    s.git(`show ${hashes.get('c5').slice(0, 7)}`);
    s.blank();
    s.say('Pickaxe: commit nào làm THAY ĐỔI số lần xuất hiện của chuỗi');
    s.git("log -S'4500,' --oneline");
    s.git(`log -G'Math\.Ceiling' --oneline`);
    s.blank();
    s.say('Lịch sử của một khoảng dòng: thân hàm Quote');
    s.git('log -L 10,25:src/BaoGia/FareCalculator.cs --format="%h %an %s" --no-patch');
    s.blank();
    s.say('File đã đổi tên: không có --follow, lịch sử dừng ở lần đổi tên');
    s.git('log --oneline -- docs/bang-gia-cuoc.md');
    s.git('log --oneline --follow -- docs/bang-gia-cuoc.md');
    s.git('log --oneline --follow --name-status -- docs/bang-gia-cuoc.md');
    s.blank();
    s.say('Blame trên file code: dòng điều kiện từ c2, câu lệnh throw từ c6');
    s.git('blame -w -L 11,13 src/BaoGia/FareCalculator.cs');
    const pick = s.value("log -S'4500,' --format=%H").split('\n');
    s.check('pickaxe tìm được commit thêm và commit đổi 4500', pick.length === 2 && pick[0] === hashes.get('c5') && pick[1] === hashes.get('c2'));
    s.check('--follow thấy 2 commit của tài liệu', s.value('log --format=%h --follow -- docs/bang-gia-cuoc.md').split('\n').length === 2);
    const blame = s.value('blame --porcelain -L 5,5 src/BaoGia/appsettings.json').split(' ')[0];
    s.check('blame dòng PerKm trỏ về commit của Lan (c5)', blame === hashes.get('c5'));
  },
};
