# Samenwerken (Alwyn & Jeremy)

## Eenmalig

```bash
git clone https://github.com/AlwynDG/OpenDeck-Virtual.git
cd OpenDeck-Virtual
git remote add upstream https://github.com/nekename/OpenDeck.git
```

Installeer de bouwomgeving (zie [SYSTEEM.md](SYSTEEM.md#bouwomgeving-opzetten-op-windows)) en draai `deno install`.

Met Claude Code: open de repo-map en Claude leest automatisch `CLAUDE.md`, die naar alle docs verwijst.

## Werkwijze

1. Pak een punt uit de roadmap in [VIRTUELE-DECK.md](VIRTUELE-DECK.md) of een GitHub-issue. Zet in het issue dat je ermee bezig bent.
2. Maak een branch: `git switch -c feat/virtual-deck-window`.
3. Commit volgens Conventional Commits (`feat: …`, `fix: …`, `docs: …`).
4. Push en open een PR naar `main`. De ander reviewt; Alwyn test op zijn echte schermopstelling.
5. Vink het roadmap-punt af in dezelfde PR.

## Upstream-updates binnenhalen

```bash
git fetch upstream
git switch -c chore/merge-upstream
git merge upstream/main
```

Conflicten oplossen, testen, PR openen.

## Testen

- Jeremy heeft geen portretscherm op Alwyns positie: test in vensterstand (zie [SYSTEEM.md](SYSTEEM.md#testen-zonder-deze-opstelling-voor-jeremy)).
- Een fysieke Stream Deck is niet nodig; dat is juist het punt.
- Sluit de Elgato Stream Deck-software af tijdens het testen.
