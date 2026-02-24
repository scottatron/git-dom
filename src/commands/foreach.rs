use anyhow::{Context, Result};
use colored::Colorize;
use git2::Repository;
use std::process::Command;

use crate::submodule;

pub fn run(command: Vec<String>, parallel: bool) -> Result<()> {
    let repo = Repository::open_from_env().context("Not in a git repository")?;
    let workdir = repo
        .workdir()
        .context("Bare repositories are not supported")?;
    let submodules = submodule::discover(&repo, None)?;

    if submodules.is_empty() {
        println!("No submodules found.");
        return Ok(());
    }

    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    if parallel {
        run_parallel(&submodules, &command, workdir)?;
    } else {
        run_sequential(&submodules, &command, workdir)?;
    }

    Ok(())
}

fn run_sequential(
    submodules: &[submodule::SubmoduleInfo],
    command: &[String],
    workdir: &std::path::Path,
) -> Result<()> {
    for sm in submodules {
        println!("{}", format!("─── {} ───", sm.name).bold());

        let output = Command::new(&command[0])
            .args(&command[1..])
            .current_dir(workdir.join(&sm.path))
            .output()
            .with_context(|| format!("Failed to run command in {}", sm.name))?;

        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            eprintln!(
                "{} {} exited with {}",
                "✗".red().bold(),
                sm.name,
                output.status
            );
        }

        println!();
    }

    Ok(())
}

fn run_parallel(
    submodules: &[submodule::SubmoduleInfo],
    command: &[String],
    workdir: &std::path::Path,
) -> Result<()> {
    use std::thread;

    let handles: Vec<_> = submodules
        .iter()
        .map(|sm| {
            let cmd = command.to_vec();
            let dir = workdir.join(&sm.path);
            let name = sm.name.clone();

            thread::spawn(move || {
                let output = Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .current_dir(&dir)
                    .output();

                (name, output)
            })
        })
        .collect();

    for handle in handles {
        let (name, result) = handle.join().unwrap();

        println!("{}", format!("─── {} ───", name).bold());

        match result {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                eprint!("{}", String::from_utf8_lossy(&output.stderr));

                if !output.status.success() {
                    eprintln!(
                        "{} {} exited with {}",
                        "✗".red().bold(),
                        name,
                        output.status
                    );
                }
            }
            Err(e) => {
                eprintln!("{} {}: {}", "✗".red().bold(), name, e);
            }
        }

        println!();
    }

    Ok(())
}
