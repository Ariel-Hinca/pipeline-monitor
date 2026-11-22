import { useCallback, useEffect, useState } from "react";
import {
  hideOverlay,
  loadSnapshot,
  markStage,
  onSnapshot,
  reloadPipeline,
  runCheck,
  saveNote,
  setClickThrough,
  setExpanded,
  setItem,
  setProjectPath,
  type Snapshot,
} from "./api";

function Lamp({ done }: { done: boolean }) {
  return (
    <svg
      className={done ? "lamp is-done shrink-0" : "lamp shrink-0"}
      width="28"
      height="28"
      viewBox="0 0 28 28"
      aria-hidden="true"
    >
      <circle className="lamp-dot" cx="14" cy="14" r="5" fill="currentColor" />
      <circle cx="14" cy="14" r="9" fill="none" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

function formatWhen(at: number) {
  return new Date(at * 1000).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export default function App() {
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [error, setError] = useState("");
  const [pending, setPending] = useState(false);
  const [path, setPath] = useState("");
  const [note, setNote] = useState("");

  const apply = useCallback((next: Snapshot) => {
    setSnap(next);
    setPath(next.projectPath);
    setError("");
  }, []);

  const run = useCallback(
    async (action: () => Promise<Snapshot | void>) => {
      setPending(true);
      setError("");
      try {
        const next = await action();
        if (next) apply(next);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setPending(false);
      }
    },
    [apply],
  );

  useEffect(() => {
    let unlisten = () => {};
    void loadSnapshot()
      .then(apply)
      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
    void onSnapshot(apply).then((stop) => {
      unlisten = stop;
    });
    return () => unlisten();
  }, [apply]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA")) return;
      if (!snap || pending) return;
      if (event.key === "d") {
        const stage = snap.stages.find((item) => item.id === snap.currentId);
        if (!stage) return;
        const done = stage.items.every((item) => item.done);
        void run(() => markStage(stage.id, !done));
      } else if (event.key === "c") {
        void run(() => runCheck());
      } else if (event.key === "e") {
        void run(() => setExpanded(!snap.expanded));
      } else if (event.key === "x") {
        void run(() => setClickThrough(!snap.clickThrough));
      } else if (event.key === "Escape" && snap.expanded) {
        void run(() => setExpanded(false));
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [snap, pending, run]);

  if (!snap) {
    return (
      <main className="tag flex h-full flex-col justify-center gap-3 p-4">
        <h1 className="text-xl font-bold">Pipeline Supervisor</h1>
        <p className="quiet text-[15px] leading-snug">
          {error || "Opening your pipeline…"}
        </p>
        {error ? (
          <button className="btn primary w-fit" type="button" onClick={() => void run(loadSnapshot)}>
            Try again
          </button>
        ) : null}
      </main>
    );
  }

  const current = snap.stages.find((stage) => stage.id === snap.currentId) ?? snap.stages[0];
  const stageDone = current.items.every((item) => item.done);

  return (
    <main className="tag flex h-full min-h-0 flex-col gap-3 p-4">
      <header data-tauri-drag-region className="flex items-start gap-3">
        <Lamp key={snap.complete ? "done" : current.id} done={snap.complete} />
        <div data-tauri-drag-region className="min-w-0 flex-1">
          <h1 data-tauri-drag-region className="text-balance text-[22px] font-bold leading-tight">
            {snap.complete ? "Pipeline complete" : current.title}
          </h1>
          <p data-tauri-drag-region className="mt-1 text-pretty text-[15px] leading-snug">
            {snap.complete
              ? "Every check is marked. Uncheck one if the work is not actually done."
              : current.next}
          </p>
        </div>
        <button className="btn shrink-0" type="button" onClick={() => void run(hideOverlay)}>
          Hide
        </button>
      </header>

      <div>
        <div className="mb-1 flex items-baseline justify-between gap-3 text-[15px]">
          <p>
            {snap.doneCount} of {snap.total} checks
          </p>
          <p className="quiet tabular-nums">{snap.percent}%</p>
        </div>
        <div
          className="meter"
          role="progressbar"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={snap.percent}
          aria-label="Pipeline complete"
        >
          <span style={{ width: `${snap.percent}%` }} />
        </div>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          className="btn primary"
          type="button"
          disabled={pending}
          onClick={() => void run(() => markStage(current.id, !stageDone))}
        >
          {stageDone ? "Reopen stage" : "Mark stage done"}
        </button>
        <button className="btn" type="button" disabled={pending} onClick={() => void run(runCheck)}>
          Run check
        </button>
        <button
          className="btn"
          type="button"
          aria-expanded={snap.expanded}
          disabled={pending}
          onClick={() => void run(() => setExpanded(!snap.expanded))}
        >
          {snap.expanded ? "Hide checklist" : "Show checklist"}
        </button>
        <button
          className="btn"
          type="button"
          aria-pressed={snap.clickThrough}
          disabled={pending}
          onClick={() => void run(() => setClickThrough(!snap.clickThrough))}
        >
          {snap.clickThrough ? "Click-through on" : "Click through"}
        </button>
      </div>

      {error ? (
        <p className="bad text-[14px] leading-snug" role="alert">
          {error}
        </p>
      ) : (
        <p className="quiet text-[14px] leading-snug" role="status">
          {snap.message || `${snap.hotkey} hides this. The tray icon brings it back.`}
        </p>
      )}

      {snap.expanded ? (
        <div className="scroll flex min-h-0 flex-1 flex-col gap-4">
          <form
            className="flex flex-col gap-2"
            onSubmit={(event) => {
              event.preventDefault();
              void run(() => setProjectPath(path));
            }}
          >
            <label className="text-[15px] font-bold" htmlFor="project-path">
              Project folder
            </label>
            <input
              id="project-path"
              name="projectPath"
              className="field"
              value={path}
              autoComplete="off"
              spellCheck={false}
              placeholder="Example: /home/you/project…"
              onChange={(event) => setPath(event.target.value)}
            />
            <button className="btn w-fit" type="submit" disabled={pending}>
              Save folder
            </button>
          </form>

          <section className="flex flex-col gap-2">
            <h2 className="text-[15px] font-bold">Stages</h2>
            <ol className="flex flex-col gap-2">
              {snap.stages.map((stage, index) => (
                <li
                  key={stage.id}
                  className={stage.id === current.id && !snap.complete ? "stage-current p-2" : "p-2"}
                >
                  <h3 className="text-[15px] font-bold">
                    {index + 1}. {stage.title}
                  </h3>
                  <ul className="mt-1 flex flex-col">
                    {stage.items.map((item) => (
                      <li key={item.id}>
                        <label className="flex min-h-9 items-center gap-2 text-[15px]">
                          <input
                            className="check"
                            type="checkbox"
                            checked={item.done}
                            disabled={pending}
                            onChange={(event) =>
                              void run(() => setItem(stage.id, item.id, event.target.checked))
                            }
                          />
                          {item.label}
                        </label>
                      </li>
                    ))}
                  </ul>
                </li>
              ))}
            </ol>
          </section>

          <form
            className="flex flex-col gap-2"
            onSubmit={(event) => {
              event.preventDefault();
              void run(async () => {
                const next = await saveNote(note);
                setNote("");
                return next;
              });
            }}
          >
            <label className="text-[15px] font-bold" htmlFor="screen-note">
              What is on screen
            </label>
            <textarea
              id="screen-note"
              name="screenNote"
              className="field min-h-16"
              value={note}
              placeholder="What you see in the front window…"
              onChange={(event) => setNote(event.target.value)}
            />
            <button className="btn w-fit" type="submit" disabled={pending}>
              Save note
            </button>
            <p className="quiet text-[14px] leading-snug">
              The note and the front window title stay on this machine. File contents, including
              secrets, are not read.
            </p>
          </form>

          {snap.notes.length > 0 ? (
            <section>
              <h2 className="text-[15px] font-bold">Recent notes</h2>
              <ul className="mt-2 flex flex-col gap-2">
                {snap.notes.slice(0, 3).map((item) => (
                  <li key={item.at} className="text-[14px] leading-snug">
                    <p>{item.guidance}</p>
                    {item.text ? <p className="mt-1">{item.text}</p> : null}
                    <p className="quiet mt-1">{formatWhen(item.at)}</p>
                  </li>
                ))}
              </ul>
            </section>
          ) : null}

          <p className="quiet text-[14px] leading-snug">
            Stage list: {snap.configPath}. Edit that file, then reload. Keys while this window is
            focused: d marks the stage, c runs the check, e shows the checklist, x toggles
            click-through.
          </p>
          <button className="btn w-fit" type="button" disabled={pending} onClick={() => void run(reloadPipeline)}>
            Reload stages
          </button>
        </div>
      ) : null}
    </main>
  );
}
