# Peanut — brand

Peanut is the mascot and name of the automatic-sequence engine in this repo
(`engine/target/release/peanut`). Created by **Andrew Hingston**.
MIT licensed, copyright 2026 Andrew Hingston.

The character is a chubby kawaii peanut: soft two-lobe silhouette, dot eyes, blushed
cheeks, gentle smile. Nothing sharp, nothing clever — the engine is doing the hard
part, the mascot's job is to be legible at 16 px and friendly at 512 px.

## Files

| File | Use |
|---|---|
| `gui/static/logo.svg` | primary full-colour mark, 256×256 viewBox |
| `gui/static/logo-mono.svg` | single-colour line mark; inherits `currentColor` |
| `gui/static/favicon.svg` | 32×32, cream rounded tile, no shell ridges |
| `gui/static/mascot-sprite.svg` | three expression symbols, hidden sprite |

Sprite symbols (`<use href="/static/mascot-sprite.svg#peanut-happy"/>`):

| Symbol | Engine state |
|---|---|
| `peanut-happy` | query answered `TRUE` — green sparkles |
| `peanut-thinking` | engine running — mint thought dots |
| `peanut-oops` | query answered `FALSE`, or the run hit the memory budget — fluster lines, sweat drop |

Never invent a fourth face by editing eyes ad hoc; add a symbol here instead.

## Palette derivation — PEANUT_DESIGN_SEED = 653658211

Every hue and saturation is a function of the seed; only lightness is chosen by role,
so the palette is reproducible from the number alone.

**Hues.** `H_i = (SEED >> 5i) mod 360`, i = 0..6:

    i  0    1   2   3    4    5   6
    H  91   59  58  148  263  19  0        (i=6: the 30-bit seed is shifted out → 0)

**Saturations.** decimal digits of the seed, `d = 6 5 3 6 5 8 2 1 1`;
`S_j = 30 + 7·d_j`  →  `72 65 51 72 65 86 44 37 37`.

**Lightness** is a fixed role ramp: surface 94/88, tint 80/78, mid 62/58, accent 42/45, ink 20/26.

| Token | HSL | Hex | Role |
|---|---|---|---|
| `--peanut-shell` | 19, 65%, 76% | `#EAB39A` | body fill |
| `--peanut-shadow` | 19, 72%, 62% | `#E48558` | shell ridges, warm shading |
| `--peanut-outline` | 19, 44%, 26% | `#5F3825` | body outline |
| `--peanut-ink` | 263, 44%, 20% | `#2E1D49` | eyes, mouth, body text |
| `--peanut-cream` | 59, 51%, 94% | `#F8F7E8` | page / tile background |
| `--peanut-highlight` | 59, 51%, 88% | `#F0EFD1` | shine, hairlines |
| `--peanut-true` | 91, 51%, 42% | `#69A234` | TRUE results, sparkles |
| `--peanut-think` | 148, 44%, 45% | `#40A56F` | running / in-progress |
| `--peanut-blush` | 0, 86%, 80% | `#F8A0A0` | cheeks |
| `--peanut-false` | 0, 72%, 58% | `#E14747` | FALSE results, budget failures |

The seed lands the base hue at 91° (pistachio), not on the shell — so the accent green is
the seeded colour and the tan is the seeded warm hue 19°. The green/indigo pair is what
keeps the mark off the usual cream-and-terracotta shelf.

## Type pairing

- **Display / wordmark:** Fraunces (variable, `wonk` 1, `SOFT` 40, weight 600). Soft-cornered
  optical serif with the same chubbiness as the mascot.
- **Body / UI:** Hanken Grotesk, 400/600. Round-shouldered grotesque, no Inter defaults.
- **Formulas, automaton tables, engine output:** Iosevka (fallback JetBrains Mono).
  Narrow enough that `A u,v. (u>=i & u<i+n & u+j=v+i) => T[u]=T[v]` fits on one line.

Wordmark: "Peanut" in Fraunces, lowercase-height mascot to the left of the cap-height P,
optical gap ≈ 0.4 em. Never set the wordmark in the mono face.

## Usage

- Minimum size 16 px; below 24 px use `favicon.svg`, which drops the ridges and blush.
- Clear space on all sides = height of the upper lobe (≈ 0.42 × mark height).
- On dark surfaces use `logo-mono.svg` with `color: var(--peanut-cream)`.
- Do not rotate, squash, recolour the shell, add limbs, or add a fourth expression.
- The mascot may bounce or blink; it does not spin.
- Respect `prefers-reduced-motion` — the thinking state falls back to a static symbol.

## Name

See the README's name line. Do not restate the origin of the name elsewhere.
