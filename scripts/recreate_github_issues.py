#!/usr/bin/env python3
"""Recreate GitHub issues from docs/github-issues-recreate.json.

The export file preserves original issue numbers, state, body text, comments,
and metadata. New issue numbers will only match the originals if the target
repository has no existing issues or pull requests.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


DEFAULT_ISSUES_FILE = Path(__file__).resolve().parents[1] / "docs" / "github-issues-recreate.json"


def run(args: list[str], *, dry_run: bool = False) -> str:
    print("+ " + " ".join(quote_arg(arg) for arg in args))
    if dry_run:
        return ""
    completed = subprocess.run(args, check=True, text=True, capture_output=True)
    if completed.stderr:
        print(completed.stderr, file=sys.stderr, end="")
    return completed.stdout.strip()


def quote_arg(arg: str) -> str:
    if not arg or any(ch.isspace() for ch in arg) or any(ch in arg for ch in '"\'`$\\'):
        return repr(arg)
    return arg


def write_temp_file(text: str) -> str:
    handle = tempfile.NamedTemporaryFile("w", encoding="utf-8", newline="\n", delete=False)
    try:
        handle.write(text or "")
        return handle.name
    finally:
        handle.close()


def load_issues(path: Path) -> list[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as file:
        payload = json.load(file)
    issues = payload["issues"] if isinstance(payload, dict) and "issues" in payload else payload
    return sorted(issues, key=lambda issue: issue.get("number", issue.get("original_number", 0)))


def issue_number(issue: dict[str, Any]) -> int:
    return int(issue.get("number", issue.get("original_number", 0)))


def close_reason(issue: dict[str, Any]) -> str:
    return "not planned" if issue.get("stateReason") == "NOT_PLANNED" else "completed"


def create_issue(repo: str, issue: dict[str, Any], args: argparse.Namespace) -> str:
    number = issue_number(issue)
    temp_paths: list[Path] = []

    def temp_issue_file(text: str) -> str:
        path = Path(write_temp_file(text))
        temp_paths.append(path)
        return str(path)

    try:
        body_file = temp_issue_file(issue.get("body") or "")
        command = [
            "gh",
            "issue",
            "create",
            "--repo",
            repo,
            "--title",
            issue.get("title") or f"Restored issue #{number}",
            "--body-file",
            body_file,
        ]

        if not args.skip_labels:
            for label in issue.get("labels") or []:
                label_name = label.get("name") if isinstance(label, dict) else str(label)
                if label_name:
                    command.extend(["--label", label_name])

        if args.include_assignees:
            for assignee in issue.get("assignees") or []:
                login = assignee.get("login") if isinstance(assignee, dict) else str(assignee)
                if login:
                    command.extend(["--assignee", login])

        if not args.skip_milestones:
            milestone = issue.get("milestone")
            milestone_title = milestone.get("title") if isinstance(milestone, dict) else milestone
            if milestone_title:
                command.extend(["--milestone", milestone_title])

        print(f"\nRestoring original issue #{number}: {issue.get('title', '')}")
        issue_url = run(command, dry_run=args.dry_run)
        if args.dry_run:
            issue_url = f"DRY-RUN-ISSUE-{number}"

        comments = sorted(issue.get("comments") or [], key=lambda comment: comment.get("createdAt") or "")
        for comment in comments:
            comment_body = comment.get("body") or ""
            if not comment_body:
                continue
            comment_file = temp_issue_file(comment_body)
            run(
                [
                    "gh",
                    "issue",
                    "comment",
                    issue_url,
                    "--repo",
                    repo,
                    "--body-file",
                    comment_file,
                ],
                dry_run=args.dry_run,
            )

        if issue.get("state") == "CLOSED":
            run(
                [
                    "gh",
                    "issue",
                    "close",
                    issue_url,
                    "--repo",
                    repo,
                    "--reason",
                    close_reason(issue),
                ],
                dry_run=args.dry_run,
            )

        return issue_url
    finally:
        for path in temp_paths:
            path.unlink(missing_ok=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Recreate GitHub issues from an exported JSON text file.")
    parser.add_argument("--repo", required=True, help="Target GitHub repository as OWNER/REPO.")
    parser.add_argument(
        "--issues-file",
        type=Path,
        default=DEFAULT_ISSUES_FILE,
        help=f"Issue export file. Defaults to {DEFAULT_ISSUES_FILE}",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print gh commands without creating issues.")
    parser.add_argument("--skip-labels", action="store_true", help="Do not reapply labels from the export.")
    parser.add_argument("--skip-milestones", action="store_true", help="Do not reapply milestones from the export.")
    parser.add_argument(
        "--include-assignees",
        action="store_true",
        help="Reapply assignees. Disabled by default because it requires target repo permissions.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    issues = load_issues(args.issues_file)
    print(f"Loaded {len(issues)} issue(s) from {args.issues_file}")
    print("Issues are recreated in original numeric order to preserve numbers in an empty target repo.")

    mapping: list[tuple[int, str]] = []
    for issue in issues:
        mapping.append((issue_number(issue), create_issue(args.repo, issue, args)))

    print("\nRestored issue mapping:")
    for original_number, new_url in mapping:
        print(f"  original #{original_number} -> {new_url}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
