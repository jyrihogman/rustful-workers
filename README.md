A very much work-in-progress worker for learning Rust.

The worker uses [`workers-rs`](https://github.com/cloudflare/workers-rs), [`Turso`](https://turso.tech/) and [`Upstash QStash`](https://upstash.com/docs/qstash/overall/getstarted) to handle notifications for a service that I'm building.
The repo is bootstrapped with [rustwasm-worker-template-worker-template](https://github.com/cloudflare/rustwasm-worker-template/) for compiling Rust to WebAssembly
and publishing the resulting worker to Cloudflare's [edge infrastructure](https://www.cloudflare.com/network/).

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
 Here is an [`architecture diagram`](https://excalidraw.com/#json=0bfV-F_IvUp5EAZDTvIpZ,Vrl_zrIzULSuQdNgNsOmXg) for how the how the service can be used.


