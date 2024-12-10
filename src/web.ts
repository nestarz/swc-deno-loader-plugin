import { fromFileUrl } from "jsr:@std/path@1.0.8/from-file-url";
const d = await import("./resolved/https_example.com/module.ts")
const a = fromFileUrl(import.meta.resolve("./web.ts"));
console.log(a, d);
