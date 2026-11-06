-- DMS KHÔNG di chuyển giá trị hiện tại của sequence/identity. Chạy trên ĐÍCH trong cửa sổ cutover,
-- SAU khi dừng ghi ở nguồn và TRƯỚC khi mở ghi vào đích, nếu không INSERT mới sẽ đụng khoá đã có.

-- trips.code là identity (bắt đầu từ 100000). Đặt sequence vượt max hiện có, cộng khoảng đệm an toàn.
SELECT setval(
  pg_get_serial_sequence('public.trips', 'code'),
  (SELECT coalesce(max(code), 100000) FROM public.trips) + 1000,
  true
);

-- trip_events.id là identity.
SELECT setval(
  pg_get_serial_sequence('public.trip_events', 'id'),
  (SELECT coalesce(max(id), 1) FROM public.trip_events) + 1000,
  true
);

-- Kiểm: giá trị kế tiếp của mỗi sequence phải lớn hơn max của cột tương ứng.
SELECT sequencename, last_value FROM pg_sequences WHERE schemaname = 'public' ORDER BY sequencename;
