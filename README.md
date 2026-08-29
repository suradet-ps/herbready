# HerbReady

```
██╗  ██╗███████╗██████╗ ██████╗ ██████╗ ███████╗ █████╗ ██████╗ ██╗   ██╗
██║  ██║██╔════╝██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗╚██╗ ██╔╝
███████║█████╗  ██████╔╝██████╔╝██████╔╝█████╗  ███████║██║  ██║ ╚████╔╝
██║  ██║██╔══╝  ██╔══██╗██╔══██╗██╔══██╗██╔══╝  ██╔══██║██║  ██║  ╚═══╝
██║  ██║███████╗██║  ██║██████╔╝██║  ██║███████╗██║  ██║██████╔╝  ██╗
╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝╚═════╝   ╚═╝
```

---

## ◆ PULSE

Herbal medicine does not ask for permission, but it should not be
dispensed blind. HerbReady is the dispensing counter for Thai herbal
medicine in hospitals and clinics: it walks the daily queue, separates
who is eligible from who is not, warns when a herb and a modern drug
should not share a patient, and prints the prescription that explains
itself. The pharmacy sees the interaction, the lab rule, and the
eligibility verdict before the medicine leaves the counter - in Thai or
in English, whichever the room speaks.

| Queue ▣ | Eligibility ▣ | Interactions ▣ | Prescriptions ▣ |
|---|---|---|---|

*The daily loop - screen, judge, warn, dispense, print - is sealed.*

> Built with Tauri 2 + Vue 3 + TypeScript, reading MySQL 8 through
> `sqlx`, printing prescriptions and exporting the day's ledger.
>
> **suradet-ps**, artifact keeper

---

## ◆ IGNITION

One clone, one install, one command.

```
⟫ git clone https://github.com/suradet-ps/herbready.git
⟫ cd herbready
⟫ npm install
⟫ npm run tauri dev
```

The release artifact: `⟫ npm run tauri build`

<details>
<summary>Prerequisites</summary>

- Node.js 18+
- Rust 1.70+
- MySQL 8.0+
- npm
- Windows / macOS / Linux

</details>

After the first launch, configure through the Settings dialog: database
connection (host, port, database, credentials - stored encrypted), herbal
drug settings with dosage cycles and capsule counts, department mappings,
lab thresholds, and herb-drug interaction rules.

---

## ◆ ANATOMY

Two layers, one counter, several honest verdicts.

- **Screens** - the daily queue loads with herbal medicine eligibility
  already asked: who qualifies, who has never been dispensed, who is not
  yet eligible. The tabs - Daily, Search, History - divide the day's
  work without mixing it.
- **Searches** - patients are found by name, HN, or CID, and their
  prescription history opens beside them.
- **Warns** - interaction rules sit between modern drugs and herbal
  medicines, and the alert appears before the dispense, not after the
  harm. Lab results answer to configurable thresholds, fetched from the
  hospital's own tables.
- **Dispenses** - drug selection, quantities, and the prescription in
  one flow; the PDF is generated, the batch prints in one pass, and the
  day's ledger exports to Excel for the record.
- **Speaks** - English and Thai live side by side in the interface; the
  patient and the pharmacist each read the language they work in.

---

## ◆ RITUALS

**The core ceremony** - the daily counter:

1. Open the Daily tab. The queue is loaded and screened: eligible,
   never-dispensed, not-yet-eligible - each group called by its name.
2. Search the patient by name, HN, or CID; the history opens beside
   them.
3. Select the herbs, set the quantities, watch the interaction alerts
   answer. Lab thresholds have already spoken.
4. Generate the prescription PDF, batch the prints, export the day to
   Excel. The counter closes with the record intact.

**The ceremony of the warning** - an interaction is announced before the
prescription exists, not discovered after the patient leaves. A rule
defined in settings is a rule the counter respects.

**The ceremony of the record** - every dispense is printable and every
day is exportable: the PDF belongs to the patient, the spreadsheet
belongs to the pharmacy, and both are produced from the same truth.

---

## ◆ ECHOES

**Where this artifact is heading**

```
screening ▸ daily queue with eligibility groups ───────────────────── ▸ sealed
judgment  ▸ herb-drug alerts, lab threshold rules ──────────────────── ▸ sealed
delivery  ▸ dispense flow, prescription PDF, batch print ───────────── ▸ sealed
record    ▸ Excel export of daily records ──────────────────────────── ▸ sealed
language  ▸ English + Thai interfaces ──────────────────────────────── ▸ sealed
```

**Raising the artifact** - the schema notes live in `SQL-DATABASE.md`;
the design language in `design.md`; Thai user documentation is built
with mdBook under `doc-th/` and deployed from the `docs.yml` workflow.
Open an issue first to discuss a change.

**Status** - documentation ships from `docs.yml`; Windows installers
from `release-windows.yml`. [Watch the workflows](.github/workflows).

---

```
  ─────────────────────────────────────────
   A herb dispensed blind is a guess.
   A herb dispensed seen is medicine.
  ─────────────────────────────────────────
```

HerbReady is released under the [MIT License](LICENSE).