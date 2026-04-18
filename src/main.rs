use chrono::Local;
use dotenvy::dotenv;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use mpris::PlayerFinder;
use relm4::{gtk, ComponentParts, ComponentSender, RelmApp, SimpleComponent};
use std::collections::HashMap;
use std::env;
use sysinfo::{Networks, System};
use std::fs;
use dirs;

mod config;
mod theme;

struct AppModel {
    cpu_usage: f32,
    ram_usage: f32,
    current_time: String,
    current_date: String,
    ssid: String,
    wifi_speed: String,
    eth_speed: String,
    networks: Networks,
    weather_temp: String,
    weather_desc: String,
    weather_icon: String,
    player_status: String,
    player_title: String,
    player_artist: String,
    sys: System,
    prev_rx: HashMap<String, u64>,
}

#[derive(serde::Deserialize)]
struct Config {
    theme: String,
    refresh_interval: u64,
}

fn load_config() -> Config {
    // Correct way: no arguments. It finds /home/zileanne/.config automatically.
    let mut config_path = dirs::config_dir().expect("Could not find config directory");
    
    // Points to /home/zileanne/.config/sys-widget/config.toml
    config_path.push("sys-widget/config.toml");

    let contents = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        // This will now print the EXACT path it is looking for
        println!("Config not found at {:?}, using default fallback.", config_path);
        "theme = 'default'\nrefresh_interval = 1".to_string()
    });

    toml::from_str(&contents).unwrap_or_else(|_| {
        Config { theme: "default".into(), refresh_interval: 1 }
    })
}

#[derive(Debug)]
enum AppMsg {
    UpdateStats,
    UpdateWeather((f32, String, String)),
    UpdatePlayer((String, String, String)),
}

fn format_speed(bytes: u64) -> String {
    if bytes > 1048576 {
        format!("{:.1} MB/s", bytes as f32 / 1048576.0)
    } else {
        format!("{:.1} KB/s", bytes as f32 / 1024.0)
    }
}
use std::process::Command;

fn get_ssid() -> String {
    Command::new("/usr/bin/iwgetid")
        .arg("-r")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or("N/A".into())
}
fn fetch_player_data() -> (String, String, String) {
    let finder = match PlayerFinder::new() {
        Ok(f) => f,
        Err(_) => {
            return (
                "Offline".to_string(),
                "No Media".to_string(),
                "".to_string(),
            )
        }
    };

    if let Ok(player) = finder.find_active() {
        let metadata = player.get_metadata().unwrap_or_default();

        let status = match player.get_playback_status() {
            Ok(mpris::PlaybackStatus::Playing) => "Playing",
            Ok(mpris::PlaybackStatus::Paused) => "Paused",
            _ => "Stopped",
        };

        let title = metadata
            .title()
            .unwrap_or("No Title")
            .chars()
            .take(25)
            .collect();

        let artist = metadata
            .artists()
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".to_string());

        (status.to_string(), title, artist)
    } else {
        (
            "Offline".to_string(),
            "No Media".to_string(),
            "".to_string(),
        )
    }
}

async fn fetch_weather(api_key: &str) -> Option<(f32, String, String)> {
    let city_id = 1715348;
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?id={}&appid={}&units=metric",
        city_id, api_key
    );

    let client = reqwest::Client::new();
    let res = client.get(url).send().await.ok()?;

    if !res.status().is_success() {
        return None;
    }

    let json: serde_json::Value = res.json().await.ok()?;
    let temp = json["main"]["temp"].as_f64()? as f32;
    let weather = json["weather"].as_array()?.get(0)?;
    let icon_code = weather["icon"].as_str()?;
    let raw_desc = weather["description"].as_str()?;

    let desc = raw_desc
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let icon = match icon_code {
        "01d" | "01n" => "☀️",
        "02d" | "02n" | "03d" | "03n" | "04d" | "04n" => "☁️",
        "09d" | "09n" | "10d" | "10n" => "🌧️",
        "11d" | "11n" => "🌩️",
        "13d" | "13n" => "❄️",
        _ => "🌡️",
    };

    Some((temp, desc, icon.to_string()))
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        gtk::Window {
            set_decorated: false,
            set_default_size: (335, 585),

            gtk::Box {
                add_css_class: "main-box",
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 20,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Label { set_label: "Time & Date", add_css_class: "label-title", set_halign: gtk::Align::Start },
                    gtk::Label { #[watch] set_label: &model.current_time, add_css_class: "time-large" },
                    gtk::Label { #[watch] set_label: &model.current_date, add_css_class: "date-sub", set_halign: gtk::Align::End },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Label { set_label: "Weather", add_css_class: "label-title", set_halign: gtk::Align::Start },
                    gtk::Box {
                        set_spacing: 12,
                        gtk::Label { #[watch] set_label: &model.weather_icon, add_css_class: "weather-icon" },
                        gtk::Label { #[watch] set_label: &model.weather_temp, add_css_class: "stats-val" },
                        gtk::Label { set_hexpand: true },
                        gtk::Label { #[watch] set_label: &model.weather_desc, add_css_class: "weather-desc" },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Label { set_label: "Network", add_css_class: "label-title", set_halign: gtk::Align::Start },
                    gtk::Box {
                        gtk::Label { set_label: "   SSID:", add_css_class: "stats-text" },
                        gtk::Label { set_hexpand: true },
                        gtk::Label { #[watch] set_label: &model.ssid, add_css_class: "stats-text" },
                    },
                    gtk::Box {
                        gtk::Label { set_label: "   Wi-Fi:", add_css_class: "stats-text" },
                        gtk::Label { set_hexpand: true },
                        gtk::Label { #[watch] set_label: &model.wifi_speed, add_css_class: "stats-text" },
                    },
                    gtk::Box {
                        gtk::Label { set_label: "   Wired:", add_css_class: "stats-text" },
                        gtk::Label { set_hexpand: true },
                        gtk::Label { #[watch] set_label: &model.eth_speed, add_css_class: "stats-text" },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Label { set_label: "System", add_css_class: "label-title", set_halign: gtk::Align::Start },
                    gtk::Box {
                        set_homogeneous: true,
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            gtk::Label { #[watch] set_label: &format!("{:.0}%", model.cpu_usage), add_css_class: "stats-val" },
                            gtk::Label { set_label: "CPU", add_css_class: "label-title" },
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            gtk::Label { #[watch] set_label: &format!("{:.0}%", model.ram_usage), add_css_class: "stats-val" },
                            gtk::Label { set_label: "RAM", add_css_class: "label-title" },
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    gtk::Label { #[watch] set_label: &model.player_status, add_css_class: "label-title", set_halign: gtk::Align::Start },
                    gtk::Label { #[watch] set_label: &model.player_artist, add_css_class: "player-artist", set_halign: gtk::Align::Start },
                    gtk::Label { #[watch] set_label: &format!("~ {}", model.player_title), add_css_class: "player-title", set_halign: gtk::Align::Start },
                },
            }
        }
    }

    fn init(_: (), root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        dotenv().ok();
        let api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY must be set in .env");

        root.init_layer_shell();
        root.set_layer(Layer::Bottom);
        root.set_namespace("sys-widget");
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Right, true);
        root.set_margin(Edge::Top, 50);
        root.set_margin(Edge::Right, 30);
        root.set_decorated(false);
        root.present();

        use gtk::gdk;
        use gtk::CssProvider;
        use gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;

        let provider = CssProvider::new();
        let cfg = crate::load_config();
        theme::load_theme(&cfg.theme);

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to display"),
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        root.set_css_classes(&["transparent-window"]);

        let mut sys = System::new_all();
        sys.refresh_all();

        let model = AppModel {
            cpu_usage: 0.0,
            ram_usage: 0.0,
            current_time: "".into(),
            current_date: "".into(),
            ssid: "N/A".into(),
            wifi_speed: "0 KB/s".into(),
            eth_speed: "0 KB/s".into(),
            networks: Networks::new_with_refreshed_list(),
            weather_temp: "--°".into(),
            weather_desc: "...".into(),
            weather_icon: "  ".into(),
            player_status: "Offline".into(),
            player_title: "No Media".into(),
            player_artist: "".into(),
            sys,
            prev_rx: HashMap::new(),
        };

        let widgets = view_output!();

        let s_stats = sender.clone();
        relm4::spawn_local(async move {
            loop {
                s_stats.input(AppMsg::UpdateStats);
                let player_data = fetch_player_data();
                s_stats.input(AppMsg::UpdatePlayer(player_data));
                let interval = cfg.refresh_interval; 
                relm4::gtk::glib::timeout_future_seconds(interval as u32).await;
            }
        });

        let s_weather = sender.clone();
        let weather_key = api_key.clone();
        relm4::spawn_local(async move {
            loop {
                if let Some(weather) = fetch_weather(&weather_key).await {
                    s_weather.input(AppMsg::UpdateWeather(weather));
                }
                relm4::gtk::glib::timeout_future_seconds(900).await;
            }
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::UpdateStats => {
                self.sys.refresh_cpu_usage();
                self.sys.refresh_memory();

                self.cpu_usage = self.sys.global_cpu_info().cpu_usage();
                self.ram_usage =
                    (self.sys.used_memory() as f32 / self.sys.total_memory() as f32) * 100.0;
                let ssid = get_ssid();
                if self.ssid != ssid {
                    self.ssid = ssid;
                }
                let now = Local::now();
                self.current_time = now.format("%H:%M:%S").to_string();
                self.current_date = now.format("%A, %B %d").to_string();

                self.networks.refresh();

                for (name, data) in &self.networks {
                    let current = data.received();
                    let prev = self.prev_rx.get(name).copied().unwrap_or(current);
                    let speed = current.saturating_sub(prev);

                    self.prev_rx.insert(name.clone(), current);

                    if name.starts_with("wl") {
                        self.wifi_speed = format_speed(speed);
                    } else if name.starts_with("eth") {
                        self.eth_speed = format_speed(speed);
                    }
                }
            }
            AppMsg::UpdateWeather((temp, desc, icon)) => {
                self.weather_temp = format!("{:.1}°C", temp);
                self.weather_desc = desc;
                self.weather_icon = icon;
            }
            AppMsg::UpdatePlayer((status, title, artist)) => {
                self.player_status = status;
                self.player_title = title;
                self.player_artist = artist;
            }
        }
    }
}

fn main() {
    let _app_config = load_config(); 

    if std::env::var("GDK_BACKEND").is_err() {
        std::env::set_var("GDK_BACKEND", "wayland");
    }

    let app = RelmApp::new("com.zileanne.sys-widget"); 

    app.run::<AppModel>(());
}

