# How to Install Sys-Widget

After downloading the repository, you need to configure your weather API first so the weather widget will work properly.

---

## Generate Your Weather API Key

1. Go to OpenWeather (`openweathermap.org`)
2. Create an account
3. Click your profile icon at the top right
4. Select **API Keys** from the top menu
5. Enter an API key name (example: your city name)
6. Click create, then copy your API key

---

## Add Your API Key to the Project

Open your terminal and go to the downloaded repository:

```bash
cd "downloaded_repo"
```

Then open the `.env` file.

### Using Nano

```bash
nano .env
```

Paste this line:

```env
WEATHER_API_KEY="your_API"
```

Then save:

* Press `CTRL + O`
* Press `ENTER`
* Press `CTRL + X`

---

### Using Vim

```bash
vim .env
```

Paste this line:

```env
WEATHER_API_KEY="your_API"
```

Then save and quit:

```vim
:wq
```

---

# Installation

While still inside the same project directory, run:

```bash
cargo build --release
cargo run
cargo install --path .
```

---

# Run Automatically on Boot

To make the widget persistent and launch automatically on startup, open your Hyprland configuration file.

Look for either:

* `autostart.conf`
* `custom.conf`

Add this line:

```conf
exec-once = sys-widget
```

Save the file and restart your session (or reload Hyprland).

Your widget should now start automatically on boot.

