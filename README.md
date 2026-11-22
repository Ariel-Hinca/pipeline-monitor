# Pipeline Supervisor

A small always-on-top overlay for your own machine. It shows the current stage of a software delivery pipeline, the next action, and a checklist you can check off. It does not deploy anything.

## What you can do

- Drag the window. It stays above other windows and does not take focus when it opens.
- Mark a stage done, or check items one by one.
- Point it at a project folder and run a check. Git, a remote, a secrets file, tests, and common backend or deploy files are detected by name. You confirm the rest.
- Save a note about what is on screen. The note and the front window title stay on disk. Pixels are not captured or uploaded.
- Hide it from the window or the tray. `Alt+Shift+P` hides it, shows it, or turns click-through off if that mode is stuck on.
- Edit the stage list. Progress survives a restart.

Closing the window hides it. Quit from the tray menu.

## Privacy

Everything stays on this computer.

- Progress and notes are JSON files in the app data directory. The overlay shows the path.
- A project check looks at file names, `package.json` script names, and `git` metadata. It does not read `.env` contents.
- "Save note" stores your text and, when the desktop allows it, the front window's title. There is no screenshot upload and no cloud account.
- The global hotkey is `Alt+Shift+P`.

Linux window titles need `xdotool`. macOS uses System Events (it may ask for Automation access). Windows does not report a window title yet; the note still saves.

## Permissions the app uses

- Always-on-top, frameless window.
- System tray icon.
- Global hotkey.
- Read and write its own app data directory.
- Read the project folder you type in, limited to the checks above.
- On macOS, Automation access only if you save a note and System Events asks.

It does not request screen recording.

## Run it in development

Install the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS, plus Rust and Node.js 20+.

Linux (Debian or Ubuntu) also needs:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf build-essential libssl-dev pkg-config
```

Optional, for window titles: `sudo apt-get install -y xdotool`.

```bash
npm install
npm run tauri dev
```

A release binary:

```bash
npm run tauri build
```

The executable is under `src-tauri/target/release/`. Installers are under `src-tauri/target/release/bundle/`.

### Windows

Install Visual Studio C++ build tools, the WebView2 runtime, Rust, and Node. Then `npm install` and `npm run tauri dev`. Official details: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

### macOS

Install Xcode command line tools, Rust, and Node. Then `npm install` and `npm run tauri dev`. The first saved note may prompt for Automation access so the app can read the front application name.

## Customize the stages

The repo root `pipeline.json` is the seed. On first launch the app copies it into the app data directory and reads that copy after that.

1. Open the overlay checklist. The file path is at the bottom.
2. Edit that `pipeline.json`.
3. Click **Reload stages**.

To start over from the seed, quit the app, delete the app data `pipeline.json`, and launch again. Your `progress.json` is a different file.

Each stage has `id`, `title`, `next`, and `items`. Each item has `id`, `label`, and `check`:

| `check` | Meaning |
| --- | --- |
| `manual` | You tick it yourself. |
| `git` | The project folder is a git repository. |
| `remote` | `git remote` prints a remote. |
| `env` | `.env` or `.env.local` exists. Contents are not read. |
| `tests` | A test config, a `tests` directory, or a real `package.json` test script. |
| `backend` | Supabase, Prisma, Drizzle, or a compose file. |
| `deploy` | `vercel.json`, `netlify.toml`, `fly.toml`, or `Dockerfile`. |
| `file:relative/path` | That file exists inside the project folder. |
| `dir:relative/path` | That directory exists inside the project folder. |

Paths may not be absolute and may not contain `..`.

Checked items are stored as `"stageId.itemId"` in `progress.json`. Renaming an id leaves the old key unused.

## Assumptions

- One person, one project folder, this computer.
- The verified build here is Linux. The Windows and macOS paths use the same Tauri APIs and have not been executed in this workspace.
- `Alt+Shift+P` can fail if another app already owns it. The tray menu still works.
- Click-through makes the overlay ignore the pointer. Turn it off with the tray item or `Alt+Shift+P`.
- shortcut: the webview CSP is unset so the Vite dev server can load. Tighten it before you distribute a build outside your own machine.

## License

[MIT](LICENSE)
