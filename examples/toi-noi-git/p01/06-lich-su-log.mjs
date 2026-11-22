// GIT-006 · Đọc lịch sử: log, đồ thị commit và tìm commit

export default {
  id: 'GIT-006',
  start: 9,
  end: 9,
  run(s, { hashes }) {
    s.say('Mặc định: đầy đủ từng commit (giới hạn 2 commit cho gọn)');
    s.git('log -2');
    s.blank();
    s.git('log --oneline');
    s.git('log --oneline --graph --decorate -3');
    s.blank();
    s.say('Tự chọn cột: hash ngắn, tác giả, ngày, summary');
    s.git('log --format="%h %<(9)%an %ad %s" --date=short');
    s.blank();
    s.say('Lọc theo tác giả, khoảng thời gian, đường dẫn, nội dung message');
    s.git('log --oneline --author=Lan');
    s.git('log --oneline --since="2026-11-05T00:00+07:00" --until="2026-11-06T23:59+07:00"');
    s.git('log --oneline -- src/BaoGia/appsettings.json');
    s.git('log --oneline -i --grep="tài liệu"');
    s.blank();
    s.say('Khoảng commit: các commit có trong HEAD nhưng không có trong HEAD~3');
    s.git('log --oneline HEAD~3..HEAD');
    s.blank();
    s.say('Xem một commit cụ thể');
    s.git(`show --stat ${hashes.get('c4').slice(0, 7)}`);
    const lan = s.value('log --format=%h --author=Lan').split('\n');
    s.check('Lan có 3 commit', lan.length === 3);
    s.check('appsettings.json đổi ở 2 commit', s.value('log --format=%h -- src/BaoGia/appsettings.json').split('\n').length === 2);
  },
};
