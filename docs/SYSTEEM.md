# Alwyns systeem

Opgemeten op 9 november 2026. Dit is de enige machine waarop de virtuele deck echt gebruikt wordt.

## Hardware

| Onderdeel           | Waarde                                                                        |
| ------------------- | ----------------------------------------------------------------------------- |
| OS                  | Windows 11 Pro (build 26300)                                                  |
| CPU                 | AMD Ryzen 9 9950X3D (16 cores)                                                |
| RAM                 | 32 GB                                                                         |
| GPU                 | NVIDIA GeForce RTX 4080 SUPER (+ AMD Radeon iGPU, geen scherm op aangesloten) |
| Fysieke Stream Deck | **Geen** — er is geen Elgato-apparaat via USB aangesloten                     |

## Schermopstelling (belangrijk!)

Coördinaten in fysieke pixels (DPI-aware gemeten), ten opzichte van het hoofdscherm.

```
                 ┌──────────────────────┐
                 │ DISPLAY2  PL2466H    │
                 │ 1920×1080            │
                 │ x=795  y=-1080       │
   ┌─────────────┴──────────────────────┴───────────┐ ┌────────┐
   │ DISPLAY1  Alienware AW3423DWF (hoofdscherm)    │ │DISPLAY3│
   │ 3440×1440  ultrawide                           │ │PL2466H │
   │ x=0  y=0                                       │ │1080×   │
   │                                                │ │1920    │
   └────────────────────────────────────────────────┘ │PORTRET │
                                                      │x=3440  │
                                                      │y=-251  │
                                                      └────────┘
```

| Scherm       | Model               | Resolutie               | Positie (x, y) | Rol                          |
| ------------ | ------------------- | ----------------------- | -------------- | ---------------------------- |
| DISPLAY1     | Alienware AW3423DWF | 3440×1440               | 0, 0           | Hoofdscherm                  |
| DISPLAY2     | iiyama PL2466H      | 1920×1080               | 795, -1080     | Boven het hoofdscherm        |
| **DISPLAY3** | **iiyama PL2466H**  | **1080×1920 (portret)** | **3440, -251** | **Doelscherm virtuele deck** |

Aandachtspunten:

- Het rechter scherm staat **staand (portret)**. De knoppen-grid moet dus meer rijen dan kolommen hebben.
- Windows-schaling verschilt per scherm (hoofdscherm ≈150%, de andere lager). Gebruik in Tauri
  fysieke posities/afmetingen (`PhysicalPosition`/`PhysicalSize`) of zoek de monitor op via
  `available_monitors()`; vertrouw niet op logische pixels.
- Kies het scherm **niet** op vaste coördinaten of op index (de volgorde kan veranderen), maar bijv.
  "het scherm met hoogte > breedte" of een instelbare monitornaam, met fallback naar het meest rechtse scherm.
- `DISPLAY3` begint op y = -251, dus negatieve coördinaten moeten werken.

### Testen zonder deze opstelling (voor Jeremy)

- Draai de virtuele deck in een gewoon venster van 1080×1920 (of geschaald 540×960) in plaats van fullscreen.
- Of zet een eigen scherm tijdelijk op portret via Windows-instellingen → Beeldscherm → Stand.

## Software

| Software                                | Status                                                                                                                                                                |
| --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Elgato Stream Deck                      | 7.5.1 geïnstalleerd en draait (`StreamDeck.exe`), maar werkt niet naar wens → wordt vervangen door deze fork. Sluit hem af tijdens testen om conflicten te voorkomen. |
| OBS Studio                              | 32.1.2 — belangrijke doelapp voor knoppen                                                                                                                             |
| Microsoft Edge WebView2 Runtime         | 157 — aanwezig (nodig voor Tauri)                                                                                                                                     |
| Visual Studio Code                      | aanwezig                                                                                                                                                              |
| Python                                  | 3.14                                                                                                                                                                  |
| Git / GitHub CLI                        | 2.55 / `gh` 2.103, ingelogd als `AlwynDG`                                                                                                                             |
| OpenDeck                                | nog **niet** geïnstalleerd                                                                                                                                            |
| Rust, Deno, Node.js, VS C++ Build Tools | nog **niet** geïnstalleerd — nodig om te bouwen                                                                                                                       |

## Bouwomgeving opzetten op Windows

```bash
winget install --id Rustlang.Rustup
winget install --id DenoLand.Deno
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Daarna nieuwe terminal openen, `rustup default stable-msvc`, en in de repo `deno install`.
Node.js is alleen nodig voor Stream Deck-plugins die in Node geschreven zijn (veel Elgato-plugins).

## Paden op Windows

- Config/profielen: `%APPDATA%\opendeck\`
- Logs: `%APPDATA%\opendeck\logs\`
- Plugins: `%APPDATA%\opendeck\plugins\`
- Lokale repo bij Alwyn: `D:\AI\OpenDeck-Virtual`
