-- CloudTrail Lake: ai xoá VPC Flow Logs, từ đâu, bằng credential nào?
-- CHỈ dành cho tài khoản đã dùng CloudTrail Lake trước 31/05/2026 (Lake đóng với khách mới từ ngày đó).
-- Tài khoản mới dùng bản tương đương cho CloudWatch Logs Insights: tệp .logsinsights cùng tên.
-- Thay <EVENT_DATA_STORE_ID> như ở who-read-secret.sql.
SELECT eventTime,
       userIdentity.arn AS principal,
       userIdentity.accessKeyId AS access_key,
       sourceIPAddress,
       userAgent,
       awsRegion
FROM <EVENT_DATA_STORE_ID>
WHERE eventSource = 'ec2.amazonaws.com'
  AND eventName = 'DeleteFlowLogs'
ORDER BY eventTime DESC
