# CS2shock-XPERIMENT OpenShock Support

# THIS CODE IS UNAFFILIATED WITH THE REAL VERSION BY VolcanoCookies
# THIS IS A MODIFIED VERSION WITH THE PURPOSE OF USING OpenShock INSTEAD OF PiShock
# THEIR CODE IS NOT AS SHIT AS THE CODE IN THIS REPO (I THINK)
# THIS CODE IS A JERRYRIGED VERSION AND IS EXPERIMENTAL AND COMES WITH **NO SUPPORT**

[![CI](https://github.com/NanashiTheNameless/cs2shock-XPERIMENT/actions/workflows/ci.yml/badge.svg)](https://github.com/NanashiTheNameless/cs2shock-XPERIMENT/actions/workflows/ci.yml)
[![Release](https://github.com/NanashiTheNameless/cs2shock-XPERIMENT/actions/workflows/release.yml/badge.svg)](https://github.com/NanashiTheNameless/cs2shock-XPERIMENT/actions/workflows/release.yml)

---

## ⚠️ CRITICAL DISCLAIMER: AI-GENERATED CODE AHEAD ⚠️

> **Listen up, dear user.** This codebase was lovingly ~~Buttfucked~~\*_ahem_\*Tweaked by an AI – you know, those super reliable entities that are *totally* known for writing production-ready, bug-free code. So naturally, you should trust it implicitly with your physical wellbeing and nervous system. What could possibly go wrong?

**In case the sarcasm wasn't thick enough, let me spell it out:**

- 🤖 This was written by an AI. An AI that has never actually *used* OpenShock hardware.
- 🤷 The AI has no concept of pain, discomfort, or why shocking yourself for dying in a video game might be questionable life choices.
- 🎲 The code *might* work. Or it might shock you at random intervals. Or continuously. Who knows? Debugging is for humans!
- 🔥 If this burns down your house, shocks your cat, or causes your OpenShock to achieve sentience and start a robot uprising, that's on YOU buddy.
- 📖 The AI read the API docs once. *Once.* And probably hallucinated half of them.
- 🧂 Take everything here with a grain of salt the size of Jupiter. Actually, make that Saturn – it has better rings.

**Legal Department's Input (they're screaming):**
- ⚖️ Use this code entirely at your own risk and peril
- 🏥 We are not responsible for any injuries, discomfort, or existential dread
- 💀 Seriously, test this thoroughly before using it on yourself
- 🧪 Maybe start with the lowest possible settings. Or better yet, don't use it at all.
- 🎯 The AI's idea of "safe defaults" is... let's call it "optimistic"

**Reality Check:**
- This code interfaces with hardware that delivers electrical shocks
- To your body
- Based on video game events
- Written by a language model
- Are you *really* sure about this?

**Advanced Disclaimer:**
- If you're reading this and thinking "the AI is being too cautious," *you are the problem*
- If you're already in pain from using this, perhaps reconsider your life choices
- If this works perfectly for you, please tell us how, because the AI certainly doesn't know

Remember: **Friends don't let friends run untested AI code connected to shock devices.** But hey, I'm just text on a screen. You do you. 🤡

---

## How it works

Simple, you get shocked when you die during a live match (so not warmup).

Matches do not need to be premier or comp, can be any match.

This project uses **OpenShock** (not PiShock). You'll need an OpenShock account and device.

There are two modes of zapping, either random that picks a value between your configured min and max, everytime you die. Or LastHitPercentage which takes a value depending on the percentage of health you had before you died, so if you had 25hp and died you will be zapped for 25% of your configured max.

There are also two options to beep whenever a match starts, and whenever a round starts. For you forgetful folks.

## Download

GitHub Actions builds a Windows x64 ZIP on pushes and pull requests to `main` or `dev`. You can also start a build from the **Actions** tab by choosing **CI** and **Run workflow**. Open a successful run and download the `cs2shock-windows-x64` artifact near the bottom of its summary page.

*Linux and macOS support has been abandoned because let's be real, you're playing CS2 on Windows anyway.*

## Usage

1. Find your install directory, to do so go to steam, right click on Counter-Strike 2, go to `Manage > Browse Local Files`.
2. Put `gamestate_integration_cs2shock.cfg` in the `game/csgo/cfg` folder. (NOT THE `csgo/cfg` folder). You can now close this folder.
3. You can now run `cs2shock.exe`.

Once you save your settings once, a `config.json` file will be placed next to `cs2shock.exe`.

### Configuration

The application requires OpenShock API credentials:

- **API token**: Create one in your OpenShock account settings.
- **Shockers**: Enter your token, click **Discover shockers**, and select the shockers you want to control. You can also enter shocker UUIDs manually when discovery is unavailable.

**Test beep** uses the current settings in the form, including unsaved edits. Save your settings to use them for game events. The app sends game events to every selected shocker. The log view is available from the app; the Windows release build starts without a separate console window.

You can obtain these from your OpenShock account at [OpenShock](https://openshock.app/).

Duration values in the app and config file are in **seconds** (1–15). OpenShock receives milliseconds after conversion.

Example `config.json`:
```json
{
  "shock_mode": "LastHitPercentage",
  "min_duration": 1,
  "max_duration": 3,
  "min_intensity": 15,
  "max_intensity": 83,
  "beep_on_match_start": false,
  "beep_on_round_start": true,
  "api_token": "your-openshock-api-token-here",
  "shocker_ids": ["first-shocker-uuid", "second-shocker-uuid"],
  "api_server": "https://api.openshock.app"
}
```

## Building from source

### Prerequisites

-   [Rust installed](https://doc.rust-lang.org/cargo/getting-started/installation.html)

1. Clone the repository and open its folder.
2. Double-click `build.bat`, or run it from Command Prompt.
3. Find the packaged app in `dist/cs2shock-windows-x64.zip`.

The script requires Rust and Cargo. It builds a Windows release executable and packages the app files. You can also run `cargo build --release` directly; its executable is at `target/release/cs2shock.exe`.
