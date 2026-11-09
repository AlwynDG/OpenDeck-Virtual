//! Virtual Stream Deck: a borderless window that fills a monitor and acts as a regular OpenDeck device.
//!
//! The device is registered like a hardware Stream Deck (with the `vd-` namespace). Key images rendered by the
//! main window are routed here by `events::outbound::devices` instead of being sent to USB hardware, and presses
//! in the virtual deck window are forwarded to the normal key down / key up handlers.

use crate::events::inbound::{PayloadEvent, devices::PressPayload};
use crate::shared::{Context, DeviceInfo, config_dir};

use std::collections::HashMap;
use std::sync::LazyLock;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalSize, Monitor, WebviewUrl, WebviewWindowBuilder};

pub const DEVICE_ID: &str = "vd-virtual-1";
pub const WINDOW_LABEL: &str = "virtual-deck";

/// User configuration, stored as `virtual_deck.json` in the OpenDeck config directory.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VirtualDeckConfig {
	/// Whether the virtual deck is registered and shown at all.
	pub enabled: bool,
	pub rows: u8,
	pub columns: u8,
	/// Gap between keys in CSS pixels.
	pub gap: u16,
	/// Monitor name (e.g. `\\.\DISPLAY3` on Windows). When unset, the rightmost portrait monitor is used,
	/// falling back to the rightmost monitor.
	pub monitor: Option<String>,
	pub always_on_top: bool,
	/// Test mode: show the deck in a normal resizable 540x960 window instead of filling a monitor.
	pub windowed: bool,
}

impl Default for VirtualDeckConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			rows: 8,
			columns: 5,
			gap: 16,
			monitor: None,
			always_on_top: true,
			windowed: false,
		}
	}
}

fn config_path() -> std::path::PathBuf {
	config_dir().join("virtual_deck.json")
}

pub fn load_config() -> VirtualDeckConfig {
	let path = config_path();
	match std::fs::read_to_string(&path) {
		Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|error| {
			log::warn!("Invalid virtual deck config at {}: {error}; using defaults", path.display());
			VirtualDeckConfig::default()
		}),
		Err(_) => {
			let config = VirtualDeckConfig::default();
			let _ = std::fs::create_dir_all(config_dir());
			if let Ok(contents) = serde_json::to_string_pretty(&config) {
				let _ = std::fs::write(&path, contents);
			}
			config
		}
	}
}

/// Last image per key position, so the window can redraw everything after (re)loading.
static IMAGES: LazyLock<DashMap<u8, String>> = LazyLock::new(DashMap::new);

pub fn is_virtual(device: &str) -> bool {
	device.starts_with("vd-")
}

#[derive(Clone, Serialize)]
struct ImageEvent {
	position: u8,
	image: Option<String>,
}

pub fn update_image(context: &Context, image: Option<&str>) {
	if context.controller != "Keypad" {
		return;
	}
	match image {
		Some(image) => {
			IMAGES.insert(context.position, image.to_owned());
		}
		None => {
			IMAGES.remove(&context.position);
		}
	}
	if let Some(app) = crate::APP_HANDLE.get() {
		let _ = app.emit_to(
			WINDOW_LABEL,
			"virtual_deck_image",
			ImageEvent {
				position: context.position,
				image: image.map(str::to_owned),
			},
		);
	}
}

pub fn clear_screen() {
	IMAGES.clear();
	if let Some(app) = crate::APP_HANDLE.get() {
		let _ = app.emit_to(WINDOW_LABEL, "virtual_deck_clear", ());
	}
}

#[derive(Serialize)]
pub struct VirtualDeckState {
	rows: u8,
	columns: u8,
	gap: u16,
	images: HashMap<u8, String>,
}

#[tauri::command]
pub fn virtual_deck_get_state() -> VirtualDeckState {
	let config = load_config();
	VirtualDeckState {
		rows: config.rows,
		columns: config.columns,
		gap: config.gap,
		images: IMAGES.iter().map(|entry| (*entry.key(), entry.value().clone())).collect(),
	}
}

#[tauri::command]
pub async fn virtual_deck_press(position: u8, pressed: bool) -> Result<(), crate::events::frontend::Error> {
	let event = PayloadEvent {
		payload: PressPayload {
			device: DEVICE_ID.to_owned(),
			position,
		},
	};
	if pressed {
		crate::events::inbound::devices::key_down(event).await?;
	} else {
		crate::events::inbound::devices::key_up(event).await?;
	}
	Ok(())
}

/// Register the virtual device with OpenDeck. Call after plugins and device sleep have been initialised.
pub async fn register(config: &VirtualDeckConfig) {
	let result = crate::events::inbound::devices::register_device(
		"",
		PayloadEvent {
			payload: DeviceInfo {
				id: DEVICE_ID.to_owned(),
				plugin: String::new(),
				name: "Virtual Stream Deck".to_owned(),
				rows: config.rows,
				columns: config.columns,
				encoders: 0,
				touchpoints: 0,
				infobars: 0,
				// Report as a Stream Deck XL, the closest real device; plugins treat it as a plain keypad.
				r#type: 2,
			},
		},
	)
	.await;
	if let Err(error) = result {
		log::error!("Failed to register virtual deck: {error}");
	}
}

fn pick_monitor(app: &AppHandle, config: &VirtualDeckConfig) -> Option<Monitor> {
	let monitors = app.available_monitors().ok()?;
	if let Some(name) = &config.monitor
		&& let Some(monitor) = monitors.iter().find(|m| m.name().is_some_and(|n| n == name))
	{
		return Some(monitor.clone());
	}
	monitors
		.iter()
		.filter(|m| m.size().height > m.size().width)
		.max_by_key(|m| m.position().x)
		.or_else(|| monitors.iter().max_by_key(|m| m.position().x))
		.cloned()
}

/// Create the virtual deck window. Must be called from the main thread (e.g. Tauri's `setup` hook).
pub fn create_window(app: &AppHandle, config: &VirtualDeckConfig) -> Result<(), tauri::Error> {
	let fullscreen = !config.windowed;
	let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("virtual-deck".into()))
		.title("Virtual Stream Deck")
		.decorations(!fullscreen)
		.resizable(!fullscreen)
		.skip_taskbar(fullscreen)
		.always_on_top(fullscreen && config.always_on_top)
		.shadow(false)
		// Clicking the deck must not steal focus from the app that hotkeys / actions are aimed at.
		.focusable(!fullscreen)
		.focused(false)
		.visible(false)
		.build()?;

	if fullscreen {
		match pick_monitor(app, config) {
			Some(monitor) => {
				log::info!("Showing virtual deck on monitor {:?}", monitor.name());
				window.set_position(*monitor.position())?;
				window.set_size(*monitor.size())?;
			}
			None => log::warn!("No monitor found for virtual deck"),
		}
	} else {
		window.set_size(LogicalSize::new(540.0, 960.0))?;
	}
	window.show()?;
	Ok(())
}
