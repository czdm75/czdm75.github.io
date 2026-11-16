# mdBook theme sources

Upstream: https://github.com/rust-lang/mdBook/tree/9ddcdcf233040441a0bdaf983ac30c951f294278/crates/mdbook-html/front-end

Imported from `main` at commit `9ddcdcf233040441a0bdaf983ac30c951f294278`.
The upstream source is licensed under Mozilla Public License 2.0; see LICENSE.

- `variables.css`: unmodified upstream `css/variables.css`, including the five palettes.
- `chrome.css`: extracted menu bar and theme popup sections of upstream `css/chrome.css`.
- `templates/page.hbs`: adapted from upstream `templates/index.hbs`; uses this project's
  PageView, shared partials and optional site tree/page outline instead of mdBook's
  chapter loader and renderer context. Fonts, editor, highlighting, search and print
  controls are omitted.
- `main.css`: local shell layout and component styles adapting these upstream UI
  conventions to the site's existing navigation schema and two optional panels.
- `main.js`: local palette switching. Upstream `js/book.js` is not loaded because
  it also processes content/code blocks. Panel behavior reuses `theme/default/main.js`.

No upstream font files, font stylesheets, editor, searcher or highlighting libraries
are included. The shared content stylesheet retains the existing site's typography;
see `../default/NOTICE.md` and `../default/LICENSE` for its historical sources.
