# Contributing to Stackvo

Thank you for your interest in contributing to Stackvo! We welcome contributions from the community.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Commit Message Format](#commit-message-format)
- [Changelog Generation](#changelog-generation)
- [Pull Request Process](#pull-request-process)

## 📜 Code of Conduct

Please read and follow our [Code of Conduct](docs/en/community/code-of-conduct.md).

## 🤝 How to Contribute

1. **Fork the repository** - Click the "Fork" button at the top right
2. **Clone your fork** - `git clone https://github.com/YOUR_USERNAME/stackvo.git`
3. **Create a feature branch** - `git checkout -b feature/amazing-feature`
4. **Make your changes** - Follow our coding standards
5. **Test your changes** - Ensure everything works
6. **Commit your changes** - Use conventional commit format
7. **Push to your fork** - `git push origin feature/amazing-feature`
8. **Create a Pull Request** - Open a PR to the `main` branch

## 🛠️ Development Setup

### Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- Bash 3.2+
- Git

### Setup Steps

```bash
# Clone the repository
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Copy environment file
cp .env.example .env

# Install CLI
./stackvo.sh install

# Generate configurations
stackvo generate

# Start services
stackvo up
```

## 📝 Coding Standards

### Bash Scripts

- Use `#!/usr/bin/env bash` shebang
- Enable strict mode: `set -euo pipefail`
- Use descriptive variable names in `snake_case`
- Add function documentation blocks in Turkish (for Turkish users)
- Use inline comments in English
- Follow existing code style

**Example:**

```bash
#!/usr/bin/env bash

##
# Generates project containers
#
# Parameters:
#   $1 - Project name
#   $2 - PHP version
##
generate_project() {
    local project_name=$1
    local php_version=$2

    # Check if project exists
    if [ -d "$project_name" ]; then
        log_error "Proje zaten mevcut"
        return 1
    fi

    # Create project directory
    mkdir -p "$project_name"
}
```

### Docker Compose Templates

- Use `.tpl` extension for templates
- Support Docker Compose profiles
- Avoid `depends_on` between optional services
- Use stdout/stderr for logs, not volume mounts
- Follow minimal configuration approach

### Documentation

- Keep documentation in sync with code
- Update both English (`docs/en/`) and Turkish (`docs/tr/`) versions
- Use clear, concise language
- Include code examples where appropriate

## 💬 Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>: <description>

[optional body]

[optional footer]
```

### Types

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

### Examples

```bash
# Good commits
feat: add PostgreSQL 15 support
fix: resolve Traefik SSL certificate error
docs: update installation guide for macOS
refactor: extract common generator functions
perf: optimize Docker image build time

# Bad commits
update readme
fixed bug
improvements
```

## 📦 Changelog Generation

### How It Works

Stackvo automatically generates changelogs from Git commit history using Conventional Commits format.

### Script

The changelog generation script is located at `docs/scripts/generate-changelog.sh`.

#### Usage

**Manual Usage** (For local testing):

```bash
./docs/scripts/generate-changelog.sh [version]
```

**Automatic Usage** (GitHub Actions):

- Runs automatically when you create a new tag on GitHub
- Workflow: `.github/workflows/changelog.yml`

#### Examples

```bash
# Mark as Unreleased
./docs/scripts/generate-changelog.sh

# For a specific version
./docs/scripts/generate-changelog.sh 1.2.0
```

#### Outputs

- `docs/tr/changelog.md` - Turkish changelog
- `docs/en/changelog.md` - English changelog

#### Conventional Commits Mapping

The script recognizes the following commit types:

- `feat:` → Added
- `fix:` → Fixed
- `docs:` → Documentation
- `refactor:` → Refactored
- `perf:` → Performance
- `test:` → Tests
- `chore:` → Chore

### GitHub Release Workflow

1. **Develop your code** and commit (in Conventional Commits format)

   ```bash
   git commit -m "feat: added new feature"
   git commit -m "fix: fixed bug"
   ```

2. **Create a new release on GitHub**

   - Releases → Draft a new release
   - Tag: `1.2.0` (without v prefix!)
   - Title: `1.2.0`
   - Description: Optional
   - Publish release

3. **GitHub Actions automatically**:
   - Updates the Changelog
   - Commits changes
   - Adds changelog to GitHub Release

#### Tag Format

> [!IMPORTANT]
> Do not use **"v" prefix** when creating tags. Correct format: `1.2.0`, `1.0.5`, etc.

**Correct:**

- ✅ `1.0.0`
- ✅ `1.2.5`
- ✅ `2.0.0`

**Incorrect:**

- ❌ `v1.0.0`
- ❌ `v1.2.5`

### Notes

- These scripts are for documentation purposes
- Main usage is via GitHub Actions
- Manual usage is for testing/development purposes only
- All commits must be in Conventional Commits format

## 🔍 Pull Request Process

1. **Update documentation** - If you change functionality, update docs
2. **Add tests** - If applicable (we're working on test coverage)
3. **Follow code style** - Match existing code patterns
4. **Keep commits clean** - Use conventional commit format
5. **Update changelog** - For major changes (optional, auto-generated)
6. **Request review** - Tag maintainers for review
7. **Address feedback** - Make requested changes
8. **Squash commits** - If requested by maintainers

### PR Title Format

Use the same format as commit messages:

```
feat: add support for PHP 8.4
fix: resolve Docker Compose profile issue
docs: improve Quick Start guide
```

## 🐛 Reporting Bugs

Use [GitHub Issues](https://github.com/stackvo/stackvo/issues) to report bugs.

**Before submitting:**

- Search existing issues
- Check if it's already fixed in `main` branch

**Include in your report:**

- Stackvo version (`stackvo --version`)
- Operating System (macOS, Linux, Windows WSL)
- Docker version (`docker --version`)
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages/logs

## 💡 Suggesting Features

We welcome feature suggestions! Open a [GitHub Discussion](https://github.com/stackvo/stackvo/discussions) or [Issue](https://github.com/stackvo/stackvo/issues).

**Good feature requests include:**

- Clear use case
- Expected behavior
- Possible implementation approach
- Examples from other tools (if applicable)

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🙏 Thank You

Thank you for contributing to Stackvo! Your efforts help make this project better for everyone.
