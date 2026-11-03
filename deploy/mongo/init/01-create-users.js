// Chạy MỘT LẦN bởi ảnh mongo chính thức khi volume dữ liệu còn trống (docker-entrypoint-initdb.d).
// Tài khoản root (MONGO_INITDB_ROOT_*) do chính ảnh tạo; script này tạo thêm:
//   - tài khoản ứng dụng: readWrite trên database của blog — API dùng;
//   - tài khoản kiểm tra: CHỈ ĐỌC — dùng khi kết nối qua VPN để xem dữ liệu (Compass, mongosh).
// Đổi mật khẩu về sau phải làm bằng mongosh (db.changeUserPassword), sửa .env không có tác dụng với volume cũ.

const dbName = process.env.MONGO_APP_DATABASE || 'tech_knowledge_hub';
const blog = db.getSiblingDB(dbName);

function ensureUser(user, pwd, roles) {
  if (!user || !pwd) {
    print(`Bỏ qua tạo user: thiếu tên hoặc mật khẩu (${user || '?'})`);
    return;
  }
  if (blog.getUser(user)) {
    print(`User ${user} đã có trên ${dbName}`);
    return;
  }
  blog.createUser({ user, pwd, roles });
  print(`Đã tạo user ${user} trên ${dbName} với quyền ${roles.map((r) => r.role).join(', ')}`);
}

ensureUser(process.env.MONGO_APP_USERNAME, process.env.MONGO_APP_PASSWORD, [{ role: 'readWrite', db: dbName }]);
ensureUser(process.env.MONGO_INSPECTOR_USERNAME, process.env.MONGO_INSPECTOR_PASSWORD, [{ role: 'read', db: dbName }]);
