<p align="center">
  <a href="https://sentry.io/?utm_source=github&utm_medium=logo" target="_blank">
    <picture>
      <source srcset="https://sentry-brand.storage.googleapis.com/sentry-logo-white.png" media="(prefers-color-scheme: dark)" />
      <source srcset="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" media="(prefers-color-scheme: light), (prefers-color-scheme: no-preference)" />
      <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" alt="Sentry" width="280">
    </picture>
  </a>
</p>

_Bad software is everywhere, and we're tired of it. Sentry is on a mission to help developers write better software faster, so we can get back to enjoying technology. If you want to join us [<kbd>**Check out our open positions**</kbd>](https://sentry.io/careers/)_

# Sentry AWS Lambda Extension

AWS Lambda Extension for instrumenting Lambda functions.

## Local development

The extension is a simple Rust project that wraps around Sentry's [relay](https://github.com/getsentry/relay) and is shipped as a binary.

An AWS extension has to talk to the [AWS Lambda Extensions API](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-extensions-api.html). For local development we have to mock this API so we can develop our extension locally. See [Mocking the AWS Lambda environment](#mocking-the-aws-lambda-environment) below.

## Mocking the AWS Lambda environment

### Prerequisites

- Make sure you set the AWS environment variable to localhost port 5000:

  ```bash
  export AWS_LAMBDA_RUNTIME_API=localhost:5000
  ```

  (You can also use https://direnv.net/ for setting environment variables.)

- Make sure you have a Python 3 version installed and running on your machine.

### Running the Mock AWS extensions API

Then open a terminal and start our little server, that mocks the AWS Lambda API being run on localhost:5000:

```bash
cd mock-aws-lambda-extensions-api
./run.sh
```

The `run.sh` will

- create a Python virtual environment
- install necessary Python libraries
- start the mock API on [http://localhost:5000](http://localhost:5000)

## Getting help/support

If you need help setting up or configuring the Python SDK (or anything else in the Sentry universe) please head over to the [Sentry Community on Discord](https://discord.com/invite/Ww9hbqr). There is a ton of great people in our Discord community ready to help you!

## How the Sentry Lambda extension works

```mermaid
sequenceDiagram
    participant AWS Lambda Service
    participant Lambda Function
    participant Sentry Lambda Extension
    participant relay

    AWS Lambda Service->>Sentry Lambda Extension: init
    Sentry Lambda Extension->>relay: start
    Sentry Lambda Extension->>AWS Lambda Service: HTTP POST /register (invoke/shutdown)
    Sentry Lambda Extension->>AWS Lambda Service: HTTP GET /next

    AWS Lambda Service->>Lambda Function: invoke
    AWS Lambda Service->>Sentry Lambda Extension: invoke
    Lambda Function->>relay: send envelope
    Lambda Function->>relay: send transaction
    Lambda Function->>relay: send error
    Lambda Function-->>AWS Lambda Service: function invocation end

    Sentry Lambda Extension->>AWS Lambda Service: HTTP GET /next
    AWS Lambda Service->>Lambda Function: invoke
    AWS Lambda Service->>Sentry Lambda Extension: invoke
    Lambda Function->>relay: send error
    Lambda Function-->>AWS Lambda Service: function invocation end

    AWS Lambda Service->>Sentry Lambda Extension: shutdown
    Sentry Lambda Extension->>relay: shutdown
    relay-->>Sentry Lambda Extension: relay execution end
    Sentry Lambda Extension-->>AWS Lambda Service: extension execution end
```

## Resources

- [![Documentation](https://img.shields.io/badge/documentation-sentry.io-green.svg)](https://docs.sentry.io/quickstart/)
- [![Discord](https://img.shields.io/discord/621778831602221064)](https://discord.gg/Ww9hbqr)
- [![Stack Overflow](https://img.shields.io/badge/stack%20overflow-sentry-green.svg)](http://stackoverflow.com/questions/tagged/sentry)
- [![Twitter Follow](https://img.shields.io/twitter/follow/getsentry?label=getsentry&style=social)](https://twitter.com/intent/follow?screen_name=getsentry)

## License

Licensed under the BSD license, see [`LICENSE`](LICENSE)
