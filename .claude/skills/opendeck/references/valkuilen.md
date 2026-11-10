# Valkuilen (elk heeft al eens tijd gekost)

## Tauri-vensters op Windows

**Destroy + create met hetzelfde label.** `window.destroy()` wordt pas later door de event-loop verwerkt. Bouw je
direct daarna een nieuw venster met label `virtual-deck`, dan faalt dat ("label already exists") en is de deck
weg. Dit gebeurde bij het wisselen tussen Floating en Fill a whole monitor. Oplossing (`reconfigure_window`):
`set_resizable`, `set_always_on_top`, `set_shadow`, `place_window` en `show` op het bestaande venster. Uitzetten
= `hide()`.

**Doorzichtig is niet klik-door.** `.transparent(true)` maakt het venster visueel doorzichtig, maar het is geen
`WS_EX_LAYERED`-venster. Windows stuurt klikken op doorzichtige pixels dus nog steeds naar de deck. Een
doorzichtige schermvullende deck blokkeert zo het hele scherm. Daarom heeft monitor-stand altijd een zwarte
achtergrond en staat de doorzichtigheid-schuif alleen bij Floating.

**Niet focusbaar.** `.focusable(false)` geeft `WS_EX_NOACTIVATE` (ex-style bevat `0x8000000`). Slepen
(`startDragging`) en vergroten (`startResizeDragging`) werken toch. Haal het niet weg: anders gaan sneltoetsen
naar de deck in plaats van naar de game/OBS.

**Window-events komen later.** `Moved`/`Resized` arriveren pas na `set_position`/`set_size`, als `PLACING` al
weer `false` is. Een automatische plaatsing kan dus als "door de gebruiker gekozen" plek opgeslagen worden.
Meestal onschuldig, maar reken er niet op dat `PLACING` dat tegenhoudt.

**DPI.** De schermen hebben verschillende schaling (hoofdscherm ~150%, DISPLAY3 100%). Gebruik fysieke pixels
(`PhysicalPosition`/`PhysicalSize`) en zet eerst de positie, dan de grootte: verhuizen naar een scherm met andere
schaling kan het formaat veranderen.

**Capabilities.** Het deck-venster heeft een eigen `src-tauri/capabilities/virtual-deck.json` (core:default +
start-dragging/resize-dragging). Nieuwe window-API's vanuit de frontend hebben daar een permissie nodig.

## Frontend

**Poort van de webserver.** Plugin-plaatjes komen van `http://127.0.0.1:<PORT_BASE+2>/`. `getImage()` gebruikt
`ports.ts`, dat standaard 57116 aanneemt tot `initPortBase()` gedraaid heeft. Het hoofdvenster doet dat in
`+page.svelte`. Elk nieuw venster dat plugin-plaatjes toont moet dat ook doen.

**Body-achtergrond.** `app.html` geeft elke pagina een grijze body. Het deck-venster zet
`document.body.style.background = "transparent"` in `onMount`.

**Emit gaat naar iedereen.** `window.emit(...)` (bijv. `key_moved` en `update_state` via het hoofdvenster) is in
Tauri 2 een globale broadcast. Het deck-venster ontvangt die events dus ook. Gebruik `emit_to(WINDOW_LABEL, …)`
voor alleen de deck.

## Git

**Regeleinden.** Git for Windows staat systeembreed op `core.autocrlf=true`. De repo is LF en `deno task format`
schrijft LF. Daarna staan ~80 bestanden als "gewijzigd" zonder echte diff. Oplossing: `git config core.autocrlf false`.
Blijven ze daarna nog staan: `git add -A`. Bestanden zonder echte wijziging verdwijnen dan uit de status.

**PR-doel.** `gh pr create` zonder `--repo` kan naar upstream (nekename) wijzen. Altijd
`--repo AlwynDG/OpenDeck-Virtual --base <branch>`.

**Gestapelde PR's.** Jeremy's PR #2 heeft als basis `feat/virtual-deck` (PR #1), niet `main`. Let op de volgorde bij mergen.

## PowerShell en shells

- PowerShell 5: `2>&1` op native tools (git, deno, cargo) geeft rode `NativeCommandError`-tekst, ook bij succes.
  Kijk naar `$LASTEXITCODE` of gebruik Git Bash.
- Een `$LASTEXITCODE` na een pipeline (`cargo clippy | Select-String`) is die van cargo, goed bruikbaar.
- Add-Type met dezelfde klassenaam twee keer in één sessie faalt. Geef elke helper-klasse een unieke naam, of draai het script in een nieuw proces.
- Git Bash zet `cd` terug naar de startmap na een commando buiten de werkmap. Gebruik absolute paden (`/d/AI/OpenDeck-Virtual`).

## Omgeving bij Alwyn

- **De Claude-app schermt `%APPDATA%` af (9 nov 2026 ontdekt).** Alles wat je vanuit een Claude-sessie in
  `%APPDATA%` schrijft, en alles wat een daar gestarte OpenDeck schrijft, komt in
  `%LOCALAPPDATA%\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\opendeck` terecht. Binnen de sessie lijkt het
  gewoon in `%APPDATA%\opendeck` te staan; een OpenDeck die Alwyn zelf start ziet het niet. Gevolg in de door
  Claude gestarte OpenDeck: de webserver (poort 57118) geeft **403** op alle plugin-plaatjes en property inspectors
  (canonicalize wijst naar de Packages-map, buiten de config-map). Geen bug in de fork. Ontsnappen uit de afscherming
  (WMI e.d.) wordt terecht geweigerd. Werkwijze: knoppenprofielen, `deck.exe` en plugins staan in `D:\AI\deck-config`
  (D: wordt niet afgeschermd); Alwyn draait daar zelf `Installeer deck.cmd` vanuit Verkenner.
  Ook het register (HKCU) en `%LOCALAPPDATA%` zijn afgeschermd: wat je leest is de privékopie, niet Alwyns echte
  logs en instellingen. Iets dat buiten de afscherming moet draaien (een dev-build die zijn echte config moet zien,
  een registerwijziging die Alwyn vroeg) gaat via `run_in_terminal`: dat typt het commando in zijn eigen shell, met
  zijn goedkeuring.
- **Dev-build of geïnstalleerde versie?** Sinds 10 nov is de release geïnstalleerd. `Start OpenDeck.cmd` heet nu
  `Start testversie (alleen voor ontwikkelen).cmd`: het zwarte venster ís de dev-build en sluiten stopt hem. Alwyn
  verwarde dat met de echte app. Een dev-build zonder draaiende Vite (`tauri dev` gestopt) toont oude code; herladen
  geeft dan "localhost weigert verbinding".
- **computer-use** werkt wel met het volledige pad: `request_access` met `D:\...\target\debug\opendeck.exe` of
  `C:\Users\Alwyn\AppData\Local\opendeck\opendeck.exe`. Verkenner krijgt alleen tier "click" (geen slepen), dus
  vensterranden slepen en klikken onder het schermtoetsenbord lukt niet.

- **Schermtoetsenbord (osk.exe)**: maakt een topmost "Input Occlusion Window" (explorer.exe) over heel DISPLAY3.
  Daaronder is de deck onzichtbaar en niet klikbaar. Niet zelf sluiten, vraag het Alwyn.
- **Elgato Stream Deck (`StreamDeck.exe`)** draait nog en registreert ook `streamdeck://`. Bij rare deep-link- of
  sneltoetsproblemen: vraag of hij die wil afsluiten.
- **Upstream-meldingen**: OpenDeck checkt upstream-releases en meldt "new version available". Dat gaat over
  nekename/OpenDeck, niet over deze fork. Niet zomaar updaten: mergen gaat via `git merge upstream/main` op een branch.
