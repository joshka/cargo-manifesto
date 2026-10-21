#!/usr/bin/env python3
"""Build the static landing page using the six doctrines from the Rust catalogue."""

import argparse
import json
from pathlib import Path
import re
import shutil
import sys


SOURCE = Path(__file__).resolve().parent
ROOT = SOURCE.parent
PLACEHOLDER = "__MANIFESTO_DEMO_DATA__"
ASSET_EXTENSIONS = {
    ".html", ".css", ".js", ".svg", ".png", ".webp", ".jpg", ".jpeg", ".avif",
    ".woff", ".woff2", ".ico", ".json", ".txt",
}


def read_rule(catalogue, rule_id):
    """Read literal title/text fields from one known Rule, not its predicate."""
    blocks = re.findall(r"(?ms)^    Rule \{\n(.*?)^    \},", catalogue)
    matches = [block for block in blocks if f'        id: "{rule_id}",' in block]
    if len(matches) != 1:
        raise ValueError(f"expected one catalogue rule {rule_id!r}; found {len(matches)}")

    result = {}
    for field in ("title", "text"):
        literals = re.findall(
            rf'(?ms)^        {field}: ("(?:[^"\\]|\\.)*"),$', matches[0]
        )
        if len(literals) != 1:
            raise ValueError(f"cannot read literal {field!r} for catalogue rule {rule_id!r}")
        # Rust's backslash-newline continuation removes the newline and indentation.
        literal = re.sub(r"\\\r?\n[ \t]*", "", literals[0])
        try:
            result[field] = json.loads(literal).replace("{name}", "serde")
        except json.JSONDecodeError as error:
            raise ValueError(
                f"unsupported Rust string in {rule_id}.{field}; update site/build.py"
            ) from error
    return result


def demo_data(catalogue):
    def choice(label, code, rule_id):
        return {"label": label, "code": code, **read_rule(catalogue, rule_id)}

    return {
        "scenarios": [
            {
                "id": "dependencies",
                "label": "Dependencies",
                "choices": [
                    choice("No dependencies", "[dependencies]", "dependencies.independence"),
                    choice("Add serde", '[dependencies]\nserde = "1"', "dependencies.cooperation"),
                ],
            },
            {
                "id": "versions",
                "label": "Version policy",
                "choices": [
                    choice("Pin exactly", '[dependencies]\nserde = "=1.0.229"', "versions.exact"),
                    choice("Allow compatible", '[dependencies]\nserde = "1"', "versions.compatible"),
                ],
            },
            {
                "id": "panic",
                "label": "Failure policy",
                "choices": [
                    choice("Abort", '[profile.release]\npanic = "abort"', "panic.abort"),
                    choice("Unwind", '[profile.release]\npanic = "unwind"', "panic.unwind"),
                ],
            },
        ]
    }


def build(output):
    if output == ROOT or output in ROOT.parents or output.is_relative_to(SOURCE):
        raise ValueError("the output directory must not overwrite the repository or site sources")

    template = (SOURCE / "index.html").read_text(encoding="utf-8")
    if template.count(PLACEHOLDER) != 1:
        raise ValueError(f"site/index.html must contain exactly one {PLACEHOLDER} placeholder")
    catalogue = (ROOT / "src/catalogue.rs").read_text(encoding="utf-8")
    # A literal '<' could close the application/json script element in HTML.
    data = json.dumps(demo_data(catalogue), ensure_ascii=False, separators=(",", ":"))
    rendered = template.replace(PLACEHOLDER, data.replace("<", "\\u003c"))

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
    print(f"Built landing page with six catalogue doctrines: {output}")


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
