# Peanut site source

Static HTML/CSS, no build step, no external CDNs. Four pages: `index.html`,
`results.html`, `benchmarks.html`, `about.html`, sharing `style.css` and the
logo/favicon in `assets/`.

## Status: NOT enabled

GitHub Pages is not turned on for this repository yet. This directory is a draft
for Andrew to review before publishing.

## To enable later

GitHub repo → Settings → Pages → Build and deployment → Source: "Deploy from a
branch" → Branch: `main`, folder: `/docs/site` → Save.

This must happen on `main`; the draft currently lives on the `site-draft` branch
and needs merging first.

## Preview locally

```
cd docs/site
python3 -m http.server 8000
```

Then open `http://localhost:8000/`.
