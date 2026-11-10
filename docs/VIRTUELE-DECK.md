# Ontwerp: virtuele Stream Deck

## Doel

Een venster zonder rand dat het volledige rechter scherm (portret, 1080×1920) vult en eruitziet en werkt
als een Stream Deck: een grid van knoppen met iconen/tekst. Klikken (of aanraken) = knop indrukken.
Alle bestaande OpenDeck-functies (profielen, multi-actions, Elgato-plugins, OBS-plugin …) moeten gewoon werken.

## Hoe OpenDeck werkt (relevant deel)

- Een "device" is een `DeviceInfo` (`src-tauri/src/shared.rs`): `id`, `name`, `rows`, `columns`, `encoders`, `type` …
- Hardware registreert zich via `crate::events::inbound::devices::register_device("", …)` (zie `elgato.rs::init`).
  Met lege `uuid` is geen plugin-namespace nodig. Device-plugins kunnen dit ook via de WebSocket (`registerDevice`).
- Knopdrukken gaan via `inbound::devices::key_down` / `key_up` met `PressPayload { device, position }`.
  Er bestaat al `trigger_virtual_press` (frontend-command) die precies dit doet voor één klik.
- Plugins sturen beelden met `setImage` → `outbound::devices::update_image(context, image)` → voor Elgato
  naar `elgato::update_image`. **Hier moet de virtuele deck aanhaken**: beelden naar ons venster sturen i.p.v. naar USB.
- Het hoofdvenster heet `"main"` (`tauri.conf.json`); de editor (`DeviceView.svelte`, `Key.svelte`) tekent het grid al.

## Hoe de deck zijn knoppen tekent (update)

De deck-pagina tekent zijn knoppen **zelf** uit het geselecteerde profiel van `vd-virtual-1`, met dezelfde renderer
als het hoofdvenster (`renderImage` uit `src/lib/rendererHelper.ts`, in `src/components/VirtualDeckKey.svelte`).
Daardoor werkt de deck ook als het OpenDeck-venster dicht is of in de tray zit. Hij luistert naar `update_state`,
`key_moved`, `show_ok`/`show_alert` en `switch_profile`, en de backend meldt profielwissels met `virtual_deck_profile`
(één regel in `set_selected_profile`). Wijzigingen in de editor (acties toevoegen/verplaatsen) worden elke 1,5 s opgehaald.
Onderstaande beschrijving van "beelden doorsturen via `update_image`" is daarmee vervangen.

Linux-noot (alleen voor testen bij Jeremy): daar is de achtergrond altijd zwart, omdat WebKitGTK een doorzichtig
venster niet ververst (de deck bleef leeg). Op Windows (WebView2) werkt doorzichtig gewoon.

## Gekozen aanpak: ingebouwd in de fork

Twee opties overwogen:

|                           | A. Ingebouwd in fork (gekozen)           | B. Losse device-plugin                 |
| ------------------------- | ---------------------------------------- | -------------------------------------- |
| Venster op rechter scherm | Tweede Tauri-venster, volledige controle | Plugin moet zelf een venster/app maken |
| Beelden                   | Direct uit `update_image`                | Via WebSocket-events                   |
| Upstream mergen           | Iets meer werk                           | Geen fork nodig                        |

A is gekozen omdat het venster dan dezelfde rendering (`Key.svelte`) kan hergebruiken en alles in één app blijft.

### Onderdelen

1. **`src-tauri/src/virtual_deck.rs`** (nieuw)
   - Registreert bij opstarten een device, bijv. id `vd-virtual-1`, naam "Virtual Stream Deck",
     `rows`/`columns` uit instellingen (standaard bijv. 8 rijen × 5 kolommen voor portret), `encoders: 0`.
   - Houdt per positie het laatste beeld bij, zodat het venster na herladen alles meteen kan tekenen.
   - Tauri-commands: `virtual_deck_press(position, pressed)` → `key_down`/`key_up` (los indrukken en loslaten,
     zodat "lang indrukken"-acties werken), `virtual_deck_get_state()`.
2. **Haakje in `outbound::devices::update_image` en `clear_screen`**: als `device` met `vd-` begint,
   event `virtual_deck_image` naar het venster `"virtual-deck"` sturen in plaats van naar `elgato`. (Kleinste mogelijke wijziging in upstream-code.)
3. **Tweede venster `"virtual-deck"`**: aangemaakt in Rust (`WebviewWindowBuilder`), zonder rand, altijd-op-voorgrond optioneel,
   niet in de taakbalk, gepositioneerd en gemaximaliseerd op het doelscherm (zie `SYSTEEM.md` voor schermkeuze).
4. **Frontend-route `src/routes/virtual-deck/+page.svelte`** (nieuw): toont alleen het grid op zwarte achtergrond,
   `pointerdown`/`pointerup` → `virtual_deck_press`. Geen editor-UI; bewerken blijft in het hoofdvenster.
5. **Instellingen** (in `SettingsView.svelte`): aan/uit, rijen, kolommen, knopafstand, welk scherm, altijd-op-voorgrond.
   Voorlopig via het bestand hieronder.

## Instellingen

In OpenDeck: **Settings → Virtual Stream Deck**, daarna **Apply**. Geen herstart nodig.
Ze staan in `%APPDATA%\opendeck\virtual_deck.json`.

| Instelling                 | Betekenis                                                                                                                                                                                                                                                              |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Window                     | **Floating**: los venster dat je overal neerzet, verplaatst (sleep aan de ruimte tussen de knoppen) en vergroot (sleep aan een rand of hoek), zoals Elgato's Virtual Stream Deck. Positie en grootte worden onthouden. **Fill a whole monitor**: vult een heel scherm. |
| Monitor                    | Alleen bij "Fill a whole monitor". Automatisch = meest rechtse staande scherm.                                                                                                                                                                                         |
| Keys                       | Kolommen × rijen. Let op: acties staan op volgorde, dus anders indelen verschuift ze; acties die niet meer passen verdwijnen.                                                                                                                                          |
| Gap                        | Ruimte tussen de knoppen.                                                                                                                                                                                                                                              |
| Background                 | 0 = doorzichtig (alleen de knoppen zweven op je bureaublad), 100 = zwart.                                                                                                                                                                                              |
| Lock position and size     | Alleen bij Floating. Aan: de deck kan niet verplaatst of vergroot worden, zodat een klik naast een knop hem niet per ongeluk versleept.                                                                                                                                |
| Start OpenDeck in the tray | Standaard aan: bij opstarten verschijnt alleen de deck; het OpenDeck-venster open je via het tray-icoon.                                                                                                                                                               |
| Always on top              | Boven andere vensters blijven.                                                                                                                                                                                                                                         |

- Verbergen en terugzetten: tray-icoon → **Show/hide virtual deck**. Er is geen sleepbalk meer (op verzoek van Alwyn weggehaald).
- Vergroten kan aan alle randen en hoeken; de buitenrand is dan minstens 8 px breed, zodat die randen nooit over een knop vallen.
- Het venster neemt nooit focus, zodat sneltoetsen naar de app gaan die je gebruikt en niet naar de deck.
- Slaapstand-instellingen van OpenDeck gelden ook voor de deck: in slaap = donker, eerste klik maakt hem wakker.
- Na een schermwissel of slaapstand zet de deck zichzelf binnen een paar seconden terug op zijn plek.

## Roadmap

- [x] Bouwomgeving werkend op Windows (`deno task tauri dev` start)
- [x] Virtueel device registreren (`vd-virtual-1`, profiel wordt aangemaakt)
- [x] Venster `virtual-deck` op het portretscherm, grid tekenen (getest op DISPLAY3, 5×8)
- [ ] Knoppen toewijzen in hoofdvenster en controleren dat ze op de deck verschijnen
- [ ] Klikken triggert acties (testen met starterpack-acties, bijv. "Run Command")
- [ ] Beelden/titels van plugins live bijwerken (`setImage`, `setTitle`, states)
- [x] Instellingen voor grid-grootte en scherm (Settings → Virtual Stream Deck)
- [x] Zwevend, verplaatsbaar venster met doorzichtige achtergrond
- [x] Venster herstellen na schermwissel/slaapstand
- [ ] Automatisch starten met Windows (bestaande OpenDeck-instelling testen)
- [ ] Installer bouwen en installeren op Alwyns pc
- [ ] Later: eigen plugins (OBS, apps starten, mediabediening …) in `plugins/`

## Openstaande vragen voor Alwyn

- Hoeveel knoppen wil je (rijen × kolommen)?
- Moet het venster altijd boven andere vensters blijven?
- Welke acties/plugins zijn het belangrijkst (OBS, Discord, Spotify, sneltoetsen …)?
