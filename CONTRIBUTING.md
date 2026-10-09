# How to contribute to RapidQuery

## Issues

- Did you find a bug?
- Do you have any questions?
- Do you have any feature requests?

These are all welcome as [issues](https://github.com/pydantic/pydantic/issues/new).

> For reporting a security vulnerability, please see [Security Policy](./SECURITY.md).

If you're going to report a bug, please:

- Include RapidQuery version you're working with.
- Include a minimal example if possible.
- Include a reference if the bug is about SQL syntax.

## Pull Requests

It should be extremely simple to get started and create a PR. All PRs are welcome.

To make contributing as easy and fast as possible, you'll want to run tests and linting locally.

> Use `make format` to fix formatting, and `make` to run tests and linting.

## AI Policy

We welcome the use of AI when contributing to RapidQuery.

But, you should certify that they fully understand the code being submitted. We reserve the right to close any pull request at our discretion, without further justification. This includes, but is not limited to, cases where:

- the contribution does not meet our quality standards,
- the author appears to be mass-submitting pull requests across multiple repositories (spam),
- the pull request description is AI generated and incoherent or nonsensical.

## Installation & Setup

You'll need the following prerequisites:

- Any Python version between Python 3.10 and 3.15
- uv
- git - For version control
- make - For running development commands (or use nmake on Windows)
- Rust - Both stable and nightly (nightly is used for formatting)

Fork the repository on GitHub and clone your fork locally.

```bash
# Clone your fork and cd into the repo directory
git clone git@github.com:<your username>/rapidquery.git
cd rapidquery

# Install UV
# for other options see:
# https://docs.astral.sh/uv/getting-started/installation/
curl -LsSf https://astral.sh/uv/install.sh | sh

# Setup
make install
```

### Check out a new branch and make your changes

Create a new branch for your changes.

```bash
git switch -c my-new-feature-branch
```

### Run tests and linting

Run tests and linting locally to make sure everything is working as expected.

```bash
# Run automated code formatting and linting
make format

# Run tests and linting
make
```

### Commit and push your changes

Commit your changes, push your branch to GitHub, and create a drafted pull request.

When your pull request is ready for review, add a comment with the message "please review" and we'll take a look as soon as we can.
