# Chụp ảnh SSH Studio cho series Linux

Ảnh trong bài là SSH Studio thật, chạy từ repository `ssh-studio`, kết nối tới server lab. Công cụ ở đây điều khiển cửa sổ đó qua cổng debug của WebView2 (Chrome DevTools Protocol): gõ **đúng từng lệnh của kịch bản bài** vào terminal, thao tác trên Files / trình sửa file / hộp thoại ở các dòng `#gui`, và chụp ở các dòng `#shot`. Vì SSH Studio là bên thực hiện mọi lệnh, sau khi chụp xong `npm run kiem-tra` phải cho 15 PASS.

## Chạy

1. Mở SSH Studio từ repository với thư mục dữ liệu riêng và tỉ lệ hiển thị 1,5 (PowerShell):

   ```powershell
   # chạy trong thư mục repository ssh-studio; thay <tech-blog> bằng đường dẫn repository này
   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9333 --force-device-scale-factor=1.5"
   npx tauri dev --config <tech-blog>\examples\toi-noi-linux\tools\chup-anh\tauri.chup-anh.conf.json
   ```

   `tauri.chup-anh.conf.json` đổi `identifier` thành `dev.sshstudio.blog` (workspace nằm ở `%APPDATA%\dev.sshstudio.blog`, tách khỏi SSH Studio đang dùng hằng ngày) và mở cửa sổ không viền 2400×1350 pixel = **1600×900 điểm CSS ở tỉ lệ 1,5**. Ảnh chụp ra 2400×1350.

   Không giả lập tỉ lệ bằng DevTools (`Emulation.setDeviceMetricsOverride`): terminal vẽ bằng WebGL và sẽ vẽ chữ sai cỡ. Phải dùng tỉ lệ thật của WebView2 như trên.

2. Để chụp từ đầu (bài LNX-002 cần workspace trống): tắt cửa sổ, xoá `%APPDATA%\dev.sshstudio.blog\workspace.json`, mở lại.

3. Chụp:

   ```sh
   cd examples/toi-noi-linux
   node tools/chup-anh/p01.mjs          # tạo lại server lab, chụp cả Phần 1
   node tools/chup-anh/p01.mjs 09 10    # chỉ vài bài; server phải đang ở đầu bài 09
   npm run kiem-tra
   ```

   Ảnh ghi vào `apps/web/public/images/articles/<slug>/ssh-studio-NN.webp` (WebP, chất lượng 90, dùng `sharp` của repo).

## Lưu ý

- Khung local của Files luôn được trỏ vào `examples/toi-noi-linux/may-cua-ban` trước khi chụp, để ảnh không lộ thư mục cá nhân.
- Mật khẩu gõ vào hộp thoại là mật khẩu của lab (`toinoi-lab`); hộp thoại hiện dấu chấm.
- Master key của vault vẫn nằm trong kho khoá của hệ điều hành dưới service `dev.sshstudio.desktop` (tên service cố định trong mã nguồn), dùng chung với bản SSH Studio đã cài. Công cụ chỉ đọc, không ghi đè.
- `cdp.mjs`: client CDP tối thiểu. `app.mjs`: thao tác mức người dùng (kết nối, gõ lệnh, khung Files). `p01.mjs`: trình tự của Phần 1.
