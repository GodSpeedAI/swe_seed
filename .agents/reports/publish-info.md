`.omc/` and `.serena/` should be formatted, ignored by Prettier, and removed from tracked source.



Actually uv publish to PyPI (needs UV_PUBLISH_TOKEN / trusted publishing).
Add a CI workflow that runs uv build + docker build on PRs to catch build-system regressions.
A [project.urls] block (homepage/source/issues) for a nicer PyPI page.
