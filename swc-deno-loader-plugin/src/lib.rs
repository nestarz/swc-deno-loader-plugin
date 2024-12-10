#![allow(clippy::not_unsafe_ptr_arg_deref)]
mod info;

use swc_core::ecma::ast::Program;
use swc_core::plugin::plugin_transform;
use swc_core::plugin::proxies::TransformPluginProgramMetadata;

use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};

#[derive(serde::Deserialize)]
pub struct Config {
    pub info_result: String,
}

pub struct TransformVisitor {
    config: Config,
    filepath: String,
}

impl VisitMut for TransformVisitor {
    fn visit_mut_module_items(&mut self, items: &mut Vec<ModuleItem>) {
        println!("filepath: {}", &self.filepath);
        for item in items.iter_mut() {
            if let ModuleItem::ModuleDecl(ModuleDecl::Import(import_decl)) = item {
                let src = &import_decl.src.value;
                if src.starts_with(".") {
                    match info::from_local_to_specifier(&self.config.info_result, &self.filepath) {
                        Ok(specifier) => {
                            println!("{}", specifier);
                            let base_url = url::Url::parse(&specifier)
                                .expect("Failed to parse local_path as URL");
                            let joined_url =
                                base_url.join(&src.to_string()).expect("Failed to join URL");
                            let joined_path_str = if joined_url.scheme() == "file" {
                                joined_url.path().to_string()
                            } else {
                                joined_url.to_string()
                            };
                            let local_path =
                                info::get_local_path(&self.config.info_result, &joined_path_str)
                                    .expect("missing");
                            import_decl.src = Box::new(Str {
                                span: import_decl.src.span,
                                raw: None,
                                value: local_path.into(),
                            });
                        }
                        Err(e) => {
                            println!("Failed to get local path: {}", e);
                        }
                    }
                } else if src.starts_with("jsr:") {
                    match info::get_local_path(&self.config.info_result, src) {
                        Ok(local_path) => {
                            import_decl.src = Box::new(Str {
                                span: import_decl.src.span,
                                raw: None,
                                value: local_path.into(),
                            });
                        }
                        Err(e) => {
                            panic!("Failed to get local path: {}", e);
                        }
                    }
                } else if src.starts_with("http://") || src.starts_with("https://") {
                    let new_path = format!("./resolved/{}", src.replace("://", "_"));
                    import_decl.src = Box::new(Str {
                        span: import_decl.src.span,
                        raw: None, // Let SWC handle the raw string representation
                        value: new_path.into(),
                    });
                }
            }
        }
    }
}

#[plugin_transform]
fn plugin(mut program: Program, data: TransformPluginProgramMetadata) -> Program {
    let config = serde_json::from_str::<Config>(
        &data
            .get_transform_plugin_config()
            .expect("failed to get plugin config"),
    )
    .expect("invalid config for swc-confidential");
    let filepath = match data
        .get_context(&swc_core::plugin::metadata::TransformPluginMetadataContextKind::Filename)
    {
        Some(s) => s,
        None => String::from(""),
    };
    program.visit_mut_with(&mut TransformVisitor { config, filepath });

    program
}

impl swc_core::ecma::ast::Pass for TransformVisitor {
    fn process(&mut self, program: &mut Program) {
        program.visit_mut_with(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_core::ecma::{
        parser::{Syntax, TsSyntax},
        transforms::testing::test_inline,
    };

    const SYNTAX: Syntax = Syntax::Typescript(TsSyntax {
        tsx: true,
        decorators: false,
        dts: false,
        no_early_errors: false,
        disallow_ambiguous_jsx_like: true,
    });

    test_inline!(
        SYNTAX,
        |_| {
            let specifier: String = "jsr:@std/path@1.0.8/from-file-url".to_string();
            let info_result =
                info::tests::get_deno_info(specifier).expect("Failed to get deno info");
            TransformVisitor {
                config: Config { info_result },
                filepath: "".to_string(),
            }
        },
        transform_jsr_imports,
        r#"import { something } from "jsr:@std/path@1.0.8/from-file-url";"#,
        r#"import { something } from "/Users/elias/Library/Caches/deno/remote/https/jsr.io/d77d190843438d9f873bc4d830989e945a260d8789624ebbc6bfc13df1be4367";"#
    );

    test_inline!(
        SYNTAX,
        |_| {
            let specifier: String = "jsr:@std/path@1.0.8/from-file-url".to_string();
            let info_result =
                info::tests::get_deno_info(specifier).expect("Failed to get deno info");
            TransformVisitor {
                config: Config { info_result },
                filepath: "".to_string(),
            }
        },
        leave_local_imports_unchanged,
        r#"import { local } from "./local-module.ts";"#,
        r#"import { local } from "./local-module.ts";"#
    );
}
