#!/usr/bin/env python3
"""Prüft, dass die Dokumentation des Projekts englisch ist.

WHAT: liest die Markdown-Dateien des Repositorys und meldet jede Datei, die
      deutschen Text trägt.
WHY: die Spezifikation, der Code und die Dokumentation sprechen eine Sprache.
      Das Benutzerhandbuch und das Glossar sind die zwei Ausnahmen, weil sie
      von Natur in beiden Sprachen gelesen werden, und das Archiv der
      Übergaben von 2026 ist eine dritte, weil es eine Aufzeichnung der
      Vergangenheit ist und keine lebende Dokumentation.
      Eine zweite Sprache in der Dokumentation verdoppelt die Arbeit und die
      Drift — dieses Skript verhindert beides.

BEISPIEL:
      python3 scripts/check-docs-language.py
      python3 scripts/check-docs-language.py --verbose

Rückgabewert 0 means "the documentation of the project is English", 1 means
"at least one document carries German text" — every one of them is named.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

# Die Ausnahmen. Sie stehen hier, weil sie eine Entscheidung sind und keine
# Vergesslichkeit.
EXCEPTED = (
    # Das Benutzerhandbuch ist ein Produktmerkmal: es gibt es in Deutsch und
    # in Englisch.
    "docs/user-guide/",
    # Das Glossar ist ein Woerterbuch mit zwei Spalten.
    "docs/glossary.md",
    # Das Archiv der Uebergaben von 2026 ist eine Aufzeichnung der
    # Vergangenheit. Ein Bericht ueber eine erledigte Aufgabe wird nicht
    # nachtraeglich umgeschrieben.
    "docs/archive/",
)

# Die Umlaute und das Eszett. Sie kommen im Englischen nicht vor und sind
# damit das verlaesslichste Zeichen fuer deutschen Text.
UMLAUTE = re.compile(r"[äöüÄÖÜß]")

# Woerter, die es im Englischen nicht gibt. Sie stehen am Satzanfang, und
# die Liste ist kurz und bewusst konservativ: ein Fehlalarm ist schlimmer
# als ein Wort, das durchrutscht.
DEUTSCHE_WOERTER = re.compile(
    r"(?:^|[.:!?]\s+|\|\s*|-\s*|#{1,6}\s*|\*\*)("
    r"Der|Die|Das|Den|Dem|Des|Ein|Eine|Einen|Einem|Einer|Eines|Und|Aber|"
    r"Nicht|Kein|Keine|Wird|Wurde|Wurden|Sind|Meilenstein|Meilensteine|"
    r"Fehler|Fehlers|Datei|Dateien|Verzeichnis|Verzeichnisse|Zeile|Zeilen|"
    r"Abschnitt|Abschnitte|Kapitel|Beispiel|Beispiele|Anforderung|"
    r"Anforderungen|Sprache|Sprachen|Eigenschaft|Regel|Regeln|Grund|"
    r"Stelle|Aufgabe|Aufgaben|Werkzeug|Pruefung|Beleg|Nachweis|Entwurf|"
    r"Schritt|Schritte|Aenderung|Aenderungen|Groesse|Grenze|Messung|"
    r"Ergebnis|Ergebnisse|Stand|Offen|Geschlossen|Unterstuetzt|Benoetigt|"
    r"Einstellungen|Konfiguration|Verzeichnisbaum"
    r")(?=[\s.,;:!?)\]|]|$)"
)


def is_excepted(relative: Path) -> bool:
    """Whether the file is one of the exceptions.

    WHAT: answers whether the path lies under one of the excepted places.
    """
    text = relative.as_posix()
    return any(text.startswith(prefix) for prefix in EXCEPTED)


def german_reasons(text: str) -> list[str]:
    """Return the reasons why the text looks German.

    WHAT: looks for the umlauts and the sharp s, and for the German words
          that the English language does not have.
    WHY: the two signals together catch a German text even when it carries no
          umlaut — a list of identifiers alone would not.
    """
    reasons = []
    if UMLAUTE.search(text):
        reasons.append("it carries an umlaut or a sharp s")
    if DEUTSCHE_WOERTER.search(text):
        reasons.append("it carries a German word")
    return reasons


def main() -> int:
    """Check every Markdown file of the repository."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="name every file that was read",
    )
    args = parser.parse_args()

    failures: list[str] = []
    gelesen = 0

    for datei in sorted(REPO_ROOT.rglob("*.md")):
        relative = datei.relative_to(REPO_ROOT)
        if any(part in (".git", "target", "node_modules") for part in relative.parts):
            continue
        if is_excepted(relative):
            if args.verbose:
                print(f"  ausgenommen  {relative}")
            continue
        gelesen += 1
        text = datei.read_text(encoding="utf-8", errors="replace")
        # Der Codeblock einer Datei traegt Kommandos, keine Prosa.
        text = re.sub(r"```.*?```", "", text, flags=re.DOTALL)
        text = re.sub(r"`[^`]*`", "", text)
        gruende = german_reasons(text)
        if gruende:
            failures.append(f"{relative}: {', '.join(gruende)}")
        elif args.verbose:
            print(f"  gelesen      {relative}")

    print(f"Files read               : {gelesen}")
    print(f"Files with German text   : {len(failures)}")
    for zeile in failures:
        print(f"  {zeile}")

    if failures:
        print(
            "RESULT: FAIL — the documentation of the project is English. The "
            "exceptions are the user guide, the glossary and the archive of "
            "the handovers of 2026.",
            file=sys.stderr,
        )
        return 1

    print("RESULT: PASS — the documentation of the project is English.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
