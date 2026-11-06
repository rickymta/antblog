#!/usr/bin/env bash
# shellcheck disable=SC2016  # dấu backtick trong JMESPath là cố ý
set -euo pipefail
# CHỈ ĐỌC: liệt kê những thứ còn tồn tại và đang (hoặc có thể) tính tiền sau khi dọn. Kết quả rỗng = sạch.
# Chạy lại ở MỌI region bạn từng dùng bằng cách đặt AWS_REGION khác.
source "$(dirname "$0")/common.sh"

check() { # tiêu đề, lệnh...
  local title="$1"; shift
  printf '\n-- %s\n' "$title"
  "$@" 2>&1 || echo "(không chạy được, kiểm quyền hoặc dịch vụ)"
}

check "NAT Gateway đang tồn tại (tính tiền theo giờ + dữ liệu)" \
  aws ec2 describe-nat-gateways --filter Name=state,Values=pending,available \
  --query 'NatGateways[].[NatGatewayId,VpcId,State]' --output text
check "Elastic IP (EIP không gắn vào gì vẫn tính tiền)" \
  aws ec2 describe-addresses --query 'Addresses[].[PublicIp,AllocationId,AssociationId]' --output text
check "Application/Network Load Balancer" \
  aws elbv2 describe-load-balancers --query 'LoadBalancers[].[LoadBalancerName,Type,State.Code]' --output text
check "EC2 instance chưa terminated (stopped vẫn còn EBS)" \
  aws ec2 describe-instances --filters Name=instance-state-name,Values=pending,running,stopping,stopped \
  --query 'Reservations[].Instances[].[InstanceId,State.Name,Tags[?Key==`Name`]|[0].Value]' --output text
check "EBS volume không gắn vào instance (status available)" \
  aws ec2 describe-volumes --filters Name=status,Values=available \
  --query 'Volumes[].[VolumeId,Size,VolumeType]' --output text
check "EBS snapshot do tài khoản này sở hữu" \
  aws ec2 describe-snapshots --owner-ids self --query 'Snapshots[].[SnapshotId,VolumeSize,StartTime]' --output text
check "VPC endpoint (loại Interface tính tiền theo giờ)" \
  aws ec2 describe-vpc-endpoints --query 'VpcEndpoints[].[VpcEndpointId,VpcEndpointType,ServiceName]' --output text
check "RDS instance" \
  aws rds describe-db-instances --query 'DBInstances[].[DBInstanceIdentifier,DBInstanceStatus,MultiAZ]' --output text
check "RDS manual snapshot (không tự hết hạn)" \
  aws rds describe-db-snapshots --snapshot-type manual --query 'DBSnapshots[].[DBSnapshotIdentifier,AllocatedStorage]' --output text
check "Auto Scaling Group" \
  aws autoscaling describe-auto-scaling-groups --query 'AutoScalingGroups[].[AutoScalingGroupName,DesiredCapacity]' --output text
check "S3 bucket bắt đầu bằng toinoi-" \
  aws s3api list-buckets --query "Buckets[?starts_with(Name, 'toinoi-')].Name" --output text
check "CloudWatch log group /toinoi/" \
  aws logs describe-log-groups --log-group-name-prefix /toinoi/ --query 'logGroups[].[logGroupName,storedBytes]' --output text
check "CloudWatch log group của RDS (/aws/rds/instance/toinoi-lab-)" \
  aws logs describe-log-groups --log-group-name-prefix /aws/rds/instance/toinoi-lab- --query 'logGroups[].[logGroupName,storedBytes]' --output text
check "CloudWatch alarm toinoi-lab-" \
  aws cloudwatch describe-alarms --alarm-name-prefix toinoi-lab- --query 'MetricAlarms[].AlarmName' --output text
check "CloudWatch dashboard toinoi-lab-" \
  aws cloudwatch list-dashboards --dashboard-name-prefix toinoi-lab- --query 'DashboardEntries[].DashboardName' --output text
check "Secrets Manager secret" \
  aws secretsmanager list-secrets --query 'SecretList[].[Name,DeletedDate]' --output text
check "Mọi tài nguyên còn mang tag Project=ToiNoi ở region này" \
  aws resourcegroupstaggingapi get-resources --tag-filters Key=Project,Values=ToiNoi \
  --query 'ResourceTagMappingList[].ResourceARN' --output text
echo
echo "Chưa có trong script (kiểm tay): VPC/subnet/IGW còn lại (miễn phí nhưng nên xoá), IAM role/policy của lab,"
echo "bản ghi/hosted zone Route 53 nếu bạn đã tạo ở bài 09 (hosted zone tính tiền theo tháng), CloudFront, WAF."
