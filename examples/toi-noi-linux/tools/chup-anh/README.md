# Chụp ảnh SSH Studio cho series Linux

Ảnh trong bài là SSH Studio thật, chạy từ repository `ssh-studio`, kết nối tới server lab. Công cụ ở đây điều khiển cửa sổ đó qua cổng debug của WebView2 (Chrome DevTools Protocol): gõ **đúng từng lệnh của kịch bản bài** vào terminal, thao tác trên Files / trình sửa file / hộp thoại ở các dòng `#gui`, và chụp ở các dòng `#shot`. Vì SSH Studio là bên thực hiện mọi lệnh, sau khi chụp xong bộ chấm của phần đó (`npm run kiem-tra`, `kiem-tra:p02` … `kiem-tra:p06`) phải cho toàn PASS.

## Chạy

1. Mở SSH Studio từ repository với thư mục dữ liệu riêng và tỉ lệ hiển thị 1,5 (PowerShell):

   ```powershell
   # chạy trong thư mục repository ssh-studio; thay <tech-blog> bằng đường dẫn repository này
   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9344 --force-device-scale-factor=1.5"
   npx tauri dev --config <tech-blog>\examples\toi-noi-linux\tools\chup-anh\tauri.chup-anh.conf.json
   ```

   `tauri.chup-anh.conf.json` đổi `identifier` thành `dev.sshstudio.blog` (workspace nằm ở `%APPDATA%\dev.sshstudio.blog`, tách khỏi SSH Studio đang dùng hằng ngày) và mở cửa sổ không viền 2400×1350 pixel = **1600×900 điểm CSS ở tỉ lệ 1,5**. Ảnh chụp ra 2400×1350. Cổng debug mặc định của công cụ là 9344; đổi bằng biến `CDP_PORT`.

   Không giả lập tỉ lệ bằng DevTools (`Emulation.setDeviceMetricsOverride`): terminal vẽ bằng WebGL và sẽ vẽ chữ sai cỡ. Phải dùng tỉ lệ thật của WebView2 như trên.

2. Để chụp từ đầu (bài LNX-002 cần workspace trống): tắt cửa sổ, xoá `%APPDATA%\dev.sshstudio.blog\workspace.json`, mở lại.

3. Chụp:

   ```sh
   cd examples/toi-noi-linux
   node tools/chup-anh/p01.mjs          # tạo lại server lab, chụp cả Phần 1
   node tools/chup-anh/p01.mjs 09 10    # chỉ vài bài; server phải đang ở đầu bài 09
   npm run kiem-tra

   node tools/chup-anh/p02.mjs          # đưa server về đầu LNX-011 (setup 11) rồi chụp cả Phần 2
   npm run kiem-tra:p02

   node tools/chup-anh/p03.mjs          # đưa server về đầu LNX-021 rồi chụp cả Phần 3
   npm run kiem-tra:p03

   node tools/chup-anh/p04.mjs          # đưa server về đầu LNX-031 rồi chụp cả Phần 4
   npm run kiem-tra:p04

   node tools/chup-anh/p05.mjs          # đưa server về đầu LNX-041 rồi chụp cả Phần 5
   npm run kiem-tra:p05

   node tools/chup-anh/p06.mjs          # đưa lab về đầu LNX-051 rồi chụp cả Phần 6
   npm run kiem-tra:p06
   ```

   Ảnh ghi vào `apps/web/public/images/articles/<slug>/ssh-studio-NN.webp` (WebP, chất lượng 90, dùng `sharp` của repo).

## Lưu ý

- Khung local của Files luôn được trỏ vào `examples/toi-noi-linux/may-cua-ban` trước khi chụp, để ảnh không lộ thư mục cá nhân.
- Mật khẩu gõ vào hộp thoại là mật khẩu của lab (`toinoi-lab`); hộp thoại hiện dấu chấm.
- Master key của vault vẫn nằm trong kho khoá của hệ điều hành dưới service `dev.sshstudio.desktop` (tên service cố định trong mã nguồn), dùng chung với bản SSH Studio đã cài. Công cụ chỉ đọc, không ghi đè.
- Từ Phần 2, các dòng `#file` được gõ thật bằng vim (`:set paste`, gõ từng dòng, `:wq`) rồi đối chiếu SHA-256 với file mẫu trong `pNN/files/`; `#check` đối chiếu sau một thao tác `#gui`. Nội dung lệch là công cụ dừng ngay.
- `cdp.mjs`: client CDP tối thiểu. `app.mjs`: thao tác mức người dùng (kết nối, gõ lệnh, khung Files). `p01.mjs`: trình tự của Phần 1. `phan.mjs`: trình phát kịch bản dùng chung từ Phần 2. `p02.mjs` … `p06.mjs`: thao tác giao diện của Phần 2–6.
- Sau mỗi lệnh, trình phát đợi tới khi shell của phiên SSH rảnh (nhóm tiến trình foreground của terminal là chính shell), nên lệnh chạy lâu như `apt install` hay `timeout 4 journalctl -f` xong rồi mới gõ lệnh sau hay chụp ảnh. Job chạy nền không tính.
- Từ Phần 4, mỗi lần kết nối cần key và mã OTP (Phần 3). `p04.mjs` quên host key cũ nếu ảnh lab được dựng lại (fingerprint đổi), đợi 35 giây sau setup vì `RATE_LIMIT 3 30` của mã OTP, và với mỗi lần kết nối đợi sang khung 30 giây mới. Các dòng `#gui ngat-ket-noi`, `ket-noi-lai` đóng hẳn tab phiên (shell trên server nhận SIGHUP thật) rồi kết nối lại.
- Phần 5: các đoạn `#may app02` của kịch bản chạy ngầm bằng runner trên `toinoi-app-02` (không gõ vào SSH Studio), đúng thứ tự với các lệnh gõ trong terminal. LNX-044 mở đường hầm Local 18080 → 127.0.0.1:8080 trong tab Ports, gọi `curl.exe` từ Windows qua đường hầm và ghi kết quả vào `p05/tu-may-ban.txt` để bài viết dẫn lại. Nút chế độ xem có số đếm (`Ports 1` khi có đường hầm chạy) vẫn được tìm theo dòng đầu.
- Phần 6: chụp từ đầu phần thì `p06.mjs` đưa SSH Studio về trạng thái cuối Phần 5 (xoá hồ sơ bastion/app02, thư mục, host key của chúng; nhóm của server về «Lab»). Dòng `#phien TÊN` chuyển sang tab SSH Studio của máy đó, và trình phát kiểm tra trạng thái (shell rảnh, sudo đang chờ, đối chiếu file) trên đúng container của máy đó. LNX-053 lưu và mở file sao lưu qua hộp thoại của Windows bằng `chon-file.ps1` (file ở `may-cua-ban/sao-luu/`, không đưa vào Git). LNX-059 chờ tới gần 3 phút cho lỗi hết thời gian của jump host.
- Phần 3 bắt đầu bằng việc đưa SSH Studio về trạng thái đầu phần: server dùng mật khẩu, không có key nào trong thư viện, không có hồ sơ của `cuong`, Known Hosts chỉ còn host key gốc. Các dòng `#sudofile` được gõ bằng vim mở qua `sudoedit`/`visudo`, mật khẩu sudo được trả lời khi `sudo` đang chờ.
- Nhập key ở LNX-022 dùng hộp thoại chọn file thật của Windows: `chon-file.ps1` tìm hộp thoại của tiến trình `ssh-studio` và **dán** đường dẫn của `may-cua-ban/khoa-lab/an-laptop` qua clipboard (không gõ từng phím, để bộ gõ tiếng Việt đang bật không làm sai ký tự). `chon-file.ps1 -Cancel` đóng hộp thoại còn sót.
- LNX-028 thử phương thức Agent khi dịch vụ `ssh-agent` của Windows đang tắt; công cụ không bật dịch vụ đó và không đổi cấu hình Windows.
