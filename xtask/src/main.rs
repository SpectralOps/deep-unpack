#![allow(dead_code)]
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command};
use duct::cmd;

const TEMPLATE_PROJECT_NAME: &str = "unpack";

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), anyhow::Error> {
    let cli = Command::new("xtask")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("test").arg(
                Arg::new("features")
                    .help("Space or comma separated list of features to activate")
                    .action(ArgAction::Append),
            ),
        )
        .subcommand(Command::new("fmt"))
        .subcommand(Command::new("clippy"))
        .subcommand(
            Command::new("docs")
                .arg(
                    Arg::new("keep")
                        .short('k')
                        .long("keep")
                        .action(ArgAction::SetFalse)
                        .help("keep previous generated docs"),
                )
                .arg(
                    Arg::new("open")
                        .short('o')
                        .long("open")
                        .action(ArgAction::SetTrue)
                        .help("Open doc website"),
                ),
        )
        .subcommand(Command::new("vars"));

    let matches = cli.get_matches();

    let root = root_dir();
    let project = root.join(TEMPLATE_PROJECT_NAME);
    let res = match matches.subcommand() {
        Some(("vars", _)) => {
            println!("project root: {:?}", project);
            println!("root: {:?}", root);
            Ok(())
        }
        Some(("test", sm)) => {
            let mut args = vec!["test"];

            let features: Vec<&str> = sm
                .get_many::<String>("features")
                .unwrap_or_default()
                .map(std::string::String::as_str)
                .collect();
            args.extend(features);

            cmd("cargo", &args).run()?;
            Ok(())
        }
        Some(("fmt", _)) => {
            cmd!("cargo", "fmt", "--all", "--", "--check").run()?;
            Ok(())
        }
        Some(("clippy", _)) => {
            cmd!("cargo", "clippy", "--all-features", "--", "-D", "warnings").run()?;
            Ok(())
        }
        Some(("docs", sm)) => {
            if !sm.get_one::<bool>("keep").expect("defaulted by cli") {
                cmd!("cargo", "clean", "--doc").run()?;
            }

            let mut args = vec![
                "doc",
                "--workspace",
                "--all-features",
                "--no-deps",
                "--document-private-items",
            ];

            if *sm.get_one::<bool>("open").expect("defaulted by cli") {
                args.push("--open");
            }

            cmd("cargo", &args).run()?;
            Ok(())
        }
        _ => unreachable!("unreachable branch"),
    };
    res
}

fn root_dir() -> PathBuf {
    let mut xtask_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    xtask_dir.pop();
    xtask_dir
}
