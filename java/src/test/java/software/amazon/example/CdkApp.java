package software.amazon.example;

import software.amazon.awscdk.App;
import software.amazon.awscdk.Environment;
import software.amazon.awscdk.Stack;
import software.amazon.awscdk.services.lambda.Code;
import software.amazon.awscdk.services.lambda.Function;
import software.amazon.awscdk.services.lambda.Runtime;
import software.amazon.awscdk.services.s3.Bucket;
import software.constructs.Construct;

public final class CdkApp {
  public static void main(final String[] args) {
    final App app = new App();
    final Stack stack = Stack.Builder.create(app, "ExampleStack")
            .env(Environment.builder()
                    .region("eu-west-3")
                    .build())
            .build();

    new CustomConstruct(stack, "Custom");

    app.synth();
  }

  private static final class CustomConstruct extends Construct {
    public CustomConstruct(final Construct scope, final String id) {
      super(scope, id);

      new Bucket(this, "S3Bucket");
      Function.Builder.create(this, "Lambda")
              .code(Code.fromInline("export async function handler(){}"))
              .runtime(Runtime.NODEJS_12_X)
              .handler("index.handler")
              .build();
    }
  }
}
