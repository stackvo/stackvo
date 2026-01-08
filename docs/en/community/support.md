# Support

Ways to get support for Stackvo. This page explains in detail support channels such as GitHub Discussions, GitHub Issues, and documentation, the guide for asking good questions, bug report and feature request templates, response times, and contact information. It covers how the community-based support system works and ways to get help fastest.

---

## Support Channels

### 1. GitHub Discussions (Recommended)

**Best option:** To ask questions, share ideas, and discuss.

[💬 Go to Discussions →](https://github.com/stackvo/stackvo/discussions)

**Categories:**
- 💡 **Ideas** - Feature suggestions
- 🙏 **Q&A** - Questions and answers
- 📣 **Announcements** - Announcements
- 💬 **General** - General discussions

### 2. GitHub Issues

**For bug reports and feature requests.**

[Open Issue →](https://github.com/stackvo/stackvo/issues/new)

**When to use:**
- When you find a bug
- When you suggest a new feature
- When you see a documentation error

### 3. Documentation

**Check the documentation first:**

- [Getting Started](../started/index.md)
- [Installation](../installation/index.md)
- [Configuration](../configuration/index.md)
- [Guides](../guides/index.md)
- [FAQ](faq.md)
- [Troubleshooting](troubleshooting.md)

---

## Question Asking Guide

### How to Ask a Good Question?

#### ✅ Good Example

```markdown
## Problem: MySQL container is not starting

**Environment:**
- OS: Ubuntu 22.04
- Docker: 24.0.7
- Stackvo: 1.0.0

**Steps:**
1. ./stackvo.sh generate
2. ./stackvo.sh up

**Error:**
```
Error: MySQL container exited with code 1
```

**Logs:**
```
docker logs stackvo-mysql
[ERROR] InnoDB: Cannot allocate memory
```

**What I tried:**
- Docker restart
- ./stackvo.sh down && ./stackvo.sh up
```

#### ❌ Bad Example

```
MySQL is not working help me
```

### Question Template

```markdown
## Problem Title

**Environment:**
- OS: [Ubuntu/macOS/Windows]
- Docker: [version]
- Stackvo: [version]

**Problem Description:**
[Detailed description]

**Steps:**
1. [Step 1]
2. [Step 2]

**Expected Behavior:**
[What did you expect to happen?]

**Actual Behavior:**
[What happened?]

**Error Message:**
```
[Error message]
```

**Logs:**
```
[Relevant logs]
```

**What I tried:**
- [Trial 1]
- [Trial 2]
```

---

## Bug Report Guide

### How to Report a Bug?

1. **Search first:** Has the same bug been reported before?
2. **Reproduce:** Can you reproduce the bug?
3. **Minimal example:** Show it in the simplest way
4. **Environment:** Add system information
5. **Logs:** Share relevant logs

### Bug Report Template

```markdown
## Bug Description

[Short and clear description]

## Reproduction Steps

1. [Step 1]
2. [Step 2]
3. [Step 3]

## Expected Behavior

[What should have happened?]

## Actual Behavior

[What happened?]

## Screenshots

[Screenshots if available]

## Environment

- **OS:** Ubuntu 22.04
- **Docker:** 24.0.7
- **Docker Compose:** 2.23.0
- **Stackvo:** 1.0.0
- **Browser:** Chrome 120 (For Web UI)

## Logs

```bash
# stackvo doctor
[Output]

# Container logs
docker logs stackvo-mysql
[Logs]

# Generator log
cat core/generator.log
[Logs]
```

## Additional Information

[Other relevant information]
```

---

## 💡 Feature Request Guide

### How to Suggest a Feature?

1. **Search:** Is there a similar suggestion?
2. **Use case:** Why is it necessary?
3. **Solution:** How should it be implemented?
4. **Alternatives:** Other solutions?

### Feature Request Template

```markdown
## Feature Description

[Briefly describe the feature]

## Motivation

[Why is this feature necessary?]

## Use Case

[In which scenarios will it be used?]

**Example:**
```
[Code example]
```

## Proposed Solution

[How should it be implemented?]

## Alternatives

[Other ways of solution?]

## Additional Information

[Other relevant information]
```

---

## Contributing

Do you want to contribute to Stackvo?

[Contributing Guide →](contributing.md)

**Contribution Areas:**
- 💻 Code
- 📝 Documentation
- 🧪 Testing
- 🌍 Translation
- 🎨 Design
- 📢 Community

---

## Support Stats

<div class="grid cards" markdown>

-   **🐛 Open Issues**
    
    GitHub Issues
    
    [Issues →](https://github.com/stackvo/stackvo/issues)

-   **💬 Discussions**
    
    Active discussions
    
    [Discussions →](https://github.com/stackvo/stackvo/discussions)

-   **👥 Contributors**
    
    Community support
    
    [Contributors →](index.md#contributors)

-   **📖 Documentation**
    
    Comprehensive guides
    
    [Docs →](../index.md)

</div>

---

## Response Times

**GitHub Issues:**
- First response: 24-48 hours
- Resolution: Depends on complexity

**GitHub Discussions:**
- Community support: Variable
- Maintainer support: 1-3 days

**Note:** Stackvo is an open source project. Response times are not guaranteed.

---

## Premium Support

Currently, premium support is not offered. All support is community-based.

---

## Contact

### GitHub

- **Repository:** [stackvo/stackvo](https://github.com/stackvo/stackvo)
- **Issues:** [Bug reports](https://github.com/stackvo/stackvo/issues)
- **Discussions:** [Q&A](https://github.com/stackvo/stackvo/discussions)
- **Pull Requests:** [Contributions](https://github.com/stackvo/stackvo/pulls)

### Email

- **General:** stackvo@example.com
- **Security:** security@stackvo.example.com

### Social Media

- **Twitter:** [@stackvo](https://twitter.com/stackvo)
- **LinkedIn:** [Stackvo](https://linkedin.com/company/stackvo)

---

## Security Issues

If you found a security vulnerability:

1. **Do not open a public issue**
2. **Send an email:** security@stackvo.example.com
3. **Provide details:** Vulnerability, impact, reproduce
4. **Wait:** Response within 48 hours

---
