import { App } from 'aws-cdk-lib';
import { Match, Template } from 'aws-cdk-lib/assertions';
import { QuoteStack } from '../lib/quote-stack';

function synth(envName: 'dev' | 'staging' | 'prod' = 'dev'): Template {
  const app = new App();
  const stack = new QuoteStack(app, `test-${envName}`, { envName, imageTag: '1.0.0' });
  return Template.fromStack(stack);
}

describe('QuoteStack', () => {
  test('bucket xuất bảng giá được mã hoá, chặn public và giữ lại khi xoá stack', () => {
    const template = synth();
    template.hasResourceProperties('AWS::S3::Bucket', {
      BucketEncryption: Match.objectLike({
        ServerSideEncryptionConfiguration: Match.anyValue(),
      }),
      PublicAccessBlockConfiguration: {
        BlockPublicAcls: true,
        BlockPublicPolicy: true,
        IgnorePublicAcls: true,
        RestrictPublicBuckets: true,
      },
    });
    template.hasResource('AWS::S3::Bucket', {
      DeletionPolicy: 'Retain',
      UpdateReplacePolicy: 'Retain',
    });
  });

  test('ECS service bật deployment circuit breaker có rollback', () => {
    synth().hasResourceProperties('AWS::ECS::Service', {
      DeploymentConfiguration: Match.objectLike({
        DeploymentCircuitBreaker: { Enable: true, Rollback: true },
      }),
    });
  });

  test('prod giữ log group khi xoá stack, dev thì không', () => {
    synth('prod').hasResource('AWS::Logs::LogGroup', { DeletionPolicy: 'Retain' });
    synth('dev').hasResource('AWS::Logs::LogGroup', { DeletionPolicy: 'Delete' });
  });

  test('có đúng một ECS service và một ALB', () => {
    const template = synth();
    template.resourceCountIs('AWS::ECS::Service', 1);
    template.resourceCountIs('AWS::ElasticLoadBalancingV2::LoadBalancer', 1);
  });
});
