// Starts the landing page's Wasm client. The glue finds its module beside
// itself, and an external script keeps the page within a `script-src 'self'`
// policy.
import init from "./pkg/reapers_landing.js";

await init();
