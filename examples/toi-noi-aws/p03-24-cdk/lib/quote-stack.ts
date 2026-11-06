import { Construct } from 'constructs';
import { CfnOutput, RemovalPolicy, Stack, StackProps, Tags } from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as s3 from 'aws-cdk-lib/aws-s3';
import { QuoteService } from './quote-service';

export interface QuoteStackProps extends StackProps {
  readonly envName: 'dev' | 'staging' | 'prod';
  readonly imageTag: string;
}

export class QuoteStack extends Stack {
  constructor(scope: Construct, id: string, props: QuoteStackProps) {
    super(scope, id, props);

    const isProd = props.envName === 'prod';

    // VPC hai AZ với một NAT Gateway (tính tiền theo giờ). Prod nên cân nhắc NAT theo AZ.
    const vpc = new ec2.Vpc(this, 'Vpc', {
      maxAzs: 2,
      natGateways: 1,
    });

    const service = new QuoteService(this, 'Quote', {
      envName: props.envName,
      vpc,
      repositoryName: 'toinoi/quote',
      imageTag: props.imageTag,
      maxTasks: isProd ? 16 : 4,
    });

    // Bucket xuất bảng giá: mã hoá, chỉ TLS, chặn public; bucket có dữ liệu thì giữ lại khi xoá stack.
    const exports = new s3.Bucket(this, 'TariffExports', {
      encryption: s3.BucketEncryption.S3_MANAGED,
      enforceSSL: true,
      blockPublicAccess: s3.BlockPublicAccess.BLOCK_ALL,
      versioned: true,
      removalPolicy: RemovalPolicy.RETAIN,
    });
    exports.grantWrite(service.fargate.taskDefinition.taskRole);

    Tags.of(this).add('Project', 'ToiNoi');
    Tags.of(this).add('Env', props.envName);
    Tags.of(this).add('Service', 'quote');
    Tags.of(this).add('Owner', 'quote-team');

    new CfnOutput(this, 'AlbDnsName', {
      value: service.fargate.loadBalancer.loadBalancerDnsName,
      description: 'DNS của ALB; trỏ api.toinoi.example bằng Route 53 alias ở bài 09',
    });
  }
}
