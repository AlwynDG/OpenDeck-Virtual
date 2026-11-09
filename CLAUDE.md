# CLAUDE.md — OpenDeck-Virtual

Dit is een fork van [nekename/OpenDeck](https://github.com/nekename/OpenDeck) met één doel:
**een virtuele Stream Deck die het hele rechter scherm van Alwyn vult**, zonder fysieke Stream Deck
en zonder de Elgato-software (die bij Alwyn niet goed werkt).

Lees voordat je iets doet:

1. [AGENTS.md](AGENTS.md) — de officiële ontwikkelgids van upstream OpenDeck (architectuur, conventies).
2. [docs/SYSTEEM.md](docs/SYSTEEM.md) — Alwyns pc, schermen en software. De virtuele deck draait alleen daar.
3. [docs/VIRTUELE-DECK.md](docs/VIRTUELE-DECK.md) — ontwerp, keuzes en roadmap van de virtuele deck.
4. [docs/BIJDRAGEN.md](docs/BIJDRAGEN.md) — werkwijze met Git/GitHub tussen Alwyn en Jeremy.

## Wie is wie

- **Alwyn** (`AlwynDG`) — eigenaar en enige eindgebruiker. Draait het op zijn eigen Windows 11-pc.
- **Jeremy** (`Jeremy0538`) — helpt mee met ontwikkelen (ook met Claude Code). Gebruikt de deck zelf niet,
  dus hij test op zijn eigen machine met een gesimuleerde schermopstelling (zie `docs/SYSTEEM.md`).

## Communicatie

- Praat Nederlands met Alwyn en Jeremy. Code, commits, identifiers en code-comments in het Engels
  (zoals upstream), zodat het makkelijk blijft om upstream-wijzigingen te mergen.
- AGENTS.md vraagt om "caveman"-stijl; dat geldt voor upstream-bijdragen. Hier: kort en duidelijk, maar normaal Nederlands.

## Harde regels

- **Repo is publiek** (forks van publieke repos kunnen niet privé). Zet er nooit wachtwoorden, tokens,
  API-keys, serienummers of andere persoonlijke gegevens in.
- **Raak upstream-bestanden zo min mogelijk aan.** Nieuwe functionaliteit liefst in nieuwe bestanden
  (bijv. `src-tauri/src/virtual_deck.rs`, `src/routes/virtual-deck/`). Hoe kleiner de diff met upstream,
  hoe makkelijker `git merge upstream/main` blijft. Wijzig `README.md` en `AGENTS.md` niet; onze eigen docs staan in `docs/` en `LEESMIJ.md`.
- Licentie is **GPL-3.0** — afgeleide code blijft GPL-3.0.
- Commits volgen [Conventional Commits](https://conventionalcommits.org/) (`feat:`, `fix:`, `docs:` …).
- Werk op een feature-branch en open een PR naar `main`; niet direct op `main` pushen
  (uitzondering: kleine doc-fixes door Alwyn zelf).

## Bouwen en controleren (zie AGENTS.md / README.md voor details)

```bash
deno install
deno task tauri dev      # ontwikkelen met live logs
deno task tauri build    # installer bouwen (Windows: .msi/.exe in src-tauri/target/release/bundle/)
```

Voor elke commit: `cargo clippy`, `cargo fmt` (in `src-tauri/`), `deno check`, `deno lint`, `deno task check`, `deno task format`.

## Upstream bijhouden

```bash
git fetch upstream
git merge upstream/main   # op een aparte branch, conflicten oplossen, PR openen
```
