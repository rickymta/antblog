// Consumer SQS: nhận batch, xử lý từng message, chỉ báo lỗi cho message hỏng
// (partial batch response, cần bật ReportBatchItemFailures trên event source mapping).
export const handler = async (event) => {
  const batchItemFailures = [];

  for (const record of event.Records) {
    try {
      const payload = JSON.parse(record.body);
      await applyPaymentEvent(payload);
    } catch (err) {
      console.error(JSON.stringify({ msg: 'consumer failed', messageId: record.messageId, error: String(err) }));
      batchItemFailures.push({ itemIdentifier: record.messageId });
    }
  }
  return { batchItemFailures };
};

// Ở hệ thống thật: INSERT vào bảng payment_events với khoá duy nhất eventId,
// gặp trùng thì bỏ qua (xem bài Outbox và Inbox). Lab chỉ ghi log.
async function applyPaymentEvent(p) {
  console.log(JSON.stringify({ msg: 'payment event', eventId: p.eventId, type: p.type, tripId: p.tripId }));
}
