# Release runners

Build macOS release binaries natively on supported GitHub-hosted runners:
`macos-15-intel` for `x86_64-apple-darwin` and `macos-15` for
`aarch64-apple-darwin`. Keep the existing stable Rust setup, locked dependencies,
archive names, and tag-only publishing behavior.

Validate the migration by manually dispatching the release workflow on the PR
branch, checking its recorded head SHA, both native architectures, builds and
packaging. Branch runs upload workflow artifacts and do not create releases.

Runner labels: https://docs.github.com/en/actions/reference/runners/github-hosted-runners
