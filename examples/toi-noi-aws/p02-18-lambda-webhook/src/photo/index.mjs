// Trigger S3 (gọi bất đồng bộ): ảnh chứng từ giao hàng vừa được tải lên.
// Key trong sự kiện S3 bị mã hoá URL (dấu cách thành "+"), phải giải mã trước khi dùng.
export const handler = async (event) => {
  for (const record of event.Records) {
    const bucket = record.s3.bucket.name;
    const key = decodeURIComponent(record.s3.object.key.replace(/\+/g, ' '));
    console.log(JSON.stringify({ msg: 'delivery photo uploaded', bucket, key, size: record.s3.object.size }));
    // Việc thật: tạo thumbnail, ghi metadata vào bảng trips, ... Ghi vào CHÍNH prefix photos/
    // sẽ kích hoạt lại hàm này (vòng lặp); ghi kết quả sang prefix hoặc bucket khác.
  }
};
