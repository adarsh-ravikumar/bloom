## Setup

[x] Readme
[x] License
[x] .gitignore
[x] First commit
[x] Setup github
[x] Setup project structure
[x] Setup svelte app
[x] Setup tauri app
[x] Setup bloom host
[x] Write a simple dev script
[x] Setup client (py)

## Protocol

- Goal: Print to console
[x] Connection (Handshake, identity and version validation)
[x] Rejection
[x] Disconnection
[x] Command
[x] Event
[x] Synchronization
[ ] Failure semantics

## Bloom core

[ ] Event Stream
[ ] Command Registry
[ ] Panel Registry
[ ] Workspace
[ ] Transporter
[ ] Base Panel
[ ] Base Widget

## Svelte + Tauri stub

[x] Integrate Tauri + Bloom host
[ ] Port the text field example
[ ] Wire and test e2e commands and events
[ ] Wire and test workspace sync

## Bloom UI Layer

[ ] Theme Manager
[ ] Basic widgets (Text, Button, Input fields)
[ ] Panel Pages (Sidebar tabs, view changes accordingly)
[ ] Preferences panel
[ ] Dynamic field / page registration for panels

## Session and project

[ ] Host loads project config
[ ] Interface to write session settings to `session.bloom.yml` (current session)
[ ] Session config loading and saving

## Panels

[ ] Window decoration
[ ] Attach / Detach panel from main workspace
[ ] Docking (to main workspace and in panels)
[ ] Dock panels in tabs
[ ] Save and load workspace config (written to session.bloom.yml)
[ ] Handle multiple monitors

## Interaction

[ ] Context menu
[ ] Keyboard shortcuts API
[ ] Custom bindings in preferences panel
[ ] Command Palette API
[ ] Command Palette

## CLI

[ ] bloom init
[ ] bloom dev
[ ] bloom run
