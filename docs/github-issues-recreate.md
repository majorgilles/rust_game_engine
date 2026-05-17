# GitHub issues recreation reference

This directory keeps a restorable snapshot of the repository issues.

- Source repo: `majorgilles/rust_game_engine`
- Exported at: `2026-05-17T10:24:08Z`
- Exported issues: 19 total (open and closed)
- Initial project commit: `854815b7729089fa7cf292a1bc151aae1ba27df2`
- Initial project tag: `project-start`

## Files

- `docs/github-issues-recreate.json` — text/JSON reference export with issue titles, bodies, comments, state, and original metadata.
- `scripts/recreate_github_issues.py` — helper script that reads the JSON export and recreates issues through GitHub CLI.

## Recreate issues from scratch

Use an empty target repository if you want recreated issue numbers to match the originals.

```bash
python scripts/recreate_github_issues.py --repo OWNER/REPO
```

Preview without creating anything:

```bash
python scripts/recreate_github_issues.py --repo OWNER/REPO --dry-run
```

The script creates issues in original numeric order, restores comments, and closes issues that were closed in the export. GitHub will assign the current user and current timestamps to recreated issues/comments; original authors and timestamps remain in the JSON reference.

## Restore code to the project start

```bash
git switch -c restore-from-start project-start
```

or inspect the tagged starting point directly:

```bash
git checkout project-start
```
