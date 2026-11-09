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

## Configuratie (`%APPDATA%\opendeck\virtual_deck.json`)

Wordt bij de eerste start aangemaakt. Na wijzigen OpenDeck herstarten.

```json
{
	"enabled": true,
	"rows": 8,
	"columns": 5,
	"gap": 16,
	"monitor": null,
	"always_on_top": true,
	"windowed": false
}
```

- `monitor`: `null` = automatisch (meest rechtse staande scherm), of een naam zoals `"\\\\.\\DISPLAY3"` (staat in de log bij opstarten).
- `windowed: true` = testmodus in een gewoon venster van 540×960 (handig voor Jeremy).
- Het venster kan niet focussen (in fullscreen-modus), zodat sneltoetsen naar de app gaan die je gebruikt en niet naar de deck.
- Let op: andere `rows`/`columns` = ander aantal knopposities in je profiel.

## Roadmap

- [x] Bouwomgeving werkend op Windows (`deno task tauri dev` start)
- [x] Virtueel device registreren (`vd-virtual-1`, profiel wordt aangemaakt)
- [x] Venster `virtual-deck` op het portretscherm, grid tekenen (getest op DISPLAY3, 5×8)
- [ ] Knoppen toewijzen in hoofdvenster en controleren dat ze op de deck verschijnen
- [ ] Klikken triggert acties (testen met starterpack-acties, bijv. "Run Command")
- [ ] Beelden/titels van plugins live bijwerken (`setImage`, `setTitle`, states)
- [ ] Instellingen voor grid-grootte en scherm
- [ ] Automatisch starten met Windows, venster herstellen na schermwissel/slaapstand
- [ ] Installer bouwen en installeren op Alwyns pc
- [ ] Later: eigen plugins (OBS, apps starten, mediabediening …) in `plugins/`

## Openstaande vragen voor Alwyn

- Hoeveel knoppen wil je (rijen × kolommen)?
- Moet het venster altijd boven andere vensters blijven?
- Welke acties/plugins zijn het belangrijkst (OBS, Discord, Spotify, sneltoetsen …)?
