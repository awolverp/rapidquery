# How to contribute to RapidQuery

## Issues

- Did you find a bug?
- Do you have any questions?
- Do you have any feature requests?

These are all welcome as [issues](https://github.com/awolverp/rapidquery/issues/new).

> For reporting a security vulnerability, please see [Security Policy](./SECURITY.md).

If you're going to report a bug, please:

- Include RapidQuery version you're working with.
- Include a minimal example if possible.
- Include a reference if the bug is about SQL syntax.

## Pull Requests

It should be extremely simple to get started and create a PR. All PRs are welcome.

To make contributing as easy and fast as possible, you'll want to run tests and linting locally.

If you're going to add a feature or make a change to SQL, our _current_ priority is things that are supported in at least 3 dialects or defined in Standard SQL.

> Use `make format` to fix formatting, and `make` to run tests and linting.

## Dialects

Dialect parameters default to disabled or unsupported. Leave a default enabled only when every dialect needs that spelling in order to render, such as identifier quotes, string literals, or the placeholder marker.

Keep the parameter list and the `OPT_*` constants small. Add a flag to an existing set, or a variant to an existing enum, before adding a new field. Do not add an `OPT_*` constant for a clause that an existing flag already controls.

## Documentation

Rust items need rustdoc, and a short usage example helps. This project does not maintain doctests. Fence an example with `ignore`, or use a fence other than `rust`, so `cargo test` does not execute it.

A PyO3 class, method, or function only needs a one-line summary in Rust. Write the full docstring in the hand-written `rapidquery/*.pyi` stub.

Python docstrings follow [Google style](https://google.github.io/styleguide/pyguide.html#38-comments-and-docstrings) because the API docs are rendered with mkdocstrings.

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
- Rust - Nightly version

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
