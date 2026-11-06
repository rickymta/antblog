import { SQSClient, SendMessageCommand } from '@aws-sdk/client-sqs';
import { SecretsManagerClient, GetSecretValueCommand } from '@aws-sdk/client-secrets-manager';
import { verifySignature } from './verify.mjs';

// Khởi tạo ngoài handler: chạy một lần ở cold start, dùng lại ở các lần invoke sau
// trong cùng execution environment.
const sqs = new SQSClient({});
const secrets = new SecretsManagerClient({});
let cachedSecret;

async function getSecret() {
  if (!cachedSecret) {
    const out = await secrets.send(new GetSecretValueCommand({ SecretId: process.env.SECRET_ID }));
    cachedSecret = out.SecretString;
  }
  return cachedSecret;
}

const reply = (statusCode, body) => ({
  statusCode,
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify(body),
});

// Function URL (payload format 2.0): header đã viết thường, body là chuỗi.
export const handler = async (event) => {
  const rawBody = event.isBase64Encoded
    ? Buffer.from(event.body ?? '', 'base64').toString('utf8')
    : (event.body ?? '');

  const check = verifySignature({
    header: event.headers?.['x-vixanh-signature'],
    rawBody,
    secret: await getSecret(),
  });
  if (!check.ok) {
    // Không log body: có thể chứa dữ liệu thanh toán.
    console.warn(JSON.stringify({ msg: 'webhook rejected', reason: check.reason }));
    return reply(401, { error: 'invalid-signature' });
  }

  let payload;
  try {
    payload = JSON.parse(rawBody);
  } catch {
    return reply(400, { error: 'invalid-json' });
  }
  if (!payload.eventId || !payload.type) {
    return reply(400, { error: 'missing-fields' });
  }

  // Ghi hàng đợi rồi trả 200 ngay; xử lý nghiệp vụ nằm ở consumer, idempotent theo eventId.
  // Nếu lệnh này ném lỗi, Lambda trả 5xx và VíXanh sẽ gửi lại.
  await sqs.send(
    new SendMessageCommand({
      QueueUrl: process.env.QUEUE_URL,
      MessageBody: rawBody,
      MessageAttributes: {
        eventId: { DataType: 'String', StringValue: String(payload.eventId) },
        type: { DataType: 'String', StringValue: String(payload.type) },
      },
    }),
  );
  return reply(200, { received: true });
};
