# Contributing to Hesha Protocol

Thank you for your interest in contributing to the Hesha Protocol! We welcome contributions from the community and appreciate your help in building a privacy-preserving phone verification system.

## Table of Contents

- [Alpha Software Notice](#alpha-software-notice)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Security Considerations](#security-considerations)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Community Guidelines](#community-guidelines)

## ⚠️ Alpha Software Notice

Please note that Hesha is currently in **alpha**. APIs and features may change significantly. We appreciate your patience and feedback during this early stage.

## Getting Started

### Project Structure

```
hesha/
├── cli/hesha-cli/          # Command-line interface
├── crates/
│   ├── hesha-types/        # Core type definitions
│   ├── hesha-crypto/       # Cryptographic operations
│   ├── hesha-core/         # Protocol implementation
│   └── hesha-client/       # Client library
├── nodes/issuer-node/      # Issuer node server
├── tests/                  # Integration tests
└── docs/                   # Documentation
```

## Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/heshaorg/hesha.git
   cd hesha
   ```

2. **Install Dependencies**
   ```bash
   # Rust toolchain is configured via rust-toolchain.toml
   cargo build --workspace
   ```

3. **Verify Setup**
   ```bash
   # Run all quality checks (simulates CI)
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test --workspace
   ```

4. **Run Integration Tests**
   ```bash
   cargo test --tests
   ```

## Making Changes

### Branch Naming Convention

Use descriptive branch names with prefixes:

- `feat/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Adding tests
- `ci/` - CI/CD improvements

**Examples:**
- `feat/proxy-number-validation`
- `fix/attestation-expiry-bug`
- `docs/update-installation-guide`

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]
```

**Types:**
- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `ci:` - CI/CD changes
- `chore:` - Maintenance tasks

**Examples:**
```bash
feat(crypto): add Ed25519 signature verification
fix(cli): resolve panic when home directory not found
docs: update README with installation instructions
```

## Security Considerations

### Critical Security Guidelines

- **Never commit private keys or secrets** to the repository
- **All cryptographic code requires security review** before merging
- **Use constant-time operations** for sensitive comparisons
- **Follow established patterns** in the `hesha-crypto` crate
- **Validate all inputs** thoroughly, especially from external sources
- **Use proper error handling** - never expose internal details in error messages

### Reporting Security Issues

**DO NOT** open public issues for security vulnerabilities. Instead:

1. Email security concerns to the maintainers privately
2. Include detailed description and reproduction steps
3. Allow reasonable time for response and fix

### Security Review Requirements

The following changes require additional security review:
- Modifications to cryptographic operations
- Changes to key generation or handling
- Input validation logic
- Authentication or authorization code
- Network protocol implementations


## Pull Request Process

1. **Create Feature Branch**
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Make Changes**
   - Follow code style guidelines
   - Add comprehensive tests
   - Update documentation
   - Ensure security best practices

3. **Test Your Changes**
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test --workspace
   ```

4. **Commit and Push**
   ```bash
   git add .
   git commit -m "feat: your descriptive commit message"
   git push origin feat/your-feature-name
   ```

5. **Create Pull Request**
   - Use clear, descriptive title following conventional commit format
   - Provide detailed description of changes
   - Link related issues using `Closes #123` or `Relates to #456`
   - Ensure all CI checks pass

### PR Review Requirements

- **At least one approving review** from a maintainer
- **All CI checks must pass** (format, lint, tests, spell-check)
- **Documentation updated** for user-facing changes
- **Security review** for sensitive changes
- **No merge conflicts** with target branch

## Issue Reporting

### Bug Reports

Please include:
- **Environment**: OS, Rust version, Hesha version
- **Steps to reproduce**: Clear, minimal example
- **Expected vs actual behavior**
- **Logs**: Relevant error messages (redact sensitive information)
- **Impact**: How severe is the issue

### Feature Requests

Please include:
- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives considered**: Other approaches
- **Breaking changes**: Any compatibility concerns

### Security Vulnerabilities

See [SECURITY.md](SECURITY.md) for reporting security issues privately.

## Community Guidelines

### Code of Conduct

- **Be respectful and constructive** in all interactions
- **Welcome newcomers** and help them get started
- **Focus on technical merit** and project goals
- **Respect different perspectives** and experiences

### Getting Help

- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Code Review**: Provide constructive feedback on PRs

## License

By contributing to Hesha Protocol, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Hesha Protocol! Your efforts help build a more private and secure digital communication ecosystem. 🚀