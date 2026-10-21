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

The interactive demonstration contains three fixed scenarios: dependencies, version requirements,
and release panic strategy. `build.py` reads the six opposing doctrines directly from
`src/catalogue.rs` and embeds their titles and text in the page. It does not execute Cargo or
implement the catalogue's predicates. If the six rule IDs or their literal fields change format,
the build fails with an explanation so the extractor can be updated.

Edit presentation and page copy in `index.html`, `styles.css`, and `app.js`. Edit the demonstrated
doctrines in the Rust catalogue, then regenerate the project's normal generated documentation as
described in the root `AGENTS.md`. Do not maintain a second copy of that prose in the JavaScript.

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
