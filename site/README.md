# Landing page

The landing page is built for <https://www.joshka.net/cargo-manifesto/>. Its source is plain
HTML, CSS, and JavaScript. Building it requires Python 3.9 or newer and no third-party packages.

## Build and preview

From the repository root:

```sh
python3 site/build.py
python3 -m http.server 8000 --directory dist
```

The build writes `dist/`, which is ignored by Git. A different output directory can be selected
with `python3 site/build.py --out /tmp/cargo-manifesto-preview`. The builder does not delete the
chosen directory. Use a fresh output directory if files have been removed from the source.

The register of amendments lets visitors change an illustrative manifest: the edition,
dependencies, the serde version requirement, and the release panic strategy. The latest edition
comes from `LATEST_EDITION` in the catalogue. `build.py` reads the article titles and
text from `src/catalogue.rs`, and the amendment titles and continuity sentence from `src/amend.rs`.
It does not execute Cargo or implement the catalogue's predicates. If those literals change format,
the build fails with an explanation so the extractor can be updated.

Static articles in `index.html` are written as `{{title:rule.id}}` and `{{text:rule.id}}`; the build
fills them from the catalogue, so the page works without JavaScript and never keeps a second copy of
the prose. Edit doctrines in the Rust catalogue, then regenerate the project's generated
documentation as described in the root `AGENTS.md`.

The palette follows `prefers-color-scheme` and is designed dark-first; the colour roles are
documented in `styles.css`. Check dark and light, desktop and mobile, before publishing. The build
adds content hashes to the stylesheet and script URLs. `social-card.png` is rendered from
`social-card.svg`; re-render it after editing the SVG.

## GitHub Pages

The `Pages` workflow builds pull requests without deploying them. Changes to the page or catalogue
on `main` publish automatically. A manual run publishes only when its selected branch is `main`.

One-time setup: in this repository's **Settings → Pages → Build and deployment**, set **Source** to
**GitHub Actions**. The workflow uses the `github-pages` environment and the standard Pages artifact
and OIDC deployment actions.

The account's user site already owns `www.joshka.net`, so this project site inherits the domain
and the `/cargo-manifesto/` path. It needs no project-specific custom domain, `CNAME` file, DNS
changes, or changes to the root website. Keep asset URLs compatible with that project path and
check both desktop and mobile layouts before publishing.
