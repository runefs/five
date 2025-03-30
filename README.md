# five
A rust crate to support DCI natively in rust

## Contributing

We welcome contributions to the `five` crate! Here's how you can help:

### Setting Up Development Environment

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/five.git
   cd five
   ```

2. Install Rust (if you haven't already):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Build the project:
   ```bash
   cargo build
   ```

4. Run tests:
   ```bash
   cargo test
   ```

### Development Workflow

1. Create a new branch for your feature:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes and ensure:
   - All tests pass: `cargo test`
   - Code is properly formatted: `cargo fmt`
   - No clippy warnings: `cargo clippy`
   - The main example runs: `cargo run`

3. Commit your changes:
   ```bash
   git commit -m "feat: add your feature description"
   ```

   Please follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test changes
   - `refactor:` for code refactoring

4. Push to your fork and create a Pull Request

### Release Process

The crate is automatically published to crates.io through GitHub Actions when a new version tag is pushed. To create a new release:

1. Update version in `Cargo.toml`:
   ```toml
   [package]
   name = "five"
   version = "x.y.z"  # Update this version number
   ```

2. Update CHANGELOG.md (if exists) with your changes

3. Commit these changes:
   ```bash
   git commit -m "chore: bump version to x.y.z"
   ```

4. Create and push a new version tag:
   ```bash
   git tag vx.y.z
   git push origin main
   git push origin vx.y.z
   ```

The GitHub Action will automatically:
1. Run all tests
2. Verify the code builds and runs
3. Publish to crates.io
4. Create a GitHub release with an auto-generated changelog

### Version Numbers

We follow Semantic Versioning (SemVer):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality additions
- PATCH version for backwards-compatible bug fixes

### Getting Help

If you need help or have questions:
1. Check existing [issues](https://github.com/owner/five/issues)
2. Create a new issue for questions or problems
3. For security issues, please see our security policy