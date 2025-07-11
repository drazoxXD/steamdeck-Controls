# SteamDeck Wireless Xbox360 Controller

Turn your SteamDeck into a wireless Xbox 360 controller! This project sends button presses and analog stick data over your local network to a WebSocket server, letting your SteamDeck act as a gamepad for your PC.

---

## 🚀 Setup Guide

### 1. Install VIGEmBus

- Download and install [VIGEmBus](https://github.com/nefarius/ViGEmBus).
- Restart your PC after installation.

### 2. Download SteamDeck Controls

- Get the latest release of **steamdeck-controls**. The latest commits include an automatic build workflow.
- Download `steamdeck.exe` and transfer it to your SteamDeck.
- Add `steamdeck.exe` to Steam. (Tested with Proton-GE 10)

### 3. Set Up the Server

- Download `server.exe` and run it on your PC.

### 4. Connect Your Devices

- Find your PC's IP address and port (format: `IP:PORT`).  
  ![How to find your IP](https://github.com/user-attachments/assets/b1630635-ed50-4f2c-8d95-739d63acf377)
- On your SteamDeck, launch the client and enter the IP:PORT of your PC.  
  *(Tip: Use `STEAM + X` to bring up the on-screen keyboard)*

### 5. Play!

- You're all set! Your SteamDeck should now function as a wireless Xbox 360 controller for your PC.
- The server displays the current latency in milliseconds to help troubleshoot connection issues.

---

## ❓ FAQ

**Why should i use this?**  
> Well, it's better to use this because other methods simply stream the entire game, which can add more latency. Also, when you're done, you need to fully restart your deck so the controls work again! (I'm mainly talking about VirtualHere, which is almost the same as this, but that "steals" the control from Steam, so you need a full restart to shut it down!)

**Will there be a UI overhaul?**  
> No, there are currently no plans to redesign the interface.

**Is this safe to use?**  
> You are encouraged to review the open-source code yourself.  
> For security, avoid using the default port (8080) if it is already open or in use on your network.  
> You can edit the code to specify a different port if needed.

---

## 📝 Notes

- Make sure both your PC and SteamDeck are on the same local network.
- For best results, use a stable Wi-Fi connection.
- Feedback, contributions, and bug reports are welcome!

---

## 🚦 Project Status & Maintenance

This project is provided as-is and is not guaranteed to receive further development or feature updates.  
**I am not required to develop this project further.**  
Bug reports and pull requests are welcome, but please understand that ongoing support or new features may not be provided.

---

## ⚠️ Disclaimer

This software is provided for educational and personal use only.

- **I am not responsible if someone gains unauthorized access ("hacks") via the built-in WebSocket server.**  
- Use at your own risk and always consider your network security.
- If you are concerned about security, review the code and consider changing the default port or implementing your own security measures.

---

## 📄 License

This project is licensed under the [GNU General Public License v3.0 (GPL-3.0)](https://www.gnu.org/licenses/gpl-3.0.html).  
You are free to use, modify, and distribute this software under the terms of the GPL-3.0 license.
