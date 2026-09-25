//! `ilmarinen-convert [--check] [PLUGIN_DIR] [OUT_DIR]`: emit hosts/opencode/
//! from plugin/. `--check` writes nothing and exits 1 when OUT_DIR is stale.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let check = args.iter().any(|a| a == "--check");
    args.retain(|a| a != "--check");
    let plugin = PathBuf::from(args.first().map_or("plugin", String::as_str));
    let out_dir = PathBuf::from(args.get(1).map_or("hosts/opencode", String::as_str));
    let typings: Vec<PathBuf> = std::env::var_os("HOME")
        .map(|h| vec![PathBuf::from(h).join(".config/opencode/node_modules/@opencode-ai/plugin/dist/index.d.ts")])
        .unwrap_or_default();
    let result = ilmarinen::convert::convert(&plugin).and_then(|mut out| {
        let note = ilmarinen::convert::probe_host(&mut out, &typings);
        for r in &out.report {
            println!("{r}");
        }
        println!("{note}");
        println!("\nNot expressible in OpenCode:");
        for (k, t) in &out.gaps {
            println!("  ! [{k}] {t}");
        }
        let bad = out.unaccepted();
        if !bad.is_empty() {
            let kinds: Vec<&str> = bad.iter().map(|(k, _)| k.as_str()).collect();
            anyhow::bail!("new gap type(s) not in the accepted list: {}", kinds.join(", "));
        }
        ilmarinen::convert::sync(&out, &out_dir, check)
    });
    match result {
        Ok(true) => {
            println!("\n{} is current", out_dir.display());
            ExitCode::SUCCESS
        }
        Ok(false) if check => {
            eprintln!("\n{} is stale; run `just convert`", out_dir.display());
            ExitCode::FAILURE
        }
        Ok(false) => {
            println!("\nwrote {}", out_dir.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
