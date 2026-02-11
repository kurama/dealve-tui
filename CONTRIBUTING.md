# Contributing to Dealve

Thank you for your interest in contributing to Dealve!

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/your-username/dealve-tui
   cd dealve-tui
   ```
3. Install pre-commit hooks:
   ```bash
   pip install pre-commit
   pre-commit install
   ```
4. Build the project:
   ```bash
   cargo build
   ```
5. Run the application:
   ```bash
   cargo run -p dealve-tui
   ```

## Project Structure

```
dealve-tui/
├── core/    # Shared types and domain logic
├── api/     # IsThereAnyDeal API client
└── tui/     # Terminal UI application
```

## Development

### Running Tests

```bash
cargo test
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Submitting Changes

1. Create a new branch for your feature or fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```
2. Make your changes and commit them with clear messages
3. Push to your fork
4. Open a Pull Request

## License

By contributing to Dealve, you agree that your contributions will be dual-licensed under the MIT and Apache 2.0 licenses.
