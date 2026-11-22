# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri v2, React, TypeScript, Vite, and Tailwind CSS. The overlay UI is a web view. The desktop shell is Tauri, chosen by the specification. Progress is a local JSON file. No cloud account and no hosted database.

## Users

A developer on their own machine, in the middle of shipping a piece of software. The job is to see the current delivery stage and the next concrete action without leaving the work.

## Product Purpose

A lightweight always-on-top overlay that supervises a normal software delivery pipeline. It shows the current stage, the next action, and a short checklist, and it can confirm a few local facts (git, project files). It does not deploy anything. Success means the person can move through the stages, keep that progress across restarts, and another developer can clone the repo and run it.

## Positioning

The check happens on the developer's machine against their own folder and their own confirmation. It guides the next step. It does not ship the product for them.

## Operating Context

Local desktop, one person, one project folder at a time. Sits above other windows while they use an editor, a terminal, GitHub, and host dashboards. Progress and notes stay in the app data directory. The stage list is editable JSON.

## Capabilities and Constraints

Confirmed: frameless always-on-top overlay, drag, click-through, tray show/hide/quit, global hotkey, compact and expanded layout, the eleven default stages, manual and file/git checks, a saved note tied to the active window title, local persistence, dark mode by default and system-aware, MIT license, README for Windows, macOS, and Linux.

Constraints: no cloud deploy, no continuous vision analysis, no multi-user sync, no mobile app. Screen checks store a note and the window title. They do not upload pixels. `.env` files are detected by name only; their contents are never read.

Active window title on Linux uses `xdotool` when it is installed. macOS uses System Events. Windows title lookup is not wired yet.

## Brand Commitments

The name is Pipeline Supervisor. Voice is plain and calm. The overlay stays small and does not steal focus. Dark mode is the default; the appearance follows the system theme.

## Evidence on Hand

The repository starts from the specification for this overlay. There are no testimonials, customers, or metrics. Do not invent any.

## Product Principles

- Answer "where am I, and what is next" before anything else.
- Keep every check on the machine that is doing the work.
- Prefer a confirmation the person understands over a silent automation.
- Stay out of the way until asked.

## Accessibility & Inclusion

The overlay must be readable at a small size, usable from the keyboard when it is focused, and visible in both system themes. No separate accessibility standard was named beyond that.
