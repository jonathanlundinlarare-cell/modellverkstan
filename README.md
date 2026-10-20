# Modellverkstan

Ett enkelt verktyg för att bygga och testa modeller av system – gjort för teknikundervisning på högstadiet.
Tänk ett lättare Vensim: samma idéer (orsaksdiagram, lager och flöden, simulering), men med låg tröskel,
svenska och reglage som ändrar grafen direkt.

## Vad kan man göra?

- **Orsaksdiagram (CLD)** – rita variabler och pilar med + (samma riktning) eller − (motsatt riktning).
  Programmet hittar alla återkopplingsloopar och märker dem **R** (förstärkande) eller **B** (balanserande).
- **Flödesmodeller** – bygg med *lager* (något som samlas), *flöden* (som fyller på eller tömmer),
  *konstanter* (blir reglage) och *hjälpvariabler* (uträkningar). Modellen räknas ut direkt när något ändras.
- **Graf och tabell** – välj vad som ska visas, spela upp förloppet och se lagren fyllas, kopiera tabellen till Excel.
- **Exempel** – fylla ett badkar, värma ett hus med luft-luftvärmepump, ladda en elbil, termostaten och
  elbilar/laddstolpar som orsaksdiagram. Varje exempel har förklaring och "Prova detta!"-uppgifter.
- **Spara/öppna** modeller som `.mvm`-filer. Den senaste modellen sparas också automatiskt.

## Två sätt att köra

### 1. I webbläsaren (ingen installation)

Webbversionen är en vanlig mapp med statiska filer (`dist/`). Lägg den på valfri webbserver,
t.ex. GitHub Pages, skolans webbplats eller en lärplattform som tillåter HTML – eleverna öppnar bara länken.

Bygg den så här:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk build --release
```

Prova lokalt med `trunk serve --release` och öppna <http://127.0.0.1:8080>.
(Filen `index.html` i `dist/` fungerar inte om man dubbelklickar på den – den måste ligga på en webbserver.)

### 2. Som Windows-program

```bash
cargo build --release
```

Programmet hamnar i `target/release/modellverkstan.exe`. Det är en enda fil som inte behöver installeras –
kopiera den till eleverna och dubbelklicka.

## Formler i korthet

Skriv namnen på andra delar precis som de heter, t.ex. `Vatten i badkaret * Avloppets storlek`.

| Vad | Exempel |
| --- | --- |
| Räknesätt | `+ - * / ^` och parenteser |
| Jämförelser | `< > <= >= = <>`, `och`, `eller`, `inte` |
| Om-sats | `om(Laddnivå < 80; 11; 2)` |
| Funktioner | `min(A; B)`, `max(A; B)`, `abs(A)`, `rot(A)`, `avrunda(A)` |
| Tiden | `tid` |

Decimaler skrivs med punkt (`0.15`), eller med komma utanför funktioner (`0,15 * A`).

## Lektionsidéer

1. **Badkaret** – introducera lager och flöden. Varför stannar vattennivån? (jämvikt)
2. **Orsaksdiagram först** – låt eleverna rita termostatens orsaksdiagram innan de öppnar flödesmodellen.
3. **Värmepumpen** – vad betyder COP? Hur påverkar isoleringen elkostnaden? Klarar pumpen −20 °C?
4. **Elbilen** – jämför vägguttag (2,3 kW) med laddbox (11 kW). Varför laddar bilen långsammare över 80 %?
5. **Bygg själv** – t.ex. en solcell som laddar ett hembatteri, eller en vattentank med pump.

## För utvecklare

> **Windows med MinGW (gnu-verktygskedjan):** nyare GNU `ld` (binutils 2.45+) kan bygga program som
> kraschar direkt vid start. Länka då med Rusts medföljande lld, t.ex. i en lokal `.cargo/config.toml`:
>
> ```toml
> [target.x86_64-pc-windows-gnu]
> rustflags = ["-C", "link-arg=-fuse-ld=lld", "-C", "link-arg=-B<rustup-sysroot>/lib/rustlib/x86_64-pc-windows-gnu/bin/gcc-ld"]
> ```
>
> Samma sak gäller när man installerar `trunk` (`RUSTFLAGS` med samma flaggor).

- Rust + [egui/eframe](https://github.com/emilk/egui), samma kod för webb (WebAssembly) och program.
- `cargo test` kör tester för formeltolken, simuleringen, loopsökningen och exemplen.
- Koden: `expr.rs` (formeltolk), `sim.rs` (simulering), `model.rs` (datamodell, loopar),
  `canvas.rs` (ritytan), `panels.rs` (paneler), `examples.rs` (exempelmodeller).
