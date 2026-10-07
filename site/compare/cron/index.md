# vs cron and launchd

> cron and raw launchd plists can schedule scripts on macOS. Zashiki Warashi writes calendar LaunchAgents for you and adds network wait, keep-awake, timeouts, logs, and run history.

Canonical: https://zashiki.anireco.app/compare/cron/

## Short answer

**cron** is a line in a file. It is everywhere, it is simple, and on macOS it runs with a thin environment, skips runs while the Mac sleeps, and mails failures to a local mailbox nobody reads.

**launchd** is what macOS actually uses. Calendar agents catch up after sleep and survive reboots, but you write XML plists by hand, load them with `launchctl`, and build your own logging.

**Zashiki Warashi** writes launchd calendar agents for you and keeps the operator view: run history, logs, and the readiness steps you would otherwise put at the top of every script.

## Side-by-side

| Concern | cron | Raw launchd plist | Zashiki Warashi |
| --- | --- | --- | --- |
| Clock | cron daemon | launchd | launchd |
| Missed while asleep | Skipped | One catch-up run on wake | One catch-up run on wake |
| Wake the Mac on time | No | Separate `pmset` setup | Optional helper, exact jobs only |
| PATH (`fnm`, `nvm`, Homebrew) | Minimal env | Whatever you put in the plist | Your login shell |
| Wait for Wi-Fi | Script it | Script it | Built in, per job |
| Keep awake during run | Script it | Script it | Built in, per run |
| Timeout | Script it | Script it | Max runtime per job |
| Logs and history | Mail / redirect | `StandardOutPath` file | Per-run logs + history in the app |
| Schedule syntax | Any cron expression | Calendar dictionaries | Interval, daily, weekly |
| Agent / script API | — | `launchctl` | Local socket + MCP |

## Who should pick Zashiki

- You have a few scripts that must run on a laptop that sleeps
- You want to see the last run’s exit code and log without hunting for files
- You want a job to wait for Wi-Fi after wake instead of failing its first request
- You want an agent to schedule and inspect jobs through MCP — see [Scheduled jobs](/scheduled-jobs/)

## Who should keep cron or plain launchd

- Servers and always-on machines where sleep is not a concern
- Schedules that need full cron expressions (for example, the last weekday of the month)
- Jobs managed by config management, dotfiles, or another tool that already owns the plist

Using both is fine. Zashiki only manages the agents it creates, and shows everything else read-only.

Download Zashiki: [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).

## FAQ

### Does Zashiki replace crontab?

No. It does not read or edit your crontab, and it does not accept arbitrary cron expressions. It creates its own launchd agents for jobs you define in the app.

### Is Zashiki just a launchd GUI?

It writes launchd agents, so launchd stays the clock. On top it adds network wait, a keep-awake hold, max runtime, logs, run history, and an MCP server.

### Can I keep using my existing plists?

Yes. Other LaunchAgents are shown read-only. Zashiki never edits agents it did not create.

## More rooms

- [Scheduled jobs](https://zashiki.anireco.app/scheduled-jobs/)
- [Run jobs while the Mac sleeps](https://zashiki.anireco.app/wake-mac/)
- [Localhost control plane](https://zashiki.anireco.app/localhost-control-plane/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
