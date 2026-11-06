#!/usr/bin/env node
import { App, Aspects } from 'aws-cdk-lib';
import { AwsSolutionsChecks } from 'cdk-nag';
import { QuoteStack } from '../lib/quote-stack';

const app = new App();

const envName = (app.node.tryGetContext('env') ?? 'dev') as 'dev' | 'staging' | 'prod';
const imageTag = (app.node.tryGetContext('imageTag') ?? '1.0.0') as string;

// Stack không gắn account/region cố định (env-agnostic) để synth chạy được mà không cần credential.
// Khi deploy thật, dùng profile SSO; môi trường prod nên gắn env tường minh.
new QuoteStack(app, `toinoi-${envName}-quote`, {
  envName,
  imageTag,
  description: 'Tới Nơi: service quote trên ECS Fargate (bài 24)',
});

// cdk-nag chạy như một Aspect: quét mọi construct khi synth và báo lỗi theo pack AwsSolutions.
// Bật bằng: npx cdk synth -c nag=true
if (app.node.tryGetContext('nag') === 'true') {
  Aspects.of(app).add(new AwsSolutionsChecks({ verbose: true }));
}
