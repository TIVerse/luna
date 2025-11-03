# Contributing to LUNA

Thank you for your interest in contributing to LUNA Voice Assistant! This document provides guidelines for contributing to the project.

## Author

**Eshan Roy** - [eshanized](https://github.com/eshanized)
- Organization: [TIVerse](https://github.com/TIVerse)
- Email: m.eshanized@gmail.com

## Getting Started

1. Fork the repository from [https://github.com/TIVerse/luna](https://github.com/TIVerse/luna)
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/luna.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Commit your changes: `git commit -m "Add your feature"`
6. Push to your fork: `git push origin feature/your-feature-name`
7. Create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75 or higher
- System dependencies (see README.md)

### Building

```bash
# Clone the repository
git clone https://github.com/TIVerse/luna.git
cd luna

# Build in development mode
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

## Code Style

- Follow Rust conventions and idiomatic practices
- Run `cargo fmt` before committing
- Run `cargo clippy` and address any warnings
- Add documentation comments for public APIs
- Write tests for new functionality

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings
```

## Testing

- Write unit tests for new functions
- Add integration tests for new features
- Ensure all tests pass before submitting PR

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

### Examples

```
feat(audio): add noise reduction support

Implements WebRTC-based noise reduction for better voice detection.

Closes #123
```

```
fix(brain): resolve command parsing for compound queries

The NLP parser now correctly handles compound queries with multiple intents.
```

## Pull Request Process

1. **Update Documentation**: Ensure README.md and relevant docs are updated
2. **Add Tests**: Include tests for new features or bug fixes
3. **Update CHANGELOG**: Add entry describing your changes
4. **Code Review**: Address feedback from maintainers
5. **Squash Commits**: Squash commits into logical units before merge

### PR Checklist

- [ ] Code follows project style guidelines
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] Commit messages follow guidelines
- [ ] No merge conflicts
- [ ] PR description clearly explains changes

## Project Structure

```
luna/
├── src/               # Source code
│   ├── audio/        # Audio system
│   ├── brain/        # NLP engine
│   ├── actions/      # Action executor
│   └── ...
├── tests/            # Integration tests
├── docs/             # Documentation
├── config/           # Configuration files
└── examples/         # Example code
```

## Areas for Contribution

### High Priority

- Audio preprocessing improvements
- NLP intent recognition
- Cross-platform compatibility
- Performance optimizations

### Documentation

- Tutorials and guides
- API documentation
- Example configurations
- Troubleshooting guides

### Testing

- Unit test coverage
- Integration tests
- Performance benchmarks
- Platform-specific tests

## Reporting Issues

When reporting issues, please include:

- **Description**: Clear description of the issue
- **Steps to Reproduce**: Detailed steps to reproduce the problem
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Rust version, LUNA version
- **Logs**: Relevant error logs or output

### Issue Template

```markdown
**Description**
A clear description of the issue.

**Steps to Reproduce**
1. Step one
2. Step two
3. ...

**Expected Behavior**
What should happen.

**Actual Behavior**
What actually happens.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust Version: [e.g., 1.75.0]
- LUNA Version: [e.g., 0.1.0]

**Logs**
```
Paste relevant logs here
```
```

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Accept constructive criticism
- Focus on what's best for the project
- Show empathy towards others

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or insulting comments
- Publishing others' private information
- Other unprofessional conduct

## Questions?

- **Issues**: [GitHub Issues](https://github.com/TIVerse/luna/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TIVerse/luna/discussions)
- **Email**: m.eshanized@gmail.com

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to LUNA! 🌙
