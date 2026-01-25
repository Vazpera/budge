# budge
A terminal user interface for managing budgets. Still a huge work in progress

# Installation
Installation is simple, all sqlx snobbiness is handled through build.rs
```bash
git clone https://github.com/vazpera/budge
cd budge
cargo build
mv target/release/budge ~/.local/bin/
```
# Usage
>[!INFO] `budge` run by itself will display help!
## Creating a budget
Use the `budge create` subcommand to generate a budget with its amount and the month it is tied to. Months should be in `YYYY-MM` format
```bash
budge create <amount> <month>
```
## View a budget 
Use the `budge load` subcommand to open the UI. Make sure to supply the ID
```bash
budge load <id>
```
## Delete a budge
```bash
budge remove <id>
```
## Listing all budgets
```bash
budge list
```

# Keybinds while in the UI
| Key | Action                       |
|-----|------------------------------|
| a   | Add a new payment            |
| Del | Delete a payment by ID       |
| Tab | Change focus while editing   |
| j/k | Scroll through payments      |
| Esc | Exits editing without saving |
| Ret.| Finalizes edits and submits  |
