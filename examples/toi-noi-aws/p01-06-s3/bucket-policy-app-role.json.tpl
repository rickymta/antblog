{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "DenyInsecureTransport",
      "Effect": "Deny",
      "Principal": "*",
      "Action": "s3:*",
      "Resource": ["arn:aws:s3:::__BUCKET__", "arn:aws:s3:::__BUCKET__/*"],
      "Condition": { "Bool": { "aws:SecureTransport": "false" } }
    },
    {
      "Sid": "AllowDeliveryPhotoRoleReadWriteProof",
      "Effect": "Allow",
      "Principal": { "AWS": "__APP_ROLE_ARN__" },
      "Action": ["s3:GetObject", "s3:PutObject"],
      "Resource": "arn:aws:s3:::__BUCKET__/proof/*"
    }
  ]
}
