#![allow(unknown_lints)] // for clippy

#[macro_use] extern crate clap;
extern crate cargo;
extern crate regex;
extern crate void;

mod bundle;
mod check;
mod discovery;
mod license;
mod licensed;
mod list;
mod load;
mod options;
mod thirdparty;

use std::process;

use cargo::{Config, CliResult};
use cargo::core::Workspace;
use cargo::util::important_paths::find_root_manifest_for_wd;

use options::{Options, Cmd};

fn main() {
    let matches = Options::app(false).get_matches();
    let options = Options::from_matches(&matches);
    let mut config = Config::default().expect("No idea why this would fail");
    let result = real_main(options, &mut config);
    if let Err(err) = result {
        config.shell().error(format!("{:?}", err)).expect("Can't do much");
        process::exit(1);
    }
}

fn real_main(options: Options, config: &mut Config) -> CliResult {
    config.configure(
        options.verbose,
        Some(options.quiet),
        &options.color,
        options.frozen,
        options.locked,
        &None,
        &[])?;

    config.shell().warn("IANAL: This is not legal advice and is not guaranteed to be correct.")?;

    let manifest_path = find_root_manifest_for_wd(config.cwd())?;
    let workspace = Workspace::new(&manifest_path, config)?;

    match options.cmd {
        Cmd::Check { package } => {
            let mut error = Ok(());
            let roots = load::resolve_roots(&workspace, package)?;
            for root in roots {
                let packages = load::resolve_packages(&workspace, vec![&root])?;
                if let Err(err) = check::run(&root, packages, config) {
                    error = Err(err);
                }
            }
            error?;
        }

        Cmd::List { by, package } => {
            let roots = load::resolve_roots(&workspace, package)?;
            let packages = load::resolve_packages(&workspace, &roots)?;
            list::run(packages, by)?;
        }

        Cmd::Bundle { variant, package } => {
            let roots = load::resolve_roots(&workspace, package)?;
            let packages = load::resolve_packages(&workspace, &roots)?;
            bundle::run(&roots, packages, config, variant)?;
        }

        Cmd::ThirdParty { full } => {
            println!("cargo-lichking uses some third party libraries under their own license terms:");
            println!();
            for krate in thirdparty::CRATES {
                print!(" * {} under the terms of {}", krate.name, krate.licenses.name);
                if full {
                    println!(":");
                    let mut first = true;
                    for license in krate.licenses.licenses {
                        if first {
                            first = false;
                        } else {
                            println!();
                            println!("    ===============");
                        }
                        println!();
                        if let Some(text) = license.text {
                            for line in text.lines() {
                                println!("    {}", line);
                            }
                        } else {
                            println!("    Missing {} license text", license.name);
                        }
                    }
                }
                println!();
            }
        }
    }

    Ok(())
}
