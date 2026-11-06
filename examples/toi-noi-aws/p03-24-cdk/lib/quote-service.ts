import { Construct } from 'constructs';
import { Duration, RemovalPolicy } from 'aws-cdk-lib';
import * as ec2 from 'aws-cdk-lib/aws-ec2';
import * as ecr from 'aws-cdk-lib/aws-ecr';
import * as ecs from 'aws-cdk-lib/aws-ecs';
import * as ecsPatterns from 'aws-cdk-lib/aws-ecs-patterns';
import * as logs from 'aws-cdk-lib/aws-logs';
import * as secretsmanager from 'aws-cdk-lib/aws-secretsmanager';

export interface QuoteServiceProps {
  /** Tên môi trường: dev, staging hoặc prod. */
  readonly envName: string;
  readonly vpc: ec2.IVpc;
  /** Tên repository ECR đã có (bài 11). */
  readonly repositoryName: string;
  /** Tag hoặc digest của image; production nên ghim digest. */
  readonly imageTag: string;
  readonly minTasks?: number;
  readonly maxTasks?: number;
}

/**
 * L3 construct của Tới Nơi: một service Fargate sau ALB với mặc định an toàn.
 * Người dùng construct này không phải nhớ bật circuit breaker, giữ log hay đưa secret vào task.
 */
export class QuoteService extends Construct {
  public readonly fargate: ecsPatterns.ApplicationLoadBalancedFargateService;
  public readonly pricingApiKey: secretsmanager.Secret;

  constructor(scope: Construct, id: string, props: QuoteServiceProps) {
    super(scope, id);

    const isProd = props.envName === 'prod';
    const prefix = `toinoi-${props.envName}-quote`;

    // Secret do Secrets Manager sinh; task nhận qua biến môi trường, giá trị không nằm trong template.
    this.pricingApiKey = new secretsmanager.Secret(this, 'PricingApiKey', {
      secretName: `${prefix}/pricing-api-key`,
      description: 'Khoá gọi nguồn bảng giá của service quote (minh hoạ)',
      generateSecretString: { passwordLength: 32, excludePunctuation: true },
    });

    const logGroup = new logs.LogGroup(this, 'Logs', {
      logGroupName: `/toinoi/${props.envName}/quote`,
      retention: isProd ? logs.RetentionDays.ONE_MONTH : logs.RetentionDays.TWO_WEEKS,
      removalPolicy: isProd ? RemovalPolicy.RETAIN : RemovalPolicy.DESTROY,
    });

    const repository = ecr.Repository.fromRepositoryName(this, 'Repository', props.repositoryName);

    const cluster = new ecs.Cluster(this, 'Cluster', {
      clusterName: prefix,
      vpc: props.vpc,
    });

    this.fargate = new ecsPatterns.ApplicationLoadBalancedFargateService(this, 'Service', {
      serviceName: prefix,
      cluster,
      cpu: 256,
      memoryLimitMiB: 512,
      desiredCount: props.minTasks ?? 2,
      publicLoadBalancer: true,
      circuitBreaker: { rollback: true },
      // Rolling deploy giữ đủ task đang chạy trong lúc thay (đổi lại cần dư chỗ cho task mới).
      minHealthyPercent: 100,
      maxHealthyPercent: 200,
      taskImageOptions: {
        image: ecs.ContainerImage.fromEcrRepository(repository, props.imageTag),
        containerPort: 8080,
        environment: {
          ENVIRONMENT: props.envName,
          SERVICE_NAME: 'quote',
        },
        secrets: {
          PRICING_API_KEY: ecs.Secret.fromSecretsManager(this.pricingApiKey),
        },
        logDriver: ecs.LogDrivers.awsLogs({ streamPrefix: 'quote', logGroup }),
      },
    });

    // Health check của target group trỏ vào endpoint nhẹ của service.
    this.fargate.targetGroup.configureHealthCheck({
      path: '/health/ready',
      healthyHttpCodes: '200',
      interval: Duration.seconds(15),
    });

    // Autoscaling theo CPU: giờ cao điểm 11:00 và 17:00 kéo thêm task, ngoài giờ co lại.
    const scaling = this.fargate.service.autoScaleTaskCount({
      minCapacity: props.minTasks ?? 2,
      maxCapacity: props.maxTasks ?? 8,
    });
    scaling.scaleOnCpuUtilization('Cpu', {
      targetUtilizationPercent: 60,
      scaleInCooldown: Duration.minutes(5),
      scaleOutCooldown: Duration.minutes(1),
    });
  }
}
