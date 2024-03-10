A very much work-in-progress worker for learning Rust, WASM and a multi region (as global as it can get pretty much) approach with CloudFlare Workers.

The worker uses [`workers-rs`](https://github.com/cloudflare/workers-rs), [`Turso`](https://turso.tech/) and [`Upstash QStash`](https://upstash.com/docs/qstash/overall/getstarted) to handle notifications for a service that I'm building.
The repo is bootstrapped with [rustwasm-worker-template-worker-template](https://github.com/cloudflare/rustwasm-worker-template/) for compiling Rust to WebAssembly
and publishing the resulting worker to Cloudflare's [edge infrastructure](https://www.cloudflare.com/network/).

## NOTE

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
