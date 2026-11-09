---
name: opendeck
description: >-
  Werken aan OpenDeck-Virtual (D:\AI\OpenDeck-Virtual, GitHub AlwynDG/OpenDeck-Virtual), Alwyns fork van OpenDeck
  met een virtuele Stream Deck op het scherm in plaats van een fysieke Stream Deck of de Elgato-software.
  Gebruik deze skill bij ELK verzoek over dit project, ook als het niet zo genoemd wordt, zoals "de deck is weg",
  "knoppen zijn leeg", "zwevend/fullscreen werkt niet", "rechter scherm", Settings → Virtual Stream Deck,
  plugins, profielen, Jeremy's PR reviewen, bouwen, testen, installer maken of pushen naar GitHub.
  Bevat de architectuur, Alwyns schermopstelling, hoe je de deck controleert zonder computer-use,
  en valkuilen die al eens tijd hebben gekost.
---

# OpenDeck-Virtual

Fork van [nekename/OpenDeck](https://github.com/nekename/OpenDeck) (Tauri 2: Rust-backend + SvelteKit-frontend,
gebouwd met Deno). Doel: een **virtuele Stream Deck** die werkt als een echte. Alles van OpenDeck (profielen,
multi-actions, Elgato-plugins) moet gewoon werken. Hij heeft twee standen: **Floating** (los venster, verplaatsen en
vergroten) en **Fill a whole monitor** (vult Alwyns staande rechter scherm).

Lees ook `CLAUDE.md`, `AGENTS.md` (upstream-gids) en `docs/` in de repo. Deze skill vult die aan met wat in de
praktijk geleerd is.

## Wie is wie

- **Alwyn** (`AlwynDG`): eigenaar en enige gebruiker. Praat Nederlands en gebruikt vaak spraak-naar-tekst, dus
  woorden kunnen verkeerd overkomen ("scale" = skill). Is niet diep technisch: leg termen kort uit en antwoord in
  gewone woorden. Test zelf op zijn pc en meldt wat hij ziet.
- **Jeremy** (`Jeremy0538`): helpt mee met ontwikkelen, ook met Claude Code, op **Linux (Hyprland)**. Gebruikt de
  deck zelf niet. Zijn PR's zijn niet op Windows getest, dus controleer Windows-gedrag altijd zelf.
- Antwoord in het Nederlands. Code, commits en comments in het Engels (zoals upstream).

## Alwyns pc (alleen hier draait de deck echt)

| | |
|---|---|
| OS | Windows 11 Pro, PowerShell 5.1 en Git Bash |
| Hardware | Ryzen 9 9950X3D, 32 GB, RTX 4080 SUPER, geen fysieke Stream Deck |
| DISPLAY1 | AW3423DWF 3440×1440 op (0,0), hoofdscherm, schaling ~150% |
| DISPLAY2 | PL2466H 1920×1080 op (795,-1080), boven het hoofdscherm |
| **DISPLAY3** | **PL2466H 1080×1920 staand op (3440,-251)**, schaling 100%, doelscherm van de deck |
| Repo | `D:\AI\OpenDeck-Virtual` (`D:\AI\Jeremy` is een ander project, niet aanraken) |
| Config | `%APPDATA%\opendeck\` (`virtual_deck.json`, `profiles\vd-virtual-1\`, `settings.json`) |
| Logs | `%LOCALAPPDATA%\opendeck\logs\OpenDeck.log` (niet `%APPDATA%`, wat README zegt) en stdout van `tauri dev` |
| Ook geïnstalleerd | Elgato Stream Deck 7.5 (draait nog, werkt niet naar wens), OBS 32, schermtoetsenbord vaak open |
| Ontbreekt | Node.js (nodig voor veel Elgato-plugins) |

Toolchain: Rust (stable-msvc), Deno 2.9, VS 2022 C++ Build Tools. Een nieuwe shell kent ze soms nog niet. Zet
dan eerst het PATH: `$env:Path = [Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [Environment]::GetEnvironmentVariable('Path','User')`.

## Hoe de virtuele deck werkt

```
src-tauri/src/virtual_deck.rs
  register()        → register_device("", DeviceInfo{id:"vd-virtual-1", rows, columns, type:2 /*XL*/})
  create_window()   → WebviewWindow "virtual-deck": geen rand, transparent, focusable(false), skip_taskbar
  reconfigure_window() → stand/always-on-top/schaduw wijzigen op het BESTAANDE venster (zie valkuilen)
  watch_monitors()  → elke 3 s terugzetten als het venster niet meer op zijn plek / op een scherm staat
  virtual_deck_press(position, pressed) → inbound::devices::key_down / key_up (lang indrukken werkt)
  config            → virtual_deck.json, via Settings → Virtual Stream Deck (VirtualDeckSettings.svelte)

src/routes/virtual-deck/+page.svelte   het deck-venster
  laadt zelf het geselecteerde profiel (get_selected_profile, + poll elke 1,5 s)
  luistert naar update_state, key_moved, show_ok/show_alert, switch_profile, virtual_deck_profile/state/sleeping
  src/components/VirtualDeckKey.svelte tekent elke knop met renderImage() (zelfde renderer als het hoofdvenster)
```

- Het deck-venster tekent **zelf**. Het hangt dus niet af van het (vaak verborgen) hoofdvenster. De oude route,
  waarbij beelden van het hoofdvenster via `outbound::devices::update_image` doorgestuurd werden, is weg.
- `focusable(false)`: klikken op de deck stelen geen focus, zodat sneltoetsen naar de actieve app/game gaan.
- `start_in_tray` (standaard aan): OpenDeck zelf start verborgen. Openen gaat via het tray-icoon. Het tray-menu heeft
  "Show/hide virtual deck".
- Upstream-bestanden zo min mogelijk aanraken. Fork-code staat in `virtual_deck.rs`, `routes/virtual-deck/`,
  `VirtualDeckKey.svelte`, `VirtualDeckSettings.svelte` en `capabilities/virtual-deck.json`.
- Minder rijen/kolommen = `keys.resize` in de profielen = **acties gaan verloren**. De settings vragen daarom
  eerst om bevestiging.

## Veelvoorkomende verzoeken

### "De deck is weg / leeg / ik zie hem niet"
1. Draai `scripts/deck-status.ps1`. Dat laat zien of het proces draait, welke OpenDeck-vensters er zijn (zichtbaar,
   positie), wat `virtual_deck.json` zegt, en welk venster bovenop de deck ligt. Het maakt ook een screenshot in de scratchpad.
2. Ligt er een **"Input Occlusion Window" (explorer)** over DISPLAY3? Dan staat het Windows-schermtoetsenbord open
   en blokkeert dat de deck. Vraag Alwyn het te sluiten (sluit het niet zelf).
3. Bestaat er geen venster "Virtual Stream Deck"? Kijk in de log of `create_window` faalde, en of iets het venster
   vernietigd heeft.
4. Bestaat het venster maar is het zwart, zonder knoppen? Dan is waarschijnlijk de frontend niet geladen: kijk of
   `Virtual deck window loaded` in de log staat en of `rows`/`columns` binnenkomen.

### Bouwen en starten
- `scripts/dev.ps1` stopt een draaiende OpenDeck, start `deno task tauri dev` en schrijft de uitvoer naar een log
  in de scratchpad. Start het met `run_in_background` en wacht op `Virtual deck window loaded`.
- OpenDeck is **single-instance**: een tweede start doet niets zolang de eerste draait.
- Stop je `opendeck.exe`, dan stopt ook de `tauri dev`-taak (exit-code 1 is dan normaal).
- Een draaiende OpenDeck van Alwyn stop je alleen als dat nodig is om te testen. Zeg dat er even bij.

### Controleren vóór commit (verplicht, zoals upstream)
In `src-tauri/`: `cargo clippy` en `cargo fmt`. In de root: `deno task check`, `deno lint` en `deno task format`.
Alle vijf moeten schoon zijn.

### Testen op Alwyns schermen
- **computer-use werkt niet** met de dev-build: `request_access` vindt "opendeck" niet. Kijk met PowerShell:
  - DPI-aware maken (`SetProcessDpiAwarenessContext(-4)`), anders kloppen coördinaten niet.
  - `EnumWindows` + `GetWindowRect` voor de vensters, `WindowFromPoint` voor wat er bovenop ligt.
  - `Graphics.CopyFromScreen` voor een screenshot. DISPLAY3 = `CopyFromScreen(3440,-251, …)` met 1080×1920.
  - Pixels tellen (`GetPixel`) is betrouwbaarder dan een verkleinde screenshot: lege knoppen zijn rgb ~(23,23,23)
    op zwart en vallen in een verkleind plaatje weg.
- Klikken in de Settings en knoppen toewijzen in het hoofdvenster kun je niet zelf. Vraag Alwyn het te doen en te
  vertellen wat hij ziet.

### Jeremy's PR reviewen
1. `gh pr list --repo AlwynDG/OpenDeck-Virtual` en `gh pr view <n> --json body,commits,files`.
2. `git fetch origin <branch>` en lees de diff. Let op Windows-specifieke dingen (zie valkuilen).
3. Lokaal bouwen en de vijf controles draaien. Zet je werkmap daarna terug op de branch waar je op stond.
4. Rapporteer aan Alwyn in het Nederlands. Plaats **niets op GitHub** zonder dat hij erom vraagt.

### Committen, pushen en PR's
- Committen mag na een getest resultaat. **Pushen en PR's of reviews plaatsen alleen als Alwyn erom vraagt.**
- PR's gaan naar de fork: `gh pr create --repo AlwynDG/OpenDeck-Virtual --base <branch>`, nooit naar upstream.
- Commit-bericht via een heredoc in Git Bash (`git commit -F - <<'EOF'`) of een bestand. Eindig met de
  Co-Authored-By-regel uit de system-reminder. Conventional Commits (`feat:`, `fix:`, `docs:`).
- De repo is **publiek**: geen tokens, wachtwoorden of serienummers.

## Stand van zaken en ideeën

Zie `references/stand-van-zaken.md`: wat af is, wat nog open staat, eerdere review-bevindingen en ideeën die
Alwyn al gehoord heeft (niet opnieuw voorstellen alsof ze nieuw zijn).

## Valkuilen

Lees `references/valkuilen.md` vóór je vensters, git of PowerShell aanraakt. Kort:
- **Venster vernietigen en meteen opnieuw maken met hetzelfde label faalt op Windows**: de deck verdwijnt. Herconfigureer het bestaande venster.
- **Een doorzichtig venster vangt op Windows toch alle klikken.** Schermvullend daarom altijd met zwarte achtergrond.
- **`core.autocrlf` moet `false`**, anders lijkt na `deno task format` elk bestand gewijzigd.
- **PowerShell 5** zet stderr van native tools om in rode `NativeCommandError`-meldingen, ook als alles goed gaat. Kijk naar `$LASTEXITCODE`.
- **Het schermtoetsenbord** legt een onzichtbaar venster over DISPLAY3.
