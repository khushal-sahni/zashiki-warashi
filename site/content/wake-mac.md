---
slug: wake-mac
title: "Run a scheduled job while the Mac is asleep — Zashiki Warashi"
h1: "Run jobs while the Mac sleeps"
description: "How Zashiki Warashi runs scheduled jobs on a sleeping Mac: catch-up runs on wake by default, or an optional helper that arms a pmset wake about 90 seconds early. Closed-lid wakes need AC power."
lede: "A schedule is only useful if it fires. Zashiki gives each job one of two policies — catch up on wake, or wake the Mac on time — and is plain about what macOS allows."
related:
  - /scheduled-jobs/
  - /compare/cron/
  - /download/
faq:
  - q: "Can a MacBook with the lid closed wake up and run a job?"
    a: "On AC power, yes, with an exact job and the wake helper installed. On battery, macOS does not honor scheduled wakes with the lid closed, so the job runs on the next wake."
  - q: "Does Zashiki disable sleep?"
    a: "No. Jobs never use pmset disablesleep. The Mac is held awake only while a run is in progress, and goes back to its normal sleep behavior afterwards."
  - q: "Why does the wake helper need an admin password?"
    a: "Scheduling a hardware wake with pmset requires root. The helper is copied into /Library/PrivilegedHelperTools so a user-writable app bundle cannot escalate, and it only touches wakes owned by Zashiki."
  - q: "What if I decline the helper?"
    a: "Exact jobs fall back to catch-up behavior: on time when awake, once on the next wake otherwise. Nothing else changes."
  - q: "If the Mac slept through several slots, do I get several runs?"
    a: "No. launchd coalesces missed calendar slots into a single catch-up run."
---

Laptops sleep. A job scheduled for 03:00 on a MacBook that closed at midnight has three possible outcomes: it never runs, it runs when you open the lid, or something wakes the Mac first. Zashiki makes the choice explicit per job.

## Two policies

| Policy | Mac awake | Mac asleep |
| --- | --- | --- |
| **Optimistic** (default) | Runs on time | Runs once on the next wake |
| **Exact** | Runs on time | Mac wakes about 90 s early, then runs on time |

Optimistic needs nothing extra. It relies on launchd’s calendar behavior: missed slots collapse into **one** catch-up run when the Mac wakes, recorded as a catch-up trigger in run history.

Exact adds a hardware wake before each slot. That needs the optional wake helper.

## The wake helper

Exact jobs schedule a `pmset` wake roughly 90 seconds before the slot, so the Mac is awake and on the network when launchd fires.

- Installed from Jobs settings with one admin prompt
- Copied to `/Library/PrivilegedHelperTools` and run as a root launch daemon
- Reconciles **only** wakes owned by `com.zashiki.warashi`; your other scheduled wakes are left alone
- Declined or removed? Exact jobs degrade to optimistic

## What macOS allows

Be clear on the hardware rules before you trust a 03:00 job:

- **Lid closed, on AC power** — scheduled wakes work
- **Lid closed, on battery** — macOS does not wake for them; the job runs on the next wake

The job inspector shows wake and battery notes for exact jobs so this is visible where you configure it.

## After it wakes

Waking is half the problem. Wi-Fi often reconnects a few seconds late. Each run can:

- **Wait for the network** (with a grace period) and then run, or skip the run as “skipped offline” if the network is required
- **Hold the Mac awake** with `caffeinate -w` for that run only, so a long job is not put back to sleep halfway through
- **Stop at a max runtime** so a hung script does not keep the Mac awake all night

Zashiki never calls `pmset disablesleep`, and it never leaves a keep-awake hold behind after a run ends.

## Setting it up

1. Create a job in Jobs mode — see [Scheduled jobs](/scheduled-jobs/)
2. Set its policy to **Exact**
3. Install the wake helper when prompted (Jobs settings)
4. Leave the Mac on power for closed-lid runs

Install notes (unsigned builds, Gatekeeper): [Download](/download/).
