<div align="center">

# ShiftTab ⚡

**A lightning-fast, native Zsh autocomplete TUI**

*Dynamically parse manuals and help outputs on the fly to generate context-aware suggestions directly in your CLI.*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-Optimized-orange.svg)](https://www.rust-lang.org/)
[![Zsh](https://img.shields.io/badge/Zsh-Ready-blue.svg)](https://www.zsh.org/)

[🚀 Quick Start](#-quick-start) &#124; [✨ Features](#-features) &#124; [🛠️ Installation](#%EF%B8%8F-installation) &#124; [⚙️ Configuration](#%EF%B8%8F-configuration) &#124; [💡 Usage](#-usage)

</div>

---

## Preview

> *I will need to add a screen shot here later*

---

## Quick Start

ShiftTab operates entirely within the background of your terminal workflow:

1. Type a command in your Zsh prompt (e.g., `tar ` or `docker run -`).
2. Press `Shift + Tab`.
3. ShiftTab intercepts the current buffer, parses the relevant manuals, and displays a navigable list of flags and arguments.
4. Select your desired flag to insert it directly into your command line.

**Standard CLI Commands:**
You can also interact with the ShiftTab binary directly for standard system queries:
* `ShiftTab --help` (or `-h`): Print usage synopsis.
* `ShiftTab --version` (or `-v`): Display the current build version.

---

## Features

* **Dynamic Parsing:** Automatically generates completions by reading standard system manuals and help outputs on the fly.
* **Dual Interface Modes:**
  * **Extended Mode:** A full-screen TUI featuring a responsive split-pane layout, providing command descriptions alongside active `tldr` community examples.
  * **Compact Mode:** An inline, minimalist completion menu for fast navigation without obscuring your terminal history.
* **TLDR Integration:** Automatically fetches and formats practical command examples via the system's `tldr` cache, saving you a trip to the browser.
* **True Color Formatting:** Full RGB ANSI styling tailored for the Catppuccin Mocha palette by default, seamlessly integrating with modern terminal themes.
* **Lightweight & Fast:** Written in Rust with aggressive release optimizations, ensuring instantaneous startup times and zero observable Zsh latency.

---

## Installation

ShiftTab provides multiple installation paths to suit varying system administration preferences.

### 1. Automated Installation Script (Recommended)
The fastest way to install on any standard Linux/macOS system. This script dynamically pulls the latest optimized binary, validates its SHA256 checksum, sets up your configuration directories, and securely appends the initialization hook to your `~/.zshrc`.

```bash
curl -sSL https://raw.githubusercontent.com/smrini/ShiftTab/master/install.sh | bash
```

### 2. Standalone Binary (Manual)
For users who prefer to manage their own binaries or install via Cargo.

Clone this repo, then build and install with cargo.

```bash
# Build and install the binary
cargo install --path .

# Or download the pre-compiled binary and place it in your $PATH
# e.g., /usr/local/bin/ShiftTab
```

Once the binary is in your `PATH`, add the following line to your `~/.zshrc` to bind ShiftTab to your shell's Zsh Line Editor (ZLE):

```bash
eval "$(ShiftTab --init zsh)"
```

Then Make sure to reload zsh cofiguration:

```bash
source ~/.zshrc
```

### 3. Arch Linux (AUR)
A standard `PKGBUILD` is provided for Arch-based distributions. You can build and install it natively through `makepkg` or an AUR helper.

```bash
# Clone the repository and build the package
git clone https://github.com/smrini/ShiftTab.git
cd ShiftTab
makepkg -si
```

---

## Configuration

Following the XDG Base Directory specification, ShiftTab automatically generates its configuration file at `~/.config/shifttab/config.toml` upon first run. 

You can customize the rendering mode, integration options, and your exact RGB color palette:

```toml
# Display mode: "extended" (full screen TUI) or "compact" (inline TUI)
mode = "extended"

# Toggle the TLDR example fetching in Extended mode
enable_tldr = true

# ANSI True Color Palette Configuration (R, G, B)
# Defaults to the Catppuccin Mocha theme.
[colors]
base = [36, 39, 58]           # Background color
text = [202, 211, 245]        # Text color
highlight = [198, 160, 246]   # Highlight/selected item color
border = [73, 77, 100]        # Border/separator color

# Keybinding customization
# Navigation uses modifiers (Ctrl or Alt) so you can type normally
# For example: Ctrl+K moves up, plain 'k' is just a regular character
[keys]
up = "k"                       # Move up when pressed with the modifier key
down = "j"                     # Move down when pressed with the modifier key
modifier = "ctrl"              # Modifier for navigation: "ctrl", "alt", or "none"

# When modifier = "ctrl": Use Ctrl+K to navigate up, Ctrl+J to navigate down
# When modifier = "alt":  Use Alt+K to navigate up, Alt+J to navigate down
# When modifier = "none": hjkl always navigate (same as old behavior)
# Arrow keys always work for navigation
# Selection: Enter always selects
# Exit: Escape always exits
```

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
