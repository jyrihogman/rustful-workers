import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import { Runtime } from "@pulumi/aws/lambda";

const iamRole = new aws.iam.Role("lambdaIamRole", {
  assumeRolePolicy: aws.iam.assumeRolePolicyForPrincipal({
    Service: "lambda.amazonaws.com",
  }),
});

new aws.iam.RolePolicyAttachment("lambdaExecuteRolePolicy", {
  role: iamRole,
  policyArn: aws.iam.ManagedPolicies.AWSLambdaExecute,
});

new aws.iam.RolePolicyAttachment("vpcExecutionRoleAttachment", {
  role: iamRole,
  policyArn: aws.iam.ManagedPolicies.AWSLambdaVPCAccessExecutionRole,
});

new aws.lambda.Function("tursoAwsLambda", {
  code: new pulumi.asset.AssetArchive({
    bootstrap: new pulumi.asset.FileAsset(
      "../../target/lambda/turso-aws-lambda/bootstrap",
    ),
  }),
  handler: "bootstrap",
  runtime: Runtime.CustomAL2023,
  role: iamRole.arn,
});
