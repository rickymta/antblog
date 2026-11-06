import { DynamoDBClient, PutItemCommand, DeleteItemCommand } from '@aws-sdk/client-dynamodb';

const ddb = new DynamoDBClient({});
const TABLE = process.env.TABLE_NAME;
const TTL_SECONDS = 2 * 60 * 60; // kết nối WebSocket không sống quá 2 giờ

// Route $connect, $disconnect, $default đều vào hàm này; phân biệt bằng routeKey.
export const handler = async (event) => {
  const { routeKey, connectionId } = event.requestContext;

  if (routeKey === '$connect') {
    // Khách gửi tripId qua query string: wss://.../lab?tripId=1001
    const tripId = event.queryStringParameters?.tripId ?? 'unknown';
    await ddb.send(
      new PutItemCommand({
        TableName: TABLE,
        Item: {
          connectionId: { S: connectionId },
          tripId: { S: tripId },
          expiresAt: { N: String(Math.floor(Date.now() / 1000) + TTL_SECONDS) },
        },
      }),
    );
    return { statusCode: 200 };
  }

  if (routeKey === '$disconnect') {
    await ddb.send(new DeleteItemCommand({ TableName: TABLE, Key: { connectionId: { S: connectionId } } }));
    return { statusCode: 200 };
  }

  // $default: client gửi thông điệp không khớp route nào; lab chỉ xác nhận.
  return { statusCode: 200, body: JSON.stringify({ message: 'ok' }) };
};
