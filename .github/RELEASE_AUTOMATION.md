# Release Automation Guide

## 🤖 Automated Release System

This repository uses an intelligent automated release system that analyzes commit messages and
automatically creates releases based on [Conventional Commits](https://www.conventionalcommits.org/)
patterns.

## 🚀 How It Works

### Automatic Triggers

- **Every push to `main`** - Analyzes commits since last release
- **Smart detection** - Automatically determines release type based on commit messages
- **Conditional execution** - Only creates releases when meaningful changes are detected

### Release Type Detection

| Commit Pattern | Release Type | Version Bump | Example |
|----------------|--------------|--------------|---------|
| `feat:` or `feat()` | **Minor** | 0.1.0 → 0.2.0 | `feat: add certificate rotation support` |
| `fix:` or `fix()` | **Patch** | 0.1.0 → 0.1.1 | `fix: resolve ARM64 compilation issue` |
| `BREAKING CHANGE` or `!` | **Major** | 0.1.0 → 1.0.0 | `feat!: redesign configuration API` |
| `chore:`, `docs:`, `style:` | **Patch** | 0.1.0 → 0.1.1 | `chore: update dependencies` |
| `perf:`, `refactor:` | **Patch** | 0.1.0 → 0.1.1 | `perf: optimize certificate generation` |

### What Happens Automatically

1. **📊 Commit Analysis** - Scans commits since last release
2. **🏷️ Version Bumping** - Updates `Cargo.toml` and `Cargo.lock`
3. **✅ Quality Checks** - Runs tests, clippy, and builds
4. **📝 Release Notes** - Generates comprehensive release notes
5. **🏗️ Git Operations** - Commits changes, creates tags, pushes
6. **🚀 GitHub Release** - Creates release with artifacts
7. **⚙️ Build Trigger** - Starts multi-platform binary builds

## 📋 Usage Examples

### Automatic Releases

```bash
# This creates a minor release (v0.1.0 → v0.2.0)
git commit -m "feat: add Slack notification support"
git push origin main

# This creates a patch release (v0.1.0 → v0.1.1)
git commit -m "fix: resolve certificate renewal bug"
git push origin main

# This creates a major release (v0.1.0 → v1.0.0)
git commit -m "feat!: redesign sidecar architecture

BREAKING CHANGE: Configuration format has changed"
git push origin main
```

### Manual Override

Sometimes you need manual control:

```bash
# Via GitHub Actions UI:
# 1. Go to Actions → Automated Release → Run workflow
# 2. Choose release type: patch/minor/major/skip
# 3. Add optional release notes

# Or via GitHub CLI:
gh workflow run auto-release.yml \
  -f release_type=minor \
  -f release_notes="Custom release with special features"
```

## 🎯 Best Practices

### Commit Message Guidelines

**✅ Good Examples:**

```bash
feat: add health check endpoint
fix: resolve Docker build timeout
docs: update deployment guide
perf: optimize certificate parsing
feat!: change configuration file format
```

**❌ Avoid These:**

```bash
Update stuff
Fix things
Changes
WIP
Test commit
```

### Release Workflow

1. **Develop features** on feature branches
2. **Use conventional commits** when merging to main
3. **Let automation handle** version bumping and releases
4. **Monitor Actions** for any build failures
5. **Verify releases** are created correctly

## 🛠️ Configuration

### Environment Variables

The automated system respects these settings:

```yaml
# .github/workflows/auto-release.yml
GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}  # Required for releases
```

### Customization

To modify the automation behavior, edit:

- `.github/workflows/auto-release.yml` - Main automation logic
- `.github/workflows/release.yml` - Binary build pipeline

## 🚨 Troubleshooting

### Common Issues

**Release not triggered:**

- Check commit message follows conventional patterns
- Verify push was to `main` branch
- Review Actions logs for analysis results

**Build failures:**

- Check test suite passes locally: `cargo test`
- Verify clippy passes: `cargo clippy`
- Ensure Cargo.lock is up to date

**Manual intervention needed:**

- Use workflow_dispatch to override automatic detection
- Set release_type to 'skip' to prevent unwanted releases

### Emergency Procedures

**Stop automatic releases:**

```bash
# Disable via workflow_dispatch
gh workflow run auto-release.yml -f release_type=skip
```

**Manual release creation:**

```bash
# Traditional manual process
git tag v0.1.2
git push origin v0.1.2
# Then create release via GitHub UI
```

## 📊 Monitoring

### Success Indicators

- ✅ Version bumped in Cargo.toml
- ✅ Git tag created and pushed
- ✅ GitHub release created
- ✅ Binary artifacts building
- ✅ Release notes generated

### Failure Recovery

If automation fails:

1. **Check Actions logs** for specific error
2. **Fix the issue** locally
3. **Manual release** if needed
4. **Update automation** to prevent recurrence

---

**Pro Tip:** Use conventional commits consistently for best results.
The system gets smarter over time! 🎯
