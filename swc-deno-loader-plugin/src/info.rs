use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct Module {
    local: Option<String>,
    specifier: String,
}

#[derive(Deserialize, Debug)]
struct InfoOutput {
    modules: Vec<Module>,
    redirects: HashMap<String, String>,
}

pub fn from_local_to_specifier(info_result: &str, local: &str) -> Result<String> {
    let info: InfoOutput =
        serde_json::from_str(&info_result).context("Failed to parse JSON output")?;
    
    info.modules
        .iter()
        .find(|m| m.local.as_deref() == Some(local))
        .map(|m| m.specifier.clone())
        .ok_or_else(|| anyhow::anyhow!("Local path not found in module information"))
}

pub fn get_local_path(info_result: &str, specifier: &str) -> Result<String> {
    let info: InfoOutput =
        serde_json::from_str(&info_result).context("Failed to parse JSON output")?;

    // Follow redirects to get the final specifier
    let final_specifier = follow_redirects(specifier, &info.redirects)?;

    // Find module with the final specifier
    info.modules
        .into_iter()
        .find(|m| m.specifier == final_specifier)
        .and_then(|m| m.local)
        .ok_or_else(|| anyhow::anyhow!("Module not found or has no local path"))
}

fn follow_redirects(initial: &str, redirects: &HashMap<String, String>) -> Result<String> {
    let mut current = initial.to_string();
    let mut seen = std::collections::HashSet::new();

    while let Some(next) = redirects.get(&current) {
        if !seen.insert(current.clone()) {
            return Err(anyhow::anyhow!("Circular redirect detected"));
        }
        current = next.clone();
    }

    Ok(current)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::path::Path;

    pub fn get_deno_info(specifier: String) -> Result<String> {
        let output: std::process::Output = std::process::Command::new("deno")
            .args(["info", "--json", &specifier])
            .output()
            .context("Failed to execute deno info command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("deno info command failed: {}", stderr));
        }

        Ok(String::from_utf8(output.stdout).context("Failed to convert output to UTF-8")?)
    }

    #[test]
    fn test_get_local_path() -> Result<()> {
        let specifier = "jsr:@std/path/from-file-url";
        let info_result = get_deno_info(specifier.to_string()).expect("get deno info failed");
        let path = get_local_path(&info_result, specifier)?;
        println!("Debug: Received path: {}", path);
        assert!(Path::new(&path).exists());
        Ok(())
    }

    #[test]
    fn test_follow_redirects() {
        let mut redirects = HashMap::new();
        redirects.insert(
            "jsr:@std/path/from-file-url".to_string(),
            "https://jsr.io/@std/path/1.0.8/from_file_url.ts".to_string(),
        );

        let final_specifier = follow_redirects("jsr:@std/path/from-file-url", &redirects).unwrap();
        assert_eq!(
            final_specifier,
            "https://jsr.io/@std/path/1.0.8/from_file_url.ts"
        );
    }

    #[test]
    fn test_circular_redirects() {
        let mut redirects = HashMap::new();
        redirects.insert("a".to_string(), "b".to_string());
        redirects.insert("b".to_string(), "a".to_string());

        assert!(follow_redirects("a", &redirects).is_err());
    }
}
