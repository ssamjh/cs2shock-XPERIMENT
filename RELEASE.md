# Build and release

## Build with GitHub Actions

The [CI workflow](https://github.com/ssamjh/cs2shock-XPERIMENT/actions/workflows/ci.yml) runs on pushes and pull requests targeting `main` or `dev`. It checks formatting, clippy, tests, then builds and packages the Windows x64 app.

To start a build manually, open the [Actions tab](https://github.com/ssamjh/cs2shock-XPERIMENT/actions), select **CI**, and choose **Run workflow**. Download `cs2shock-windows-x64.zip` from the successful run's artifacts. Artifacts are kept for 30 days.

If no runs appear after a push, check that Actions are enabled for this fork in **Settings > Actions > General**.

## Create a GitHub release manually

There is no automated release workflow. To publish a build:

1. Open a successful run in the [Actions tab](https://github.com/ssamjh/cs2shock-XPERIMENT/actions) and download the `cs2shock-windows-x64` artifact.
2. Open the [Releases page](https://github.com/ssamjh/cs2shock-XPERIMENT/releases) and create a release.
3. Attach `cs2shock-windows-x64.zip`, add release notes, and publish.

The archive contains the Windows executable, README, example configuration, and CS2 Game State Integration config. The ZIP uses a placeholder API token; users must add their own token before running the app.

## Local build

On Windows with Rust installed, run:

```bat
build.bat --no-pause
```

This creates `dist\cs2shock-windows-x64.zip`. To run the CI checks locally:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
