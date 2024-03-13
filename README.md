A very much work-in-progress monorepo for a set of **serverless** workers for learning Rust and trying out a few different multi-region approaches (global) approaches with CloudFlare Workers and AWS.

---

## Serverless modules

All of the workers will use [Turso](https://turso.tech/), which I find to be the best database that runs in Edge with a set of
cool features. `turso-edge` and `turso-edge-cache` use the [workers-rs](https://github.com/cloudflare/workers-rs) crate and deploy to CloudFlare Workers, running in CloudFlare's [edge infrastructure](https://www.cloudflare.com/network/).

Plan is also to build workers for AWS Lambda. One for Edge and one region locked Lambda. Region locked lambda will
use the standard one region approach with caching.

In the end I want to measure the complexities of the different lambas and their environments.
I will also include performance tests to see how big of a latency difference there is with the different
approaches and runtimes.

## Note

Any external crates need to compile to `wasm32-unknown-unknown` or `wasm32-freestanding` target.
You can test crate compatibility with `cargo install -q worker-build && worker-build --release` or by `npx wrangler dev`.

## Wrangler

Wrangler is used to develop, deploy, and configure your Worker via CLI.
Further documentation for Wrangler can be found [here](https://developers.cloudflare.com/workers/tooling/wrangler).

With `wrangler`, you can build, test, and deploy your Worker with the following commands:

```sh
# Run dev environment
$ npm run dev
# Deploy to worker to CloudFlare
$ npm run deploy
```
