# Project Baca — Asset Catalog

Visual assets and illustrations following the **Vintage Literary / Mid-Century Writer (1900–1950)** aesthetic (pen-and-ink cross-hatch engravings).

## 1. Interface Illustrations (`assets/illustrations/`)

PNG files are the originals (640 px); the `.webp` siblings (quality 82) are what `crates/web/index.html` copies into the build. Regenerate after editing a PNG: `magick in.png -quality 82 -define webp:method=6 in.webp`.

| File | Subject | UI Placement |
|---|---|---|
| `library-bookshelf-ladder.png` | Reader climbing a wooden library ladder | **Home hero plate** (wide screens) |
| `cozy-reader-armchair-owl.png` | Reader in an armchair with tea and an owl | **Shelf** (guest empty state) |
| `manuscript-inspection-clothesline.png` | Writer drying manuscript pages on a line | Reserved (not shipped) |
| `admin-sorting-pigeonholes.png` | Archivist sorting mail into wooden pigeonholes | **Admin** (non-curator empty state) |
| `retro-rocket-discovery.png` | Victorian explorers launching in a retro rocket | **Catalog** (no results) |
| `rotaria-windmill.svg` | Watermill barn by the river (pen-and-ink line, terracotta hub) | Detailed brand plate for onboarding and email headers; too fine for 24–36 px |
| `rotaria-mark.svg` | Simplified seven-stroke mark on a paper tile | **Favicon and PWA tile**; the header draws the same mark inline (`icons::brand_mark`) in the current text color |

## 2. Design References (`assets/references/`)

* `gbrain-design-reference.png`: Editorial reference screenshot (`#1F1916` deep espresso, `#CE734E` terracotta clay, lyrical italics, and `❖` fleuron dividers).
