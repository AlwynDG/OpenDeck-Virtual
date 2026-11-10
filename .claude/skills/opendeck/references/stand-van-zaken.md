# Stand van zaken (bijgewerkt 10 november 2026)

## Branches en PR's

| Branch | Inhoud | PR |
|---|---|---|
| `main` | upstream OpenDeck 2.14.2 + projectdocs | |
| `feat/virtual-deck` | eerste versie: schermvullende deck, beelden via het hoofdvenster | #1 → `main` |
| `feat/virtual-deck-floating` | Jeremy: Floating-stand, doorzichtig, Settings, tray, deck tekent zelf. Plus Alwyn/Claude: `reconfigure_window`-fix en de fixes van 10 nov (zie hieronder) | #2 → `feat/virtual-deck` |

Alwyn werkt lokaal op `feat/virtual-deck-floating`. Lokale commits zijn niet altijd gepusht: kijk naar `git status -sb`.
Op 10 nov stonden er 6 commits lokaal die nog niet op GitHub staan (niet pushen zonder dat Alwyn erom vraagt).

## Gedaan op 10 november 2026

- **Hoofdvenster vergrootbaar**: `DeviceSelector.svelte` zette de minimumgrootte gelijk aan de ideale grootte en zette
  het venster bij elke `devices`-update terug. Nu alleen passen bij een andere indeling, minimum 800×600 (de knoppen
  scrollen). Bewuste wijziging in een upstream-bestand.
- **Floating deck vergrootbaar aan alle randen/hoeken**: onzichtbare banden van 8 px (`EDGE`) in
  `routes/virtual-deck/+page.svelte`. Zolang de deck vergrootbaar is, is de buitenmarge minstens `EDGE`, zodat de
  banden nooit over een knop vallen.
- **Sleepbalk weg** (titel, punaise, oog) op verzoek van Alwyn, inclusief de instelling `toolbar` en het commando
  `virtual_deck_set_always_on_top`. Oude `virtual_deck.json` met `toolbar` laadt gewoon (serde negeert onbekende velden).
- **Nieuwe instelling "Lock position and size"** (`locked`): Floating-deck niet verplaatsen of vergroten.
- **Nieuw logo** (blauwe "D" met 4 knoppen) in `src-tauri/icons` (`icon.ico` is Alwyns eigen multi-size ico) en
  `static/favicon.ico`.
- **Release gebouwd en geïnstalleerd**: `C:\Users\Alwyn\AppData\Local\OpenDeck\opendeck.exe`, autostart via
  `HKCU\...\Run` met `--hide` (bevestigd). Elgato Stream Deck en Elgato Volume Controller staan uit in het opstarten
  (StartupApproved, zoals Taakbeheer).
- Code-review gedaan; 6 van 7 punten opgelost.

## Getest op Alwyns pc

- [x] Deck schermvullend op DISPLAY3 (5×8)
- [x] Floating start midden op het hoofdscherm
- [x] Wisselen Fill a whole monitor → Floating via Settings (na de `reconfigure_window`-fix)
- [x] Actie toewijzen in het hoofdvenster en vanaf de deck triggeren (10 nov)
- [x] Hoofdvenster en deck vergroten/verkleinen, Lock aan/uit (10 nov)
- [x] Installer (`deno task tauri build`) en starten met Windows (10 nov)
- [ ] Focus blijft in de actieve app/game bij een klik
- [ ] Lang indrukken
- [ ] Na slaapstand / scherm uit-aan op de goede plek

## Knoppen van Alwyn (buiten de repo)

Staat in `D:\AI\deck-config`, niet in de repo (persoonlijke paden en links):
- `build-menus.ps1` maakt alle pagina's (5 breed × 3 hoog): startpagina met Windows, Gaming, Media, Communicatie, Apps;
  elk menu heeft **Terug linksonder**; Spotify-mapje in Media, Discord-mapje in Communicatie. Vergrendelen, Herstarten
  en Afsluiten staan rechtsonder op Windows (Herstarten/Afsluiten vragen eerst bevestiging). Lege plekken = plaatje
  "leeg" op een Open URL-actie zonder URL.
- Iconen in `iconen\OpenDeck` (uit Alwyns zip); eigen iconen in `iconen\OpenDeck\05 Eigen`.
  `make-playlist-icons.ps1` tekent playlist-iconen in zijn stijl. Ontbreekt een icoon: zeg het en geef een
  ChatGPT-prompt (donkere marineblauwe tegel #0E1B2B, dun lijnicoon, blauw verloop #7CC8FF → #2F80FF).
- `Installeer deck.cmd` (install.ps1) zet de pagina's in OpenDeck en start de geïnstalleerde OpenDeck weer op.
  `Alles wissen.cmd` (reset.ps1) zet alle knoppen in `backups`. Alwyn start ze zelf vanuit Verkenner.
- **Spotify is gekoppeld** (Essentials for Spotify, eigen developer-app "OpenDeck", redirect `http://127.0.0.1:6545`).
  Playlistknoppen gebruiken de actie Play Context (`spotify_url`).
- **Discord**: mute/deafen via Discords eigen sneltoetsen Ctrl+Shift+M / Ctrl+Shift+D (`deck.exe key`). Kanalen wachten
  op links van Alwyn. De Discord-plugin staat in `plugins-disabled` (niet gekoppeld).

## Open punten uit reviews

| Punt | Status |
|---|---|
| Pluginpoort niet opgevraagd in deck-venster (`initPortBase`) | gefixt (commit 4bccd33) |
| Schermvullend + doorzichtig blokkeert het scherm | gefixt: monitor-stand altijd zwart |
| Minder knoppen wist acties zonder waarschuwing | gefixt: bevestigingsvraag in Settings |
| Deck pollt het hele profiel elke 1,5 s (JSON incl. plaatjes) | open. Beter: event bij profielwijziging. Weegt zwaarder nu de knoppen plaatjes als data-URL hebben |
| `PLACING`-vlag houdt automatische plaatsing niet tegen | open, klein |
| `watch_monitors` vergelijkt exact in monitor-stand; kan bij DPI-verschil blijven verplaatsen | open. Let op herhaalde "Repositioning virtual deck window" in de log |
| OpenDeck heeft geen Nederlandse vertaling | gezien, Alwyn wil het niet |
| Node.js ontbreekt (veel Elgato-plugins) | opgelost: Node 26 staat erop |
| Elgato-software draait nog mee | opgelost: uit het opstarten gehaald (10 nov) |

## Ideeën die Alwyn al gehoord heeft

Niet opnieuw voorstellen alsof ze nieuw zijn. Wel oppakken als hij erom vraagt:
- Elgato-profielen/-plugins overzetten uit `%APPDATA%\Elgato\StreamDeck\Plugins`
- Automatisch van profiel wisselen per actieve app (OpenDeck kan dit al: koppelen per profiel)
- Eigen plugins: OBS-scènes/status, volume per app (Focusrite/Windows-mixer), pc-status (CPU/GPU-temperatuur, FPS)
- Grote knoppen (één actie over 2×2)
- Knop-editor op de deck zelf (rechtsklik)
- Op 10 nov afgewezen ("niet nuttig"): Nederlandse vertaling, sneltoets om de deck te tonen/verbergen, verbergen bij
  fullscreen-games, tray-klik toont de deck
