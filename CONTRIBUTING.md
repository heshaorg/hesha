# Contributing to Hesha Protocol

Thank you for your interest in contributing to the Hesha Protocol! We welcome contributions from the community.

## ⚠️ Alpha Software Notice

Please note that Hesha is currently in **alpha**. APIs and features may change significantly. We appreciate your patience and feedback during this early stage.

## How to Contribute

### Reporting Issues

- Check if the issue already exists in our [issue tracker](https://github.com/heshaorg/hesha/issues)
- Include detailed steps to reproduce the problem
- Provide your environment details (OS, Rust version, etc.)
- For security vulnerabilities, please see [SECURITY.md](SECURITY.md)

### Suggesting Features

- Open an issue with the "enhancement" label
- Clearly describe the use case and benefits
- Be open to discussion and feedback

### Submitting Code

1. **Fork the repository**
   ```bash
   git clone https://github.com/heshaorg/hesha.git
   cd hesha
   ```

2. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**
   - Follow existing code style and conventions
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass: `cargo test`

4. **Commit your changes**
   ```bash
   git commit -m "feat: add new feature"
   ```
   
   Commit message format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test additions/changes
   - `refactor:` for code refactoring

5. **Push and create a Pull Request**
   ```bash
   git push origin feature/your-feature-name
   ```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and address any warnings
- Follow Rust naming conventions
- Add documentation comments for public APIs

### Testing

- Write unit tests for new functions
- Add integration tests for new features
- Ensure all tests pass before submitting PR
- Include test cases for error conditions

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/heshaorg/hesha.git
cd hesha

# Build all components
cargo build --workspace

# Run tests
cargo test --workspace

# Run a specific crate
cargo run -p issuer-node
```

## Documentation

- Update relevant `.md` files in `/docs`
- Add inline documentation for new code
- Include examples where helpful
- Keep documentation concise and clear

## Review Process

1. All PRs require at least one review
2. CI tests must pass
3. Documentation must be updated
4. Code must follow project conventions

## Community

- Be respectful and constructive
- Help others in issues and discussions
- Share your use cases and feedback

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

Thank you for helping make Hesha Protocol better! 🚀