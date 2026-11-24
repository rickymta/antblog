// Chỉ dẫn `#xem LOẠI [ĐỐI_SỐ] NN` của kịch bản: mở một màn hình của SSH Studio, chụp ảnh NN, rồi
// quay lại tab phiên đang gõ. Dùng chung cho p01.mjs và phan.mjs, để thêm ảnh giao diện mà không
// phải viết thao tác riêng cho từng bài.
//
//   #xem files /srv/toinoi 05        khung Files: local ở may-cua-ban, remote ở thư mục đó
//   #xem chia-doi /home/an 06        Chia đôi: terminal ở trên, Files ở dưới
//   #xem song-song /home/an 07       Song song: terminal bên trái, Files bên phải
//   #xem servers 08                  trang Servers (danh sách, trạng thái kết nối)
//   #xem known-hosts 09              trang Known Hosts
//   #xem ssh-keys 10                 trang SSH Keys
//   #xem launchpad 11                Launchpad, tab Phiên đang mở
//   #xem nhat-ky 12                  nhật ký hoạt động (thanh trạng thái → Hoạt động)

export const XEM = /^#xem (files|chia-doi|song-song|servers|known-hosts|ssh-keys|launchpad|nhat-ky)(?: (\S+))? (\d\d)$/;

const VIEW = { files: 'Files', 'chia-doi': 'Chia đôi', 'song-song': 'Song song' };
const NAV = { servers: 'Servers', 'known-hosts': 'Known Hosts', 'ssh-keys': 'SSH Keys', launchpad: 'Launchpad' };

/**
 * @param ctx { a, c, sleep, shot, local } — `a` là đối tượng của app.mjs, `local` là thư mục
 *   "máy của bạn" của lab (khung local của Files luôn trỏ vào đó để ảnh không lộ thư mục riêng).
 */
export async function xem(ctx, loai, doiSo, so) {
  const { a, c, sleep, shot, local } = ctx;
  if (VIEW[loai]) {
    if (!doiSo) throw new Error(`#xem ${loai} cần đường dẫn trên server`);
    await a.view(VIEW[loai]);
    await a.path('local', local);
    await a.path('remote', doiSo);
    await c.click('.file-pane *', 'Remote server', { exact: true }).catch(() => {});
    await sleep(500);
    await shot(so);
    await a.view('Terminal');
    // Chờ terminal nhận lại focus, kẻo lệnh gõ tiếp theo dính vào chữ đang dở.
    await sleep(600);
    await a.focusTerminal();
    await sleep(300);
    return;
  }
  // Trang của ứng dụng: nhớ tab phiên đang chọn để quay lại đúng tab đó.
  const tab = await c.eval(`document.querySelector('.tab.selected')?.innerText.trim() || ''`);
  if (NAV[loai]) {
    await a.nav(NAV[loai]);
    await sleep(700);
    if (loai === 'launchpad') {
      await c.click('.lp-tabs [role=tab]', 'Phiên đang mở').catch(() => {});
      await sleep(500);
    }
    await shot(so);
  } else if (loai === 'nhat-ky') {
    await c.click('button[title^="Nhật ký hoạt động"]');
    await sleep(700);
    await shot(so);
    await c.esc();
    await sleep(400);
  }
  if (tab && loai !== 'nhat-ky') {
    await c.click('.tab', tab, { exact: true }).catch(async () => {
      await c.click('button.session-nav', tab.split('\n')[0]);
    });
    await sleep(700);
  }
}
