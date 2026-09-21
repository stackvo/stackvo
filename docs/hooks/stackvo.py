"""
What the site takes from outside its own `docs_dir`, and what it fixes on
the way out.

Mounted into every build without being copied anywhere:

* `docs/help/<locale>/` — the application's own help documents, the ones the
  running app pulls from `main` when a card's **?** is pressed. They are the
  same files, read from the same place, so the site can never show a different
  sentence from the app. They appear under `help/`, titled from each file's
  own `# heading`, and are listed on `help/index.md` grouped by the prefix of
  their topic name (`page-dashboard-`, `page-`, `panel-`, `project-`,
  `settings-`) rather than in the navigation, where a hundred entries were
  most of every page's sidebar. Nothing is copied into `docs_dir`:
  `tests/help-topics.spec.js` fails on any file under `docs/help/<locale>/`
  that a card does not open.

* `docs/assets/` and `docs/screenshots/` — the logo, the stylesheets and the
  screenshots the README already carries, shared by both languages rather than
  checked in twice.

* The documents at the repository root — `CHANGELOG.md`, `SECURITY.md`,
  `PRIVACY.md`, `CODE_OF_CONDUCT.md`, `ARCHITECTURE.md`, `ACCESSIBILITY.md`
  and `LICENSE` — as pages, the same file in both languages with a note under
  the heading in the Turkish build (`extra.mounted.note`). Their relative
  links name files in the repository, which the site does not carry, so they
  are pointed at GitHub. The changelog is kept out of the search index (it was
  most of it) and every release but the newest is folded away.

`STACKVO_HELP_DIR` overrides where the help documents are read from. It exists
for the day this site moves to a repository of its own: a sparse checkout of
`stackvo/stackvo` with only `docs/help` in it, and this hook unchanged.

Fixed on the way out:

* Every page gets a `description` for its `<meta>` and Open Graph tags from
  its first paragraph when the front matter does not set one, so a shared
  link shows what the page is about rather than the site's one sentence.

* The language switcher and the `hreflang` links point at the same page in
  the other language, not at its front page. The theme renders them from
  `extra.alternate` as fixed addresses; the page's own path is appended here.

The landing page's download links are built here too, at build time, from
the latest release tag: `config.extra.release` carries `tag` and `version`,
and the template writes each installer's direct URL from its file-name
pattern. Asked of the releases API once per build — with `GITHUB_TOKEN` when
CI has one, because the anonymous limit is sixty requests an hour per
address and a page that asked at every visit would hit it from any office.
`STACKVO_RELEASE_TAG` pins it (offline builds, reproducibility); with neither
the tag nor a network, the links fall back to the releases page.
"""

from __future__ import annotations

import json
import logging
import os
import re
import urllib.request
from pathlib import Path

from mkdocs.config.defaults import MkDocsConfig
from mkdocs.structure.files import File, Files
from mkdocs.structure.pages import Page

DOCS = Path(__file__).resolve().parents[1]
REPO = DOCS.parent
RELEASES_API = "https://api.github.com/repos/stackvo/stackvo/releases/latest"
BLOB = "https://github.com/stackvo/stackvo/blob/main/"

log = logging.getLogger("mkdocs.plugins.stackvo")

# Topic-name prefix → key under `extra.help.sections` in the config. Order
# matters twice: the first prefix that matches wins, so the longer one is
# first; and the groups appear on the index page in this order.
GROUPS = (
    ("page-dashboard-", "dashboard"),
    ("page-", "pages"),
    ("panel-", "panels"),
    ("project-", "project"),
    ("settings-", "settings"),
)

# Repository-root documents → where they appear on the site.
MOUNTED = (
    ("CHANGELOG.md", "insiders/changelog.md"),
    ("SECURITY.md", "reference/project/security.md"),
    ("PRIVACY.md", "reference/project/privacy-policy.md"),
    ("CODE_OF_CONDUCT.md", "reference/project/code-of-conduct.md"),
    ("ARCHITECTURE.md", "reference/project/architecture.md"),
    ("ACCESSIBILITY.md", "reference/project/accessibility.md"),
    ("LICENSE", "reference/project/licence.md"),
)

# Filled by `on_files`, read by `on_page_markdown` for the help index.
_help_groups: dict[str, list[tuple[str, list[tuple[str, str]]]]] = {}


def _help_dir(config: MkDocsConfig) -> Path:
    root = Path(os.environ.get("STACKVO_HELP_DIR") or DOCS / "help")
    return root / config.theme["language"]


def _title(path: Path) -> str:
    """The document's own `# heading`; every help document opens with one."""
    with path.open(encoding="utf-8") as handle:
        first = handle.readline()
    match = re.match(r"#\s+(.+)", first)
    return match.group(1).strip() if match else path.stem


def _mount(files: Files, config: MkDocsConfig, base: Path, prefix: str, keep) -> None:
    for path in sorted(base.rglob("*")):
        if not path.is_file() or path.name.startswith(".") or not keep(path):
            continue
        uri = f"{prefix}/{path.relative_to(base).as_posix()}"
        files.append(File.generated(config, uri, abs_src_path=str(path)))


def _github_links(text: str) -> str:
    """Relative links — `tools/x.mjs`, `docs/help/README.md` — pointed at GitHub."""
    return re.sub(
        r"\]\((?!https?://|mailto:|#)([^)\s]+)\)",
        lambda m: f"]({BLOB}{m.group(1)})",
        text,
    )


def _fold_old_releases(text: str, keep: int = 2) -> str:
    """Every `## ` section after the first `keep` folded into a closed block.

    `Unreleased` and the newest release stay open; older ones are a click
    away, which keeps a page that is six thousand lines long readable.
    """
    parts = re.split(r"(?m)^(?=## )", text)
    head, sections = parts[0], parts[1:]
    folded = []
    for i, section in enumerate(sections):
        if i < keep:
            folded.append(section)
            continue
        title, _, body = section.partition("\n")
        title = title[3:].strip()
        body = "\n".join(f"    {line}" if line.strip() else "" for line in body.split("\n"))
        folded.append(f'??? note "{title}"\n\n{body}\n')
    return head + "".join(folded)


def _repo_document(config: MkDocsConfig, source: str, uri: str) -> File:
    """One repository-root document as a page, both languages, notes and all."""
    text = (REPO / source).read_text(encoding="utf-8")
    if not text.startswith("# "):
        # LICENSE is plain text; shown as it is, under a heading.
        text = f"# {source.title() if source != 'LICENSE' else 'Licence'}\n\n```text\n{text.rstrip()}\n```\n"
    text = _github_links(text)
    if source == "CHANGELOG.md":
        text = _fold_old_releases(text)
    note = (config.extra.get("mounted") or {}).get("note", "")
    if note:
        heading, _, rest = text.partition("\n")
        text = f'{heading}\n\n!!! note ""\n\n    {note}\n{rest}'
    if source == "CHANGELOG.md":
        text = "---\nsearch:\n  exclude: true\n---\n" + text
    file = File.generated(config, uri, content=text)
    # `edit_uri` is joined onto the config's `edit/main/docs/content/<locale>/`;
    # three steps up is the repository root.
    file.edit_uri = f"../../../{source}"
    return file


def _latest_release() -> dict | None:
    """The latest release's tag, or None when it cannot be known."""
    tag = os.environ.get("STACKVO_RELEASE_TAG", "").strip()
    if not tag:
        request = urllib.request.Request(
            RELEASES_API, headers={"Accept": "application/vnd.github+json"}
        )
        token = os.environ.get("GITHUB_TOKEN", "").strip()
        if token:
            request.add_header("Authorization", f"Bearer {token}")
        try:
            with urllib.request.urlopen(request, timeout=10) as response:
                tag = json.load(response)["tag_name"]
        except Exception as error:  # noqa: BLE001 — any failure means "unknown"
            # INFO, not WARNING: `--strict` turns a warning into a failed
            # build, and a rate-limited API is not a reason to fail one — the
            # links degrade to the releases page, which still works.
            log.info(
                "stackvo: latest release unknown (%s); download links point at "
                "the releases page. Set STACKVO_RELEASE_TAG or GITHUB_TOKEN.",
                error,
            )
            return None
    return {"tag": tag, "version": tag.lstrip("v")}


def on_config(config: MkDocsConfig) -> MkDocsConfig:
    config.extra["release"] = _latest_release()
    return config


def on_files(files: Files, config: MkDocsConfig) -> Files:
    locale = config.theme["language"]
    help_dir = _help_dir(config)
    if not help_dir.is_dir():
        raise FileNotFoundError(
            f"no help documents at {help_dir}; set STACKVO_HELP_DIR to the "
            "checkout's docs/help directory"
        )

    sections: dict[str, list[tuple[str, str]]] = {key: [] for _, key in GROUPS}
    unplaced: list[str] = []
    for path in sorted(help_dir.glob("*.md")):
        uri = f"help/{path.name}"
        file = File.generated(config, uri, abs_src_path=str(path))
        # The edit link must point at the document, not at where the site
        # pretends it is. `edit_uri` is joined onto the config's
        # `edit/main/docs/content/<locale>/`, and `urljoin` folds the `..`.
        file.edit_uri = f"../../help/{locale}/{path.name}"
        files.append(file)
        for prefix, key in GROUPS:
            if path.name.startswith(prefix):
                sections[key].append((_title(path), path.name))
                break
        else:
            unplaced.append(path.name)
    # A document with a prefix this file does not know would be built and not
    # listed anywhere. Say so: the fix is a line in GROUPS and a label in both
    # configs.
    if unplaced:
        raise ValueError(
            "help documents with no group (add the prefix to GROUPS in "
            f"{__file__} and a label under extra.help.sections): "
            + ", ".join(unplaced)
        )
    labels = config.extra["help"]["sections"]
    _help_groups[locale] = [(labels[key], sections[key]) for _, key in GROUPS if sections[key]]

    for source, uri in MOUNTED:
        files.append(_repo_document(config, source, uri))
    _mount(files, config, DOCS / "assets", "assets", lambda p: True)
    _mount(files, config, DOCS / "screenshots", "screenshots", lambda p: p.suffix != ".md")
    return files


_ICON = re.compile(r":[a-z0-9_-]+:")
_LINK = re.compile(r"!?\[([^\]]*)\]\([^)]*\)")
_MARK = re.compile(r"[*_`]+")


def _first_paragraph(markdown: str) -> str:
    lines = markdown.split("\n")
    paragraph: list[str] = []
    for line in lines:
        stripped = line.strip()
        if paragraph and not stripped:
            break
        if not stripped or stripped.startswith(("#", "!!!", "???", "<", "|", "-", "*", "```", "===", ":", "1.", "[")):
            if paragraph:
                break
            continue
        paragraph.append(stripped)
    text = " ".join(paragraph)
    text = _LINK.sub(r"\1", text)
    text = _ICON.sub("", text)
    text = _MARK.sub("", text)
    text = re.sub(r"\s+", " ", text).strip()
    if len(text) > 160:
        text = text[:157].rsplit(" ", 1)[0] + "…"
    return text


def on_page_markdown(markdown: str, page: Page, config: MkDocsConfig, files: Files) -> str:
    if page.file.src_uri == "help/index.md":
        groups = _help_groups.get(config.theme["language"], [])
        listing = []
        for label, entries in groups:
            listing.append(f"\n## {label}\n")
            listing.extend(f"- [{title}]({name})" for title, name in entries)
        markdown = markdown.rstrip() + "\n" + "\n".join(listing) + "\n"

    if not page.meta.get("description"):
        hero = page.meta.get("hero") or {}
        text = hero.get("text") if isinstance(hero, dict) else None
        page.meta["description"] = re.sub(r"\s+", " ", text).strip() if text else _first_paragraph(markdown)
    return markdown


def on_post_page(output: str, page: Page, config: MkDocsConfig) -> str:
    for alt in config.extra.get("alternate") or []:
        output = output.replace(
            f'href="{alt["link"]}" hreflang="{alt["lang"]}"',
            f'href="{alt["link"]}{page.url}" hreflang="{alt["lang"]}"',
        )
    return output
