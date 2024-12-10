import { bundle } from "@swc/core";
import { config } from "@swc/core/spack.js";
import { fromFileUrl } from "@std/path";
import { info } from "./info.ts";

// https://github.com/swc-project/swc/issues/2725

const output = await bundle(
  config({
    entry: {
      web: fromFileUrl(import.meta.resolve("./src/web.ts")),
    },
    output: {
      name: "build",
      path: import.meta.resolve("./build/"),
    },
    module: {},
    options: {
      jsc: {
        baseUrl: fromFileUrl(import.meta.resolve("./")),
        parser: {
          syntax: "typescript",
          tsx: true,
          decorators: true,
        },
        experimental: {
          plugins: [
            [
              fromFileUrl(
                import.meta.resolve(
                  "./swc-deno-loader-plugin/target/wasm32-wasip1/release/swc_deno_loader_plugin.wasm"
                )
              ),
              {
                info_result: JSON.stringify(await info("./src/web.ts")),
              },
            ],
          ],
        },
      },
    },
  })
);

console.log(output.web.code);
