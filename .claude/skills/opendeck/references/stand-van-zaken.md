# Stand van zaken (bijgewerkt 9 november 2026)

## Branches en PR's

| Branch | Inhoud | PR |
|---|---|---|
| `main` | upstream OpenDeck 2.14.2 + projectdocs | |
| `feat/virtual-deck` | eerste versie: schermvullende deck, beelden via het hoofdvenster | #1 → `main` |
| `feat/virtual-deck-floating` | Jeremy: Floating-stand, doorzichtig, Settings, tray, deck tekent zelf. Plus Alwyn/Claude: `reconfigure_window`-fix | #2 → `feat/virtual-deck` |

Alwyn werkt lokaal op `feat/virtual-deck-floating`. Lokale commits zijn niet altijd gepusht: kijk naar `git status -sb`.

## Getest op Alwyns pc

- [x] Deck schermvullend op DISPLAY3 (5×8)
- [x] Floating start midden op het hoofdscherm
- [x] Wisselen Fill a whole monitor → Floating via Settings (na de `reconfigure_window`-fix)
- [ ] Actie toewijzen in het hoofdvenster en vanaf de deck triggeren
- [ ] Focus blijft in de actieve app/game bij een klik
- [ ] Lang indrukken
- [ ] Plugin-plaatjes en titels live bijwerken
- [ ] Na slaapstand / scherm uit-aan op de goede plek
- [ ] Installer (`deno task tauri build`) en starten met Windows

## Open punten uit reviews

| Punt | Status |
|---|---|
| Pluginpoort niet opgevraagd in deck-venster (`initPortBase`) | gefixt, nog niet gecommit (9 nov) |
| Schermvullend + doorzichtig blokkeert het scherm | gefixt, nog niet gecommit: monitor-stand altijd zwart |
| Minder knoppen wist acties zonder waarschuwing | gefixt, nog niet gecommit: bevestigingsvraag in Settings |
| Deck pollt het hele profiel elke 1,5 s (JSON incl. plaatjes) | open. Beter: event bij profielwijziging |
| `PLACING`-vlag houdt automatische plaatsing niet tegen | open, klein |
| `watch_monitors` vergelijkt exact in monitor-stand; kan bij DPI-verschil blijven verplaatsen | open. Let op herhaalde "Repositioning virtual deck window" in de log |
| Node.js ontbreekt (veel Elgato-plugins) | open, Alwyn moet het installeren |
| Elgato-software draait nog mee | open, advies: autostart uitzetten |

## Ideeën die Alwyn al gehoord heeft

Niet opnieuw voorstellen alsof ze nieuw zijn. Wel oppakken als hij erom vraagt:
- Elgato-profielen/-plugins overzetten uit `%APPDATA%\Elgato\StreamDeck\Plugins`
- Mappen/pagina's op de deck
- Automatisch van profiel wisselen per actieve app (OpenDeck kan dit al: koppelen per profiel)
- Eigen plugins: OBS-scènes/status, volume per app (Focusrite/Windows-mixer), Discord mute/deafen, Spotify met
  albumhoes, pc-status (CPU/GPU-temperatuur, FPS)
- Grote knoppen (één actie over 2×2)
- Knop-editor op de deck zelf (rechtsklik)
- Deck verbergen tijdens fullscreen-games / terug met een sneltoets
