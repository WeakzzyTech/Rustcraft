<p align="center">
  <h1 align="center">Rustcraft</h1>
  <p align="center">A high-performance Minecraft: Java Edition client, written from scratch in Rust and Vulkan</p>
  <p align="center">
    <a href="https://github.com/RustcraftMC/Client/actions/workflows/ci.yml"><img src="https://github.com/RustcraftMC/Client/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
    <img src="https://img.shields.io/badge/Minecraft-1.20%20to%2026.3-5D9C3F" alt="Minecraft 1.20 to 26.3">
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-2A2470" alt="Windows, macOS and Linux">
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue" alt="GPL-3.0 license"></a>
  </p>
  <p align="center">
    <a href="#download"><b>Download</b></a> · <a href="https://discord.gg/ucBA55bHPR"><b>Discord</b></a> · <a href="./BUILDING.md"><b>Build from source</b></a> · <a href="./CONTRIBUTING.md"><b>Contributing</b></a>
  </p>
</p>

---

Rustcraft contains no Mojang code, and joins vanilla servers from 1.20 through 26.3.

> [!NOTE]
> Rustcraft is a fork of the [Pomme client](https://github.com/PommeMC/Client).
> The original project and its copyright belong to the Pomme authors; this fork
> renames the project and continues development independently.

<p align="center">
  <img width="100%" alt="Rustcraft flying over a jungle" src="https://github.com/user-attachments/assets/c35e717e-3535-4f4f-9539-9e62dd482e04" />
</p>

## Download

Get Rustcraft from the [latest release](https://github.com/RustcraftMC/Client/releases/latest):

- **Windows x64**: `rustcraft-client-windows-x64-setup.exe`
- **macOS (Apple Silicon)**: `rustcraft-client-macos-arm64.dmg`, then drag Rustcraft to Applications
- **Linux x64**: `rustcraft-client-linux-x64-gnu.AppImage`
- **Android (ARM64)**: Build from source via `adb install ./target/...` or see [BUILDING.md](./BUILDING.md)
- **iOS**: Build from source via Xcode; see [BUILDING.md](./BUILDING.md)

It plays offline by default. On first run, a `local` account is automatically created and selected. You can also add additional local accounts via the launcher's **Add Account** dialog, or pass `--username <name>` directly on the command line. Microsoft sign-in is optional; online-mode servers require it. The builds aren't signed yet, so the first launch needs one extra step, described in the release notes.

### Local accounts

- A `local` account is auto-created on first launch and selected by default
- Click **Add** in the account dropdown to create additional local accounts with custom names
- Local accounts play offline: the client derives the same vanilla offline UUID (`MD5("OfflinePlayer:<name>")`), so worlds/servers treat you consistently
- Removing the last local account re-creates `local`, so the launcher always has a default
- Microsoft sign-in remains a separate option for online-mode servers

### Releases per platform

- **Windows x64**: Installer (`setup.exe`) — includes JRE bundling, start menu shortcut, and auto-update support
- **macOS (Apple Silicon)**: DMG disk image — drag-and-drop application, verified signature pending
- **Linux x64**: AppImage — single executable, no installation needed, works on most modern distros
- **Android (ARM64)**: Source-only; build with `adb` or see [BUILDING.md](./BUILDING.md) for full instructions
- **iOS**: Source-only; build with Xcode; see [BUILDING.md](./BUILDING.md) for full instructions

Early Android and iOS builds build from source: see [BUILDING.md](./BUILDING.md).

## Why Rustcraft?

- **No JVM**: a native binary, so there's no Java to install or tune.
- **Vanilla behaviour**: physics, rendering, menus and networking are ported line
  by line from the decompiled game, and checked against it.
- **Tested against real servers**: scripted [in-game tests](./tests/ingame/README.md)
  run the client against a vanilla 26.2 server and screenshot the results.
- **Many versions, one client**: per-version protocol tables are generated from
  each version's decompiled source, and other versions are translated to 26.2.

> [!WARNING]
> Rustcraft is in active development. Expect missing features and bugs, and please
> report them in [Issues](https://github.com/RustcraftMC/Client/issues) or on Discord.

## Features

- **Rendering**: chunk meshing, GPU frustum and cave-occlusion culling, smooth lighting, water and lava, mobs with skins, particles, weather, clouds and sky
- **Singleplayer**: worlds run on [SteelMC](https://github.com/Steel-Foundation/SteelMC), a Rust server embedded in the client
- **Audio**: vanilla sound events, music and subtitles through OpenAL Soft
- **Resource packs**: local and server-sent packs, applied live
- **Inventories**: survival and creative inventories, crafting, furnaces, anvils, enchanting, chests, bundles and the recipe book
- **Controls**: rebindable keys and controllers on desktop, touch controls on phones
- **Friends**: the Minecraft friends list, requests and who's online
- **Updates**: installs per platform and keeps itself up to date

<details>
<summary>Everything else</summary>

- Vanilla-exact movement, collision, swimming and drowning
- Block entities, dropped items, block overlays and hand animation
- HUD and menus: health, hunger, air, status effects, hotbar, boss bars, titles, waypoints, advancement toasts, player list, F3 debug, chat, pause menu, options, server list, world list
- Create, edit and delete singleplayer worlds from the menu
- Optional Microsoft sign-in from the menu for online servers, with tokens stored in the OS keyring
- Secure chat: signed messages, as vanilla sends them
- Discord presence for the menu, servers and worlds

</details>

## Building from source

See [BUILDING.md](./BUILDING.md) for the client and launcher builds, running
the client standalone, and the development commands.

## Contributing

Contributions are welcome.
Please open an issue first to discuss what you'd like to change, and see
[CONTRIBUTING.md](./CONTRIBUTING.md) for the workflow.

## License

Rustcraft is licensed under the [GNU General Public License v3.0 or later](./LICENSE)
(GPL-3.0-or-later). Copyright (C) 2026 Purdze.

`rustcraft-singleplayer` links [SteelMC](./third_party/SteelMC) and is therefore
AGPL-3.0-or-later. The [allocator crate](./rustcraft-gpu-allocator) is MIT, a port
of the [gpu-allocator crate](https://github.com/Traverse-Research/gpu-allocator)
by Traverse Research, and [rustcraft-block](./rustcraft-block) is likewise MIT. See
[THIRD_PARTY_LICENSES.md](./THIRD_PARTY_LICENSES.md) for the other third-party code.

Rustcraft is not affiliated with or endorsed by Mojang Studios or Microsoft.
Minecraft is a trademark of Mojang Studios.

## Community

[![Discord](https://img.shields.io/discord/1483578136544280618?color=5865F2&label=Discord&logo=discord&logoColor=white)](https://discord.gg/ucBA55bHPR)
[![Sponsor](https://img.shields.io/badge/Sponsor-Purdze-ea4aaa?logo=githubsponsors&logoColor=white)](https://github.com/sponsors/Purdze)

<details>
<summary>Star history</summary>

<a href="https://www.star-history.com/?repos=RustcraftMC%2FClient&type=date&logscale=&legend=top-left">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=RustcraftMC/Client&type=date&theme=dark&logscale&legend=top-left&sealed_token=r7ib_n7xrIE4aUaRqlz9KCFjr5-Cpocxw6LPtk06-CcFfxLAEjzgPfz5bSzzGcSnbhRnkbN6KwRem4jwXnRJzcsuZYIZX5tyIAmYKFIc60c_2HBXRhGHA4rlvmyr2MXfeZDe2thqz-maf8Tdh5FxHAWJUFflGQP1JjoIfbyUZ0GW6pMxxJ8qfga5eta0" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=RustcraftMC/Client&type=date&logscale&legend=top-left&sealed_token=r7ib_n7xrIE4aUaRqlz9KCFjr5-Cpocxw6LPtk06-CcFfxLAEjzgPfz5bSzzGcSnbhRnkbN6KwRem4jwXnRJzcsuZYIZX5tyIAmYKFIc60c_2HBXRhGHA4rlvmyr2MXfeZDe2thqz-maf8Tdh5FxHAWJUFflGQP1JjoIfbyUZ0GW6pMxxJ8qfga5eta0" />
   <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=RustcraftMC/Client&type=date&logscale&legend=top-left&sealed_token=r7ib_n7xrIE4aUaRqlz9KCFjr5-Cpocxw6LPtk06-CcFfxLAEjzgPfz5bSzzGcSnbhRnkbN6KwRem4jwXnRJzcsuZYIZX5tyIAmYKFIc60c_2HBXRhGHA4rlvmyr2MXfeZDe2thqz-maf8Tdh5FxHAWJUFflGQP1JjoIfbyUZ0GW6pMxxJ8qfga5eta0" />
 </picture>
</a>

</details>
