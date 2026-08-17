# Project Structure

bloom-app
|  bloom
|- app
|  |- src
|  |  |- lib
|  |  |  |- bloom.ts
|  |  |  |- widgets
|  |  |  |  |- Base
|  |  |  |  |  |  Widget.svelte
|  |  |  |  |  |  style.scss
|  |  |  |  |  |  contract.d.ts
|  |  |  |  |
|  |  |  |  |- Button
|  |  |  |  |  |  Button.svelte
|  |  |  |  |  |  style.scss
|  |  |  |  |  |  contract.d.ts
|  |  |  |
|  |  |  |- panels
|  |  |  |  |- Base
|  |  |  |  |  |  Panel.svelte
|  |  |  |  |  |  style.scss
|  |  |  |  |  |  contract.d.ts
|  |  |  |  |
|  |  |  |  |- Inspector
|  |  |  |  |  |  Inspector.svelte
|  |  |  |  |  |  style.scss
|  |  |  |  |  |  contract.d.ts
|  |  |  |
|  |  |  |- themes
|  |  |  |  |  base.scss
|  |  |  |  |  classic-light.scss
|  |  |  |  |  classic-dark.scss
|  |  |  |  |  classic-gruvbox-light.scss
|  |  |  |  |  classic-gruvbox-dark.scss
|  |  |  |
|  |  |  |  global.svelte.ts
|  |  |
|  |  |  bloom.svelte
|  |  |  main.ts
|  |
|  | index.html
|
|- client
|  |- src
|  |  | ...
|  | run.sh
|
|  proj.bloom.yml
|  session.bloom.yml

## Notes

### bloom

- This executable sits at the root of the project.
- It a single executable one can download, and place it in the directory of their choosing to seed the project.
- Idea is, bloom isn't a clunky framework that you install on to your machine. It is designed this way purely so one can simple "use-and-throw" the library
- It is recommended that this executable is not commited to your vcs, purely because distribution becomes tied to the platform the project was built on.
- In order to run a bloom project, download this file, and throw it in the root directory. More about the CLI usecase in later section

### app

- This folder contains the svelte app
- The structure is pretty much the same of any svelte application

#### src/lib/bloom.ts

- Contains the entire library, in a single file.
- Bloom's core will never be shipped as an npm package or similar dependancy, because of the "use-and-throw" philosophy.
- It will also never be minified, to keep the file readable.
- It will also not be split into multiple files, and put into a src/lib/core (or similar) folder.
- This is a delibrate decission, take to keep the actual fingerprint of bloom-core tiny.

#### src/lib/widgets and src/lib/panels

- These directories house all the widgets and panels.
- The specific structure ensures that if one intends to distribute a widget or a panel, only those directories can be zipped up (or prepped otherwise) and shared.
- A distribution of a widget or panel is self-contained, with no external dependencies.
- This is done to keep each widget and panel isolated, and to keep their size fairly small.
- `[Name].svelte` describes the actual widget / panel, it's functionality, etc. This is the implementation.
- `style.scss` describes the style. One can also write the style within the `.svelte` file itself. I personally find this to be a bit cleaner
- `contract.d.ts` is a declaration file. It declares all the properties, events and commands a widget or panel exposes.

#### src/lib/global.svelte.ts

- All global declarations (Bloom specfic: Global Event Stream, Global Command Registry, etc., or other user defined constructs) are defined here.
- It is highly recommended that one uses the modern svelte `runes` paradigm, rather than the older `Writable` streams.
- Hence, the file delibrately has a `.svelte.ts` extension. To allow for exactly this.
- All of bloom's core global constructs have their own implementation to handle subscription. This is done to keep the core framework-agnostic.
- Any user introduced construct, however, is highly encouraged to use svelte's `runes`

#### src/lib/themes

- `themes/base.scss` exports mixins, with defaults
- A theme can be created by including mixins from base, and composing them as needed.
- All theme files must be included in the `bloom.svelte` file, and registered in the Theme manager

### client

- The client directory contains all the code that drives the bloom app
- Even though at first glance, the data layer seems like the host in the traditional sense (as it serves the data to the UI), it is the UI layer that accepts connections from the data layer.
- The `client/run.sh` is what Bloom executes when the app starts / whenever the watchdog triggers.

## Bloom CLI

### ./bloom init

- When the executable is placed in an empty directory (or otherwise), this launches the initialization prompt.
- It constructs the entire project structure.

### ./bloom dev

- This command launches:
  - `vite` dev server on the svelte app
  - Bloom host
  - Tauri app
  - A watchdog on the client directory that runs `client/run.sh` to start the process

- It ensures that if one of the three core processes die (vite, bloom host, tauri), the rest are terminated properly.

### ./bloom run

- This command builds the svelte app using `vite` and launches
  - `vite` preview server
  - Bloom host
  - Tauri app
  - `client/run.sh`

- It ensures that if any one of these processes die (including `client/run.sh`), the rest are terminated properly.
