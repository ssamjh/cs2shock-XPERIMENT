# GitHub Actions

The repository currently has one workflow: `.github/workflows/ci.yml`.

## CI workflow

CI runs on pushes to `main` or `dev`, pull requests targeting either branch, and manual `workflow_dispatch` requests. It runs on `windows-latest` and checks formatting, clippy warnings, tests, and the Windows release build/package script.

Each successful run uploads `cs2shock-windows-x64.zip` as the `cs2shock-windows-x64` artifact. The artifact is available from that run's summary for 30 days.

To start a build manually, open **Actions > CI > Run workflow**, select a branch, and run it. If Actions shows no runs after a push, check **Settings > Actions > General** and enable Actions for this fork.

## Creating a release

There is no automated release workflow. To distribute a build, download the ZIP artifact from a successful CI run and create a GitHub release manually if desired.
