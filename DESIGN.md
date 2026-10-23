# Design

Visuellt system för Modellverkstan. Strategin följer PRODUCT.md: lugnt och neutralt gränssnitt, så att modellens egna färger bär betydelsen; en sak i taget; orsak bredvid verkan.

Scen: en högstadieelev vid en 13-tums Chromebook i ett ljust klassrum, medan läraren visar samma vy på en projektor som ska gå att läsa från sista bänkraden. Därför ljust tema, hög kontrast och kraftiga rubriker.

## Färg

Strategi: **Restrained** för gränssnittet + **Full palette** för modellnotationen. Neutralerna är svagt kalla (nyans ≈ 255°).

| Roll | Värde | Används till |
| --- | --- | --- |
| Ink | `#1A2233` | Brödtext, rubriker, primärknappar (fylld), valt verktyg |
| Ink 2 | `#4A5468` | Sekundär text, ikoner |
| Ink 3 | `#687186` | Hjälptexter, etiketter (AA på vitt) |
| Surface | `#FCFCFD` | Paneler, toppfält |
| Surface 2 | `#F2F4F8` | Verktygsfält, spår, valda flikars bakgrund |
| Canvas | `#F7F8FB` | Ritytan (med punktrutnät `#D5DBE5`) |
| Line | `#E3E7EE` | Avdelare, ramar |
| Line strong | `#CBD2DD` | Inmatningsfält, knappramar |
| Highlight | `#F5A524` | Markering på ytan och "gör så här"-tips (överstrykningspenna) |
| Success | `#1E8E5A` | Klart, träning |
| Error | `#D03A3A` | Fel |

Modellnotation (betyder alltid samma sak, och kompletteras alltid med form):

| Del | Kant | Fyllning | Form |
| --- | --- | --- | --- |
| Lager | `#2F6FDB` | `#EAF2FE`, nivå `#BCD5FA` | Rundad rektangel |
| Flöde | `#1F9D5C` | `#D5F0E2` | Tjockt rör med ventil |
| Konstant | `#D9741C` | `#FDEFDF` | Rundad rektangel med reglage-ikon |
| Hjälpvariabel | `#7353D1` | `#EFEAFC` | Pill |
| Loop R / B | `#D8642A` / `#2F6FDB` | – | Cirkelpil med bokstav |
| Pil + / − | `#1E8E5A` / `#D03A3A` | – | Tecknet + eller − i cirkel |

## Typografi

En familj: **Figtree** (400, 600, 700), med egui:s standardtypsnitt som reserv för symboler.

| Steg | Storlek | Vikt |
| --- | --- | --- |
| Display (startsidan) | 34 | 700 |
| Rubrik panel | 21 | 700 |
| Avsnittsrubrik | 17 | 600 |
| Brödtext | 16 | 400 |
| Etikett/knapp | 15 | 600 |
| Liten | 13.5 | 400 |
| Överrubrik | 12.5 | 700, versaler, Ink 3 |

## Form och rytm

- Avstånd: 4, 8, 12, 16, 24, 32, 48. Tätt inom en grupp (8), luftigt mellan grupper (24–32).
- Hörn: 8 kontroller, 12 kort och paneler, helt runda chips.
- Inga kort i kort. Paneler delas med luft och tunna linjer, inte med ramade rutor.
- Skuggor bara för det som ligger ovanpå (flytande kontroller, valt verktyg, delar på ytan): mjuka och svaga.

## Komponenter

- **Primärknapp:** fylld Ink, ljus text. Högst en per vy (t.ex. Spela upp, Nästa i träningen).
- **Vanlig knapp:** Surface med Line strong-ram. **Diskret knapp:** ingen ram, bakgrund vid hover.
- **Segmentkontroll:** spår i Surface 2, valt segment som upphöjd Surface-platta.
- **Chips:** för grafens serier (färgprick + namn), ämnesfilter och jämförelser.
- **Verktyg:** ikon + kort etikett; valt verktyg är en upphöjd platta med Ink-ram; tips från träningen ger Highlight-ram.

## Layout

- Toppfält 52 px: logotyp, modellens namn, ångra/gör om, öppna/spara, hjälp.
- Vänster verktygsrad 84 px. Zoom ligger som flytande kontroll nere till höger på ytan.
- Höger panel 330 px: **antingen** Om modellen **eller** det markerade (inspektör); träningen ligger överst när den pågår. Loopar för orsaksdiagram.
- Resultatpanel nertill (flödesmodeller): **reglage till vänster, grafen till höger** – orsak bredvid verkan.
- Fel visas som ett band överst på ytan, inte som en extra ruta.
