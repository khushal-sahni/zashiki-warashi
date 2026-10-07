---
slug: scheduled-jobs
title: "Schedule scripts on macOS with launchd — Zashiki Warashi"
h1: "Scheduled jobs on your Mac"
description: "Run scripts on a schedule on macOS: Zashiki Warashi writes per-job launchd agents, waits for the network, keeps the Mac awake for the run, and keeps logs and run history — even with the app closed."
lede: "launchd is the most reliable clock on a Mac and the least pleasant to operate. Zashiki keeps launchd as the clock and adds the parts you would otherwise script by hand: network wait, keep-awake, timeouts, logs, and history."
related:
  - /wake-mac/
  - /compare/cron/
  - /start-stop/
  - /download/
faq:
  - q: "Do jobs run when Zashiki is closed?"
    a: "Yes. Each job is its own user LaunchAgent. launchd starts it on schedule whether or not the app window is open."
  - q: "What schedules are supported?"
    a: "Every N minutes (N divides 60), every N hours (N divides 24), daily at a time, or weekly on chosen weekdays at a time. Each maps directly onto launchd StartCalendarInterval."
  - q: "What happens to a run that was due while the Mac slept?"
    a: "launchd coalesces missed slots into one catch-up run on the next wake. Exact jobs can wake the Mac shortly before the slot instead — see the wake page."
  - q: "Does Zashiki write anything into my project repo?"
    a: "No. Jobs, runs, and logs live in app data on your Mac. Your script stays where it is and owns the work."
  - q: "Can an AI agent manage jobs?"
    a: "Yes. The app binary serves an MCP server over stdio with tools to list, create, pause, and run jobs and read run logs."
---

Most developers end up with a handful of scripts that should run on a schedule: a nightly ingest, a backup, a scraper, a report. On macOS the options are `cron` (deprecated in spirit, quiet on failure) or hand-written launchd plists (reliable, but no history and no logs worth reading).

**Zashiki Warashi** keeps launchd as the clock and puts a readiness layer and an operator view on top.

## What a job is

A job is a command, an optional working directory, and a schedule:

| Schedule | Example |
| --- | --- |
| Interval | Every 15 minutes, every 6 hours |
| Daily | 07:30 every day |
| Weekly | Mon, Wed, Fri at 09:00 |

Intervals must divide the hour (1, 2, 3, 4, 5, 6, 10, 12, 15, 20, 30 minutes) or the day (1, 2, 3, 4, 6, 8, 12 hours) so every slot lands on a real calendar time launchd can express.

The command runs through your **login shell** (`$SHELL -lc`), the same way project start does on [Start and stop](/start-stop/). `fnm`, `nvm`, Homebrew, and `uv` resolve like they do in Terminal.

## What Zashiki adds around each run

When launchd fires a job, the agent calls the Zashiki binary, which:

1. **Waits for the network** — best effort (wait, then run anyway), required (skip the run if still offline), or none
2. **Holds the Mac awake** for that run only, via `caffeinate -w` on the run’s process
3. **Enforces a max runtime** if you set one, then stops the process
4. **Records the run** — trigger (calendar, catch-up, manual), status, exit code, and a log file

Status lands in one of: succeeded, failed, timed out, skipped offline, or interrupted.

## Operate it like the rest of the house

Jobs live in a Jobs mode next to Projects, using the same pane layout:

- A list with next fire time and last result
- An inspector with **Run now**, **Edit**, **Pause**, and delete
- A run log pane with a run picker and live follow for the current run

Pausing a job unloads its LaunchAgent. Other LaunchAgents on your Mac are shown read-only; Zashiki never edits agents it did not create.

## Asleep, lid closed, on battery

Default jobs are **optimistic**: on time if the Mac is awake, otherwise once on the next wake. **Exact** jobs can wake the Mac shortly before each slot through an optional helper. The limits are macOS limits, and the details are on [Run jobs while the Mac sleeps](/wake-mac/).

## Scripts and agents

The same binary exposes a local socket and an MCP server (`<app> job mcp`, shown in Jobs settings). Tools:

- `list_jobs`, `upsert_job`, `set_job_enabled`, `run_job`
- `job_runs`, `job_log`

An agent can schedule the script it just wrote and read the log of the last run. Zashiki owns the schedule, wake, network wait, and logs; the command owns the work.

## Compared to cron

Short version: cron is a line in a file, Zashiki is launchd plus a run history. Longer version: [vs cron and launchd](/compare/cron/).

Install notes (unsigned builds, Gatekeeper): [Download](/download/).
