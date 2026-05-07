# HerbReady

[![Rust](https://img.shields.io/badge/rust-2021-orange?logo=rust)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/vue-3.5.13-42b883?logo=vuedotjs)](https://vuejs.org)
[![Tauri](https://img.shields.io/badge/tauri-2-ffc107?logo=tauri)](https://tauri.app)

HerbReady is a Thai herbal medicine dispensing system for hospitals and clinics, built with Tauri, Vue 3, and TypeScript.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust (Tauri 2) |
| Database | MySQL 8.0 (via sqlx) |
| UI Components | Lucide Vue Icons |

## Features

- **Daily Queue Processing** — Load and process daily patient queue with herbal medicine eligibility
- **Patient Search** — Search patients by name, HN, or CID to view prescription history
- **Drug Eligibility System** — Categorize patients into eligible, never-dispensed, and not-yet-eligible groups
- **Drug Interaction Alerts** — Detect potential interactions between modern drugs and herbal medicines
- **Lab Result Integration** — Fetch and display lab results with configurable threshold rules
- **Dispense Workflow** — Select drugs to dispense, set quantities, and generate prescriptions
- **Prescription PDF** — Generate printable PDF prescriptions for patients
- **Excel Export** — Export daily dispensing records to Excel (xlsx format)
- **Print Management** — Batch print multiple prescriptions
- **Multi-language** — Support for English and Thai

## Prerequisites

- Node.js 18+
- Rust 1.70+
- MySQL 8.0+
- npm
- Windows/macOS/Linux

## Installation

```bash
# Clone repository
git clone https://github.com/suradet-ps/herbready.git
cd herbready

# Install dependencies
npm install

# Run development server
npm run tauri dev

# Build for production
npm run tauri build
```

## Configuration

Configure the application via the Settings dialog:

1. **Database Connection** — Host, port, database name, username, password
2. **Drug Settings** — Configure herbal medicines with dosage cycles (days) and capsule counts
3. **Department Settings** — Map department codes for patient filtering
4. **Lab Rules** — Set threshold values for lab result monitoring
5. **Herb-Drug Interactions** — Define interaction rules between modern and herbal drugs

## Project Structure

```
├── src/                          # Vue 3 frontend
│   ├── components/               # Vue components
│   │   ├── dialogs/              # Modal dialogs
│   │   ├── DailyTab.vue          # Daily queue tab
│   │   ├── SearchTab.vue         # Patient search tab
│   │   ├── HistoryTab.vue        # Dispensing history tab
│   │   └── DrugPanel.vue         # Drug selection panel
│   ├── stores/                   # Pinia state management
│   ├── api/                      # Tauri API wrappers
│   ├── types/                    # TypeScript interfaces
│   └── utils/                    # Utility functions
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands.rs           # Tauri command handlers
│   │   ├── db.rs                 # Database connection & queries
│   │   ├── models.rs             # Data structures
│   │   ├── queries.rs            # SQL query definitions
│   │   ├── config.rs             # Configuration management
│   │   └── lib.rs                # Library exports
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
└── scripts/                      # Build & icon generation scripts
```

## Database Schema

Key tables (requires connection to hospital HIS database):

- `opday` — Outpatient daily records
- `patient` — Patient demographics
- `opdscreen` — Vital signs and screening
- `drugitems` — Drug master data
- `lab_order` — Laboratory orders and results

## Development Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Run Vite dev server |
| `npm run build` | Build Vue frontend |
| `npm run tauri dev` | Run Tauri in development mode |
| `npm run tauri build` | Build production executable |
| `npm run gen-icons` | Generate app icons from source |

## License

MIT License