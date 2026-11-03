// Đặt biến môi trường TRƯỚC khi mã nguồn đọc cấu hình. Test luôn dùng DB riêng và tự dọn.
process.env.NODE_ENV = 'test';
process.env.MONGODB_URI = process.env.TEST_MONGODB_URI ?? 'mongodb://127.0.0.1:27017/tech_knowledge_hub_test';
process.env.JWT_SECRET = 'test-secret-only-for-automated-tests-0123456789abcdef0123456789';
