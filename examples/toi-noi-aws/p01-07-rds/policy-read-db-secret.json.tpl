{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "ReadTripsDbMasterSecret",
      "Effect": "Allow",
      "Action": "secretsmanager:GetSecretValue",
      "Resource": "__SECRET_ARN__"
    }
  ]
}
