# Contributing to Stellar Bridge Watch

Thank you for your interest in contributing to Stellar Bridge Watch! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Pull Request Process](#pull-request-process)
5. [Coding Standards](#coding-standards)
6. [Testing Guidelines](#testing-guidelines)
7. [CI/CD Pipeline](#cicd-pipeline)
8. [Commit Message Guidelines](#commit-message-guidelines)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Node.js 20+
- Rust (latest stable)
- Docker and Docker Compose
- Git

### Setup Development Environment

1. Fork the repository
2. Clone your fork:

   ```bash
   git clone https://github.com/YOUR_USERNAME/Bridge-Watch.git
   cd Bridge-Watch
   ```

3. Add upstream remote:

   ```bash
   git remote add upstream https://github.com/StellaBridge/Bridge-Watch.git
   ```

4. Install dependencies:

   ```bash
   # Backend
   cd backend && npm install

   # Frontend (when available)
   cd frontend && npm install

   # Contracts
   cd contracts && cargo build
   ```

5. Set up environment variables:

   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

6. Start development services:
   ```bash
   docker-compose up -d postgres redis
   ```

## Development Workflow

### Branch Strategy

We use a Git Flow-inspired branching model:

- `main` - Production-ready code
- `develop` - Integration branch for features
- `feature/*` - New features
- `bugfix/*` - Bug fixes
- `hotfix/*` - Critical production fixes
- `release/*` - Release preparation

### Creating a Feature Branch

1. Ensure your local repository is up to date:

   ```bash
   git checkout develop
   git pull upstream develop
   ```

2. Create a feature branch:

   ```bash
   git checkout -b feature/your-feature-name
   ```

3. Make your changes and commit regularly:

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

4. Keep your branch up to date:

   ```bash
   git fetch upstream
   git rebase upstream/develop
   ```

5. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

## Pull Request Process

### Before Submitting

1. **Run tests locally:**

   ```bash
   # Backend tests
   cd backend && npm test

   # Contract tests
   cd contracts && cargo test
   ```

2. **Run linters:**

   ```bash
   # Backend
   cd backend && npm run lint

   # Contracts
   cd contracts && cargo clippy
   ```

3. **Build successfully:**

   ```bash
   # Backend
   cd backend && npm run build

   # Contracts
   cd contracts && cargo build --release
   ```

4. **Update documentation** if needed

### Submitting a Pull Request

1. Push your branch to your fork
2. Go to the original repository on GitHub
3. Click "New Pull Request"
4. Select your branch
5. Fill out the PR template completely
6. Link related issues using "Closes #123"
7. Request review from maintainers

### PR Requirements

- ✅ All CI checks must pass
- ✅ Code review approval required
- ✅ No merge conflicts
- ✅ Branch is up to date with base branch
- ✅ All conversations resolved

### After Submission

- Monitor CI/CD pipeline results
- Respond to review comments promptly
- Make requested changes in new commits
- Keep the PR updated with base branch

## Coding Standards

### TypeScript/JavaScript (Backend/Frontend)

- Use TypeScript for all new code
- Follow ESLint configuration
- Use meaningful variable and function names
- Add JSDoc comments for public APIs
- Prefer functional programming patterns
- Use async/await over promises

**Example:**

```typescript
/**
 * Fetches bridge health data for a specific asset
 * @param assetSymbol - The asset symbol (e.g., "USDC")
 * @returns Bridge health score and metrics
 */
async function getBridgeHealth(assetSymbol: string): Promise<BridgeHealth> {
  // Implementation
}
```

### Rust (Smart Contracts)

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Address all Clippy warnings
- Add documentation comments
- Write comprehensive tests
- Minimize contract size

**Example:**

```rust
/// Verifies bridge reserve backing for an asset
///
/// # Arguments
/// * `env` - Contract environment
/// * `asset_id` - Asset identifier
///
/// # Returns
/// Reserve verification result
pub fn verify_reserves(env: Env, asset_id: Symbol) -> Result<ReserveStatus, Error> {
    // Implementation
}
```

### General Guidelines

- Keep functions small and focused
- Avoid deep nesting (max 3 levels)
- Use early returns to reduce complexity
- Handle errors explicitly
- Don't commit commented-out code
- Remove console.log/println before committing

## Testing Guidelines

### Backend Tests

- Write unit tests for all services
- Use integration tests for API endpoints
- Mock external dependencies
- Aim for >80% code coverage
- Test error cases

**Example:**

```typescript
describe("BridgeService", () => {
  it("should fetch bridge status", async () => {
    const service = new BridgeService();
    const status = await service.getStatus("ethereum-bridge");
    expect(status.isHealthy).toBe(true);
  });
});
```

### Contract Tests

- Test all public functions
- Test edge cases and error conditions
- Use property-based testing where appropriate
- Test gas consumption

**Example:**

```rust
#[test]
fn test_reserve_verification() {
    let env = Env::default();
    let contract = create_contract(&env);

    let result = contract.verify_reserves(symbol_short!("USDC"));
    assert!(result.is_ok());
}
```

### Running Tests

```bash
# Backend tests
cd backend
npm test                    # Run all tests
npm test -- --coverage      # With coverage
npm test -- --watch         # Watch mode

# Contract tests
cd contracts
cargo test                  # Run all tests
cargo test --package soroban # Specific package
```

## CI/CD Pipeline

Our CI/CD pipeline automatically runs on every pull request and push to main branches. Understanding the pipeline helps you anticipate and fix issues before they block your PR.

### Pipeline Stages

1. **CI Workflow** (Required)
   - Backend: Lint, build, test
   - Frontend: Lint, build, test
   - Contracts: Format check, clippy, build, test
   - Docker: Build verification

2. **Code Quality** (Required)
   - ESLint analysis
   - Rust Clippy analysis
   - Dependency review

3. **Security Scanning** (Required)
   - NPM audit
   - Cargo audit
   - CodeQL analysis
   - Trivy vulnerability scan
   - Secret detection

### Viewing CI Results

1. Go to your PR on GitHub
2. Scroll to "Checks" section
3. Click on failed checks to view logs
4. Fix issues and push new commits

### Common CI Failures

**Linting Errors:**

```bash
# Fix automatically
npm run lint:fix
cargo fmt
```

**Test Failures:**

- Review test output in CI logs
- Run tests locally to reproduce
- Fix the issue and commit

**Build Failures:**

- Check for TypeScript errors
- Verify all dependencies are installed
- Ensure environment variables are set

**Security Issues:**

- Review security scan results
- Update vulnerable dependencies
- Fix code security issues

### Local CI Simulation

Run the same checks locally before pushing:

```bash
# Backend checks
cd backend
npm run lint
npm run build
npm test

# Contract checks
cd contracts
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --target wasm32-unknown-unknown --release
```

For detailed CI/CD documentation, see [docs/CI-CD-SETUP.md](docs/CI-CD-SETUP.md).

## Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes

### Examples

```bash
# Feature
feat(bridge): add reserve verification for USDC

# Bug fix
fix(api): resolve race condition in price aggregation

# Documentation
docs: update API endpoint documentation

# CI/CD
ci: add security scanning workflow

# Breaking change
feat(api)!: change health score calculation algorithm

BREAKING CHANGE: Health score now ranges from 0-100 instead of 0-10
```

### Scope

Use these scopes when applicable:

- `backend`: Backend changes
- `frontend`: Frontend changes
- `contracts`: Smart contract changes
- `api`: API changes
- `bridge`: Bridge monitoring
- `liquidity`: Liquidity tracking
- `price`: Price oracle
- `health`: Health scoring
- `ci`: CI/CD pipeline
- `docs`: Documentation

## Issue Assignment

Before starting work on an issue:

1. Check if the issue is already assigned
2. Comment on the issue expressing interest
3. Wait for maintainer assignment
4. Create your branch after assignment

## Questions and Help

- **General questions**: Open a discussion on GitHub
- **Bug reports**: Open an issue with reproduction steps
- **Feature requests**: Open an issue with detailed description
- **Security issues**: Email security@stellarbridgewatch.io (do not open public issue)

## Recognition

Contributors will be recognized in:

- README.md contributors section
- Release notes
- Project documentation

Thank you for contributing to Stellar Bridge Watch! 🚀
