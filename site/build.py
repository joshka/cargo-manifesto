#!/usr/bin/env python3
"""Build the static landing page using doctrine and amendment text from the Rust sources."""

import argparse
import hashlib
import html
import json
from pathlib import Path
import re
import shutil
import sys


SOURCE = Path(__file__).resolve().parent
ROOT = SOURCE.parent
DATA_PLACEHOLDER = "__MANIFESTO_DEMO_DATA__"
# `{{title:rule.id}}` and `{{text:rule.id}}` in index.html are replaced with catalogue text.
DOCTRINE_PLACEHOLDER = re.compile(r"\{\{(title|text):([a-z.-]+)\}\}")
ASSET_EXTENSIONS = {
    ".html", ".css", ".js", ".svg", ".png", ".webp", ".jpg", ".jpeg", ".avif",
    ".woff", ".woff2", ".ico", ".json", ".txt",
}

# Placeholder values for doctrines that name a package. The lockfile name matches this
# repository's own MANIFESTO.md; the others match the illustrative manifest in the demonstration.
NAMES = {
    "versions.exact": "serde",
    "lock.multiple": "winnow",
    "tooling.subcommand": "cargo-manifesto",
}
DEMO_RULES = [
    "dependencies.independence", "dependencies.cooperation",
    "versions.exact", "versions.compatible",
    "panic.unwind", "panic.abort",
    "edition.settled", "edition.current",
]


def rust_string(literal, context):
    """Decode one Rust string literal, honoring backslash-newline continuations."""
    literal = re.sub(r"\\\r?\n[ \t]*", "", literal)
    try:
        return json.loads(literal)
    except json.JSONDecodeError as error:
        raise ValueError(f"unsupported Rust string in {context}; update site/build.py") from error


STRING = r'("(?:[^"\\]|\\.)*")'


def read_doctrine(catalogue, rule_id):
    """Read literal title/text fields from one Rule or Combination, not its predicate."""
    blocks = re.findall(r"(?ms)^    (?:Rule|Combination) \{\n(.*?)^    \},", catalogue)
    matches = [block for block in blocks if f'        id: "{rule_id}",' in block]
    if len(matches) != 1:
        raise ValueError(f"expected one catalogue entry {rule_id!r}; found {len(matches)}")

    result = {}
    for field in ("title", "text"):
        literals = re.findall(rf"(?ms)^        {field}: {STRING},$", matches[0])
        if len(literals) != 1:
            raise ValueError(f"cannot read literal {field!r} for catalogue entry {rule_id!r}")
        value = rust_string(literals[0], f"{rule_id}.{field}")
        if "{name}" in value and rule_id not in NAMES:
            raise ValueError(f"catalogue entry {rule_id!r} needs a name in site/build.py")
        result[field] = value.replace("{name}", NAMES.get(rule_id, ""))
        if "{" in result[field]:
            raise ValueError(f"unfilled placeholder in {rule_id}.{field}; update site/build.py")
    return result


def read_amendments(amend):
    """Read the amendment titles and continuity sentence that `--amend` prints."""

    def reversal(rule_id):
        blocks = re.findall(r"(?ms)^    Reversal \{\n(.*?)^    \},", amend)
        matches = [block for block in blocks if f'"{rule_id}"' in block]
        if len(matches) != 1:
            raise ValueError(f"expected one amendment reversal for {rule_id!r}")
        fields = {}
        for field in ("title", "subject"):
            literal = re.search(rf"(?m)^        {field}: {STRING},$", matches[0])
            if not literal:
                raise ValueError(f"cannot read amendment {field!r} for {rule_id!r}")
            fields[field] = rust_string(literal.group(1), f"amendment {rule_id}.{field}")
        return fields

    responsibility = "Our Continuing Commitment to Responsibility"
    if f'"{responsibility}"' not in amend:
        raise ValueError("cannot find the dependency amendment title in src/amend.rs")
    continuity = re.search(r"(?ms)^fn continuity\(.*?format!\(\s*" + STRING, amend)
    if not continuity:
        raise ValueError("cannot read the continuity sentence in src/amend.rs")
    return {
        "dependencies": {"title": responsibility, "subject": "dependency declaration"},
        "versions": reversal("versions.exact"),
        "panic": reversal("panic.abort"),
        "edition": reversal("edition.current"),
        "continuity": rust_string(continuity.group(1), "amend continuity"),
    }


def build(output):
    if output == ROOT or output in ROOT.parents or output.is_relative_to(SOURCE):
        raise ValueError("the output directory must not overwrite the repository or site sources")

    template = (SOURCE / "index.html").read_text(encoding="utf-8")
    if template.count(DATA_PLACEHOLDER) != 1:
        raise ValueError(f"site/index.html must contain exactly one {DATA_PLACEHOLDER} placeholder")
    catalogue = (ROOT / "src/catalogue.rs").read_text(encoding="utf-8")
    amend = (ROOT / "src/amend.rs").read_text(encoding="utf-8")

    latest = re.search(r'(?m)^const LATEST_EDITION: &str = "(\d{4})";$', catalogue)
    if not latest:
        raise ValueError("cannot read LATEST_EDITION in src/catalogue.rs")
    template = template.replace("{{latest-edition}}", latest.group(1))

    cited = {rule_id for _, rule_id in DOCTRINE_PLACEHOLDER.findall(template)}
    doctrines = {rule_id: read_doctrine(catalogue, rule_id) for rule_id in sorted(cited | set(DEMO_RULES))}
    data = {
        "doctrines": {rule_id: doctrines[rule_id] for rule_id in DEMO_RULES},
        "amendments": read_amendments(amend),
        "latestEdition": latest.group(1),
    }

    rendered = DOCTRINE_PLACEHOLDER.sub(
        lambda m: html.escape(doctrines[m.group(2)][m.group(1)], quote=False), template
    )
    # A literal '<' could close the application/json script element in HTML.
    encoded = json.dumps(data, ensure_ascii=False, separators=(",", ":")).replace("<", "\\u003c")
    rendered = rendered.replace(DATA_PLACEHOLDER, encoded)

    # Version the stylesheet and script by content so a cached copy never pairs with new HTML.
    for name in ("styles.css", "app.js"):
        digest = hashlib.sha256((SOURCE / name).read_bytes()).hexdigest()[:10]
        if rendered.count(f'"./{name}"') != 1:
            raise ValueError(f"site/index.html must reference ./{name} exactly once")
        rendered = rendered.replace(f'"./{name}"', f'"./{name}?v={digest}"')

    output.mkdir(parents=True, exist_ok=True)
    for asset in sorted(SOURCE.rglob("*")):
        if not asset.is_file() or asset.suffix.lower() not in ASSET_EXTENSIONS:
            continue
        if any(part.startswith(".") for part in asset.relative_to(SOURCE).parts):
            continue
        destination = output / asset.relative_to(SOURCE)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(asset, destination)
    (output / "index.html").write_text(rendered, encoding="utf-8")
    (output / ".nojekyll").touch()
    print(f"Built landing page with {len(doctrines)} catalogue doctrines: {output}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT / "dist", help="output directory (default: dist/)")
    args = parser.parse_args()
    try:
        build(args.out.resolve())
    except (OSError, ValueError) as error:
        print(f"site build failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
