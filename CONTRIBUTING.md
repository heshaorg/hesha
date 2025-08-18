# Contributing to Hesha Protocol

Thank you for your interest in contributing to the Hesha Protocol! We welcome contributions from the community and appreciate your help in building a privacy-preserving phone verification system.

## Table of Contents

- [Alpha Software Notice](#alpha-software-notice)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Code Guidelines](#code-guidelines)
- [Security Considerations](#security-considerations)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Community Guidelines](#community-guidelines)

## ⚠️ Alpha Software Notice

Please note that Hesha is currently in **alpha**. APIs and features may change significantly. We appreciate your patience and feedback during this early stage.

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or later (stable recommended)
- **Git**: For version control
- **GitHub Account**: For contributing via pull requests

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

## Code Guidelines

### Rust Code Standards

- **Formatting**: Use `cargo fmt` (enforced by CI)
- **Linting**: Follow `cargo clippy` suggestions (warnings treated as errors)
- **Naming Conventions**:
  - `snake_case` for functions and variables
  - `PascalCase` for types and structs
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

- **Error Handling**: Use `Result<T, E>` and `?` operator, avoid `unwrap()` in production
- **Documentation**: Document all public APIs with examples
- **Modules**: Keep modules focused and well-organized
- **Dependencies**: Minimize external dependencies, especially for crypto operations

### Documentation Standards

All public APIs must include documentation:

```rust
/// Creates a new phone number with validation.
///
/// # Arguments
/// * `number` - Phone number in E.164 format (e.g., "+1234567890")
///
/// # Returns
/// * `Ok(PhoneNumber)` - Successfully validated phone number
/// * `Err(HeshaError)` - Invalid format or length
///
/// # Examples
/// ```
/// use hesha_types::PhoneNumber;
///
/// let phone = PhoneNumber::new("+1234567890")?;
/// assert_eq!(phone.as_str(), "+1234567890");
/// ```
///
/// # Security
/// Phone numbers are validated but never logged in plaintext.
pub fn new(number: impl Into<String>) -> HeshaResult<Self>
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

## Testing Requirements

### Test Categories

1. **Unit Tests**: Test individual functions/modules
   ```bash
   cargo test --lib --bins
   ```

2. **Integration Tests**: Test component interactions
   ```bash
   cargo test --tests
   ```

3. **Documentation Tests**: Ensure examples in docs work
   ```bash
   cargo test --doc
   ```

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

### Our CI Pipeline

The CI runs the following jobs on every PR:

- **format**: Code formatting check (`cargo fmt --check`)
- **lint**: Clippy linting (`cargo clippy --all-targets -- -D warnings`)
- **unit-tests**: Library and binary tests (`cargo test --lib --bins`)
- **integration-tests**: Integration test suite (`cargo test --tests`)
- **build**: Workspace build verification (`cargo build --workspace --all-targets`)
- **spell-check**: Documentation spell checking

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