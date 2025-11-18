# Rusty Browser

![Status](https://img.shields.io/badge/Status-Stable-success)
![Language](https://img.shields.io/badge/Made%20With-Rust-orange)
![Engine](https://img.shields.io/badge/Engine-WebKitGTK%206.0-blue)
![License](https://img.shields.io/badge/License-Apache%202.0-yellow)

**Rusty Browser** is a lightweight, privacy-focused web browser built for the Modern Linux Desktop. It leverages the memory safety of **Rust** and the modern rendering capabilities of **GTK4** and **WebKit6**.

It is a power-user tool designed to blend seamlessly with KDE Plasma and GNOME while offering advanced features.

## ✨ Features

* **Native Performance:** Built on `gtk4` and `webkit6` with custom rendering paths for high efficiency.
* **Amnesia Mode:** A true incognito mode. Runs entirely in RAM. Closing the browser wipes all data instantly.
* **Native AdBlock:** Built-in CSS injection blocks ads and tracking frames at the engine level for maximum speed.
* **Smart Omnibar:** Intelligent routing detects search queries vs. URLs automatically.
* **Hardware Accelerated Video:** Full GStreamer pipeline integration for smooth 1080p/4K playback.
* **Custom Theming:** Supports dynamic theming (e.g., Liquid Glass) and integrates with system themes.

---

## 📦 Installation

### Prerequisites
Rusty Browser relies on modern system libraries. You need **GTK4** and **WebKitGTK 6.0**.

**Arch Linux / Manjaro / Aurora:**
```bash
sudo pacman -S base-devel gtk4 libadwaita webkitgtk-6.0 gst-plugins-bad gst-plugins-ugly gst-libav

Debian / Ubuntu (22.04+):
Bash

sudo apt install build-essential libgtk-4-dev libadwaita-1-dev libwebkitgtk-6.0-dev libgstreamer1.0-dev

Build from Source

    Clone the repository:
    Bash

git clone [https://github.com/Wyind/rusty_browser.git](https://github.com/Wyind/rusty_browser.git)
cd rusty_browser

Build in release mode:
Bash

cargo build --release

Run:
Bash

    ./target/release/rusty_browser

🛠️ Configuration

The browser creates a configuration folder at ~/.config/rusty_browser/settings.json. You can use the Settings (⚙️) menu inside the app to toggle features like Hardware Acceleration, AdBlock, and Amnesia Mode.

⚖️ License

Distributed under the Apache License 2.0. See LICENSE for more information.

🤝 Contributing

Contributions are welcome!

    Fork the Project

    Create your Feature Branch (git checkout -b feature/AmazingFeature)

    Commit your Changes (git commit -m 'Add some AmazingFeature')

    Push to the Branch (git push origin feature/AmazingFeature)

    Open a Pull Request

Built by [Wyind].
