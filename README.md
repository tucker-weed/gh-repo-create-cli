# gh-repo-create-cli

Create a GitHub repository via the API using a Personal Access Token (PAT), then push your local repo over SSH with the correct `git@github.com:OWNER/REPO.git` remote.

## Prerequisites
- Rust toolchain and Cargo installed (https://rustup.rs)
- Git installed with working SSH auth to GitHub (e.g. `ssh -T git@github.com`)
- `GITHUB_TOKEN` in your environment with `repo` scope

## Install to your CLI toolchain
- From this checkout: `cargo install --path .`
- Can also install directly from GitHub: 
  - SSH: `cargo install --git git@github.com:tucker-weed/gh-repo-create-cli.git`  
  - HTTPS: `cargo install --git https://github.com/tucker-weed/gh-repo-create-cli.git`
The binary will be placed in `~/.cargo/bin` (ensure that directory is on your `PATH`).

## Usage
```bash
export GITHUB_TOKEN=ghp_yourtokenhere
gh-repo-create <repo-name> [--private] [--org ORG_NAME]
```

### Examples
```bash
# Public repo in your user account
gh-repo-create prompt-graph-tools

# Private repo
gh-repo-create prompt-graph-tools --private

# Repo under an org
gh-repo-create prompt-graph-tools --org my-org-name
```

## What the tool does
1. Creates a folder named after the repo and writes `README.md` with a header.
2. Runs `git init`, stages files, and commits "Initial commit".
3. Calls the GitHub API with your PAT to create the repo (user or org) and reads the returned `ssh_url`.
4. Adds `origin` using that SSH URL, renames the branch to `main`, and pushes `main` over SSH.

If anything fails, the CLI exits with the GitHub error body or the failing git command.
