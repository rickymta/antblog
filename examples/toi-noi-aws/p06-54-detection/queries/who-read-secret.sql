-- CloudTrail Lake: ai đọc secret của VíXanh trong 7 ngày qua?
-- CHỈ dành cho tài khoản đã dùng CloudTrail Lake trước 31/05/2026 (Lake đóng với khách mới từ ngày đó).
-- Tài khoản mới dùng bản tương đương cho CloudWatch Logs Insights: tệp .logsinsights cùng tên.
-- Thay <EVENT_DATA_STORE_ID> bằng phần ID cuối của ARN event data store (terraform output event_data_store_arn).
-- Chạy: aws cloudtrail start-query --query-statement file://queries/who-read-secret.sql
SELECT eventTime,
       userIdentity.arn AS principal,
       sourceIPAddress,
       userAgent,
       errorCode
FROM <EVENT_DATA_STORE_ID>
WHERE eventSource = 'secretsmanager.amazonaws.com'
  AND eventName = 'GetSecretValue'
  AND element_at(requestParameters, 'secretId') LIKE '%toinoi/prod/vixanh%'
  AND eventTime > '2026-11-01 00:00:00'
ORDER BY eventTime DESC
