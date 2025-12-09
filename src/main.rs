use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(
    name = "gh-repo-create",
    about = "Create a GitHub repo via the API, then push a local repo over SSH."
)]
struct Cli {
    /// Name of the repository to create (also used for the local folder)
    repo_name: String,
    /// Create the repo as private
    #[arg(long)]
    private: bool,
    /// GitHub organization to create the repo under
    #[arg(long)]
    org: Option<String>,
}

#[derive(Serialize)]
struct RepoRequest<'a> {
    name: &'a str,
    private: bool,
}

#[derive(Deserialize)]
struct RepoResponse {
    ssh_url: String,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let token = env::var("GITHUB_TOKEN").context("GITHUB_TOKEN is not set in the environment")?;

    let repo_dir = Path::new(&cli.repo_name);
    fs::create_dir_all(repo_dir)
        .with_context(|| format!("creating directory {}", repo_dir.display()))?;
    fs::write(repo_dir.join("README.md"), format!("# {}\n", cli.repo_name))
        .context("writing README.md")?;
    println!("Created directory and README for {}", cli.repo_name);

    run_command(repo_dir, "git", &["init"])?;
    run_command(repo_dir, "git", &["add", "."])?;
    run_command(repo_dir, "git", &["commit", "-m", "Initial commit"])?;
    println!("Initialized git repo and created initial commit");

    println!("Creating GitHub repository via API...");
    let remote_url = create_github_repo(&cli.repo_name, cli.private, cli.org.as_deref(), &token)?;
    println!("Remote created: {remote_url}");

    run_command(repo_dir, "git", &["remote", "add", "origin", &remote_url])?;
    run_command(repo_dir, "git", &["branch", "-M", "main"])?;
    run_command(repo_dir, "git", &["push", "-u", "origin", "main"])?;
    println!("Pushed initial commit to GitHub over SSH");

    println!("\n✔ Repository setup complete!");
    println!("Local directory: ./{}", cli.repo_name);
    println!("Remote URL: {remote_url}");

    Ok(())
}

fn run_command(cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("running {program} {}", args.join(" ")))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("{program} {} failed: {stderr}", args.join(" "));
}

fn create_github_repo(
    repo_name: &str,
    private: bool,
    org: Option<&str>,
    token: &str,
) -> Result<String> {
    let url = match org {
        Some(org) => format!("https://api.github.com/orgs/{org}/repos"),
        None => "https://api.github.com/user/repos".to_string(),
    };

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("token {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "gh-repo-create-cli")
        .json(&RepoRequest {
            name: repo_name,
            private,
        })
        .send()
        .context("GitHub API request failed")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        bail!("GitHub repo creation failed: {status} {body}");
    }

    let repo: RepoResponse = response.json().context("parsing GitHub API response")?;
    Ok(repo.ssh_url)
}
