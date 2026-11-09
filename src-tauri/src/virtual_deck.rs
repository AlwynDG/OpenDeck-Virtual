//! Virtual Stream Deck: an on-screen window that acts as a regular OpenDeck device.
//!
//! The device is registered like a hardware Stream Deck (with the `vd-` namespace). Key images rendered by the
//! main window are routed here by `events::outbound::devices` instead of being sent to USB hardware, and presses
//! in the virtual deck window are forwarded to the normal key down / key up handlers.
//!
//! Two window modes:
//! - `floating`: a small borderless window you can drag and resize anywhere, like Elgato's Virtual Stream Deck.
//!   Its position and size are remembered.
//! - `monitor`: fills a whole monitor (by default the rightmost portrait monitor).

use crate::events::inbound::{PayloadEvent, devices::PressPayload};
use crate::shared::{Context, DeviceInfo, config_dir};

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::Duration;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, Monitor, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub const DEVICE_ID: &str = "vd-virtual-1";
pub const WINDOW_LABEL: &str = "virtual-deck";

/// Height of the drag bar shown at the top of the floating window, in CSS pixels. Keep in sync with the frontend.
const TOOLBAR_HEIGHT: f64 = 28.0;
/// Default key size of a new floating window, in CSS pixels.
const DEFAULT_KEY_SIZE: f64 = 88.0;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
	Floating,
	Monitor,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Toolbar {
	Always,
	Hover,
	Never,
}

/// Window position and size in physical pixels.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometry {
	pub x: i32,
	pub y: i32,
	pub width: u32,
	pub height: u32,
}

/// User configuration, stored as `virtual_deck.json` in the OpenDeck config directory.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VirtualDeckConfig {
	/// Whether the virtual deck is registered and shown at all.
	pub enabled: bool,
	pub mode: Mode,
	pub rows: u8,
	pub columns: u8,
	/// Gap between keys in CSS pixels.
	pub gap: u16,
	/// Monitor name for `monitor` mode (e.g. `\\.\DISPLAY3` on Windows). When unset, the rightmost portrait monitor
	/// is used, falling back to the rightmost monitor.
	pub monitor: Option<String>,
	pub always_on_top: bool,
	/// Remembered geometry of the floating window.
	pub floating: Option<Geometry>,
	/// Whether the window is shown. Toggled from the tray menu or the window's own hide button.
	pub visible: bool,
	/// Opacity of the window background behind the keys, 0-100. 0 = fully transparent: only the keys float on the desktop.
	pub background_opacity: u8,
	/// Drag bar of the floating window. The window can always be dragged by the space between the keys.
	pub toolbar: Toolbar,
	/// Start OpenDeck's own window hidden in the tray; the deck is what you use day to day.
	pub start_in_tray: bool,
}

impl Default for VirtualDeckConfig {
	fn default() -> Self {
		Self {
			enabled: true,
			mode: Mode::Monitor,
			rows: 8,
			columns: 5,
			gap: 16,
			monitor: None,
			always_on_top: true,
			floating: None,
			visible: true,
			background_opacity: 0,
			toolbar: Toolbar::Never,
			start_in_tray: true,
		}
	}
}

impl VirtualDeckConfig {
	fn sanitised(mut self) -> Self {
		self.rows = self.rows.clamp(1, 16);
		self.columns = self.columns.clamp(1, 16);
		self.gap = self.gap.min(128);
		self.background_opacity = self.background_opacity.min(100);
		self
	}
}

fn config_path() -> std::path::PathBuf {
	config_dir().join("virtual_deck.json")
}

fn load_config() -> VirtualDeckConfig {
	let path = config_path();
	match std::fs::read_to_string(&path) {
		Ok(contents) => serde_json::from_str::<VirtualDeckConfig>(&contents)
			.unwrap_or_else(|error| {
				log::warn!("Invalid virtual deck config at {}: {error}; using defaults", path.display());
				VirtualDeckConfig::default()
			})
			.sanitised(),
		Err(_) => {
			let config = VirtualDeckConfig::default();
			save_config(&config);
			config
		}
	}
}

fn save_config(config: &VirtualDeckConfig) {
	let path = config_path();
	let _ = std::fs::create_dir_all(config_dir());
	match serde_json::to_string_pretty(config) {
		Ok(contents) => {
			// Write to a temporary file first so a crash never leaves a half-written config behind.
			let temp = path.with_extension("json.temp");
			if let Err(error) = std::fs::write(&temp, contents).and_then(|_| std::fs::rename(&temp, &path)) {
				log::error!("Failed to save virtual deck config: {error}");
			}
		}
		Err(error) => log::error!("Failed to serialise virtual deck config: {error}"),
	}
}

static CONFIG: LazyLock<RwLock<VirtualDeckConfig>> = LazyLock::new(|| RwLock::new(load_config()));

fn config() -> VirtualDeckConfig {
	CONFIG.read().unwrap().clone()
}

/// Whether OpenDeck's main window should start hidden in the tray. Call after `APP_HANDLE` is set.
pub fn start_in_tray() -> bool {
	let config = config();
	config.enabled && config.start_in_tray
}

pub fn is_virtual(device: &str) -> bool {
	device.starts_with("vd-")
}

// ---------------------------------------------------------------------------------------------------------------
// Images
// ---------------------------------------------------------------------------------------------------------------

/// Last image per (profile, key position). Kept for every profile so switching back to a profile shows its keys
/// immediately instead of a black frame while the main window re-renders them.
static IMAGES: LazyLock<DashMap<(String, u8), String>> = LazyLock::new(DashMap::new);
/// The profile whose images the window currently shows.
static SHOWN_PROFILE: Mutex<String> = Mutex::new(String::new());
/// Bumped on every image update, so a delayed clear can tell whether new images arrived in the meantime.
static IMAGE_GENERATION: AtomicU64 = AtomicU64::new(0);
static SLEEPING: AtomicBool = AtomicBool::new(false);

fn shown_images() -> HashMap<u8, String> {
	let profile = SHOWN_PROFILE.lock().unwrap().clone();
	IMAGES.iter().filter(|entry| entry.key().0 == profile).map(|entry| (entry.key().1, entry.value().clone())).collect()
}

fn emit<S: Serialize + Clone>(event: &str, payload: S) {
	if let Some(app) = crate::APP_HANDLE.get() {
		let _ = app.emit_to(WINDOW_LABEL, event, payload);
	}
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
	IMAGE_GENERATION.fetch_add(1, Ordering::SeqCst);

	let key = (context.profile.clone(), context.position);
	match image {
		Some(image) => {
			IMAGES.insert(key, image.to_owned());
		}
		None => {
			IMAGES.remove(&key);
		}
	}

	let profile_changed = {
		let mut shown = SHOWN_PROFILE.lock().unwrap();
		if *shown != context.profile {
			*shown = context.profile.clone();
			true
		} else {
			false
		}
	};
	if profile_changed {
		// Replace the whole grid at once with what we remember of the new profile.
		emit("virtual_deck_images", shown_images());
	} else {
		emit(
			"virtual_deck_image",
			ImageEvent {
				position: context.position,
				image: image.map(str::to_owned),
			},
		);
	}
}

pub fn clear_screen() {
	// OpenDeck clears the screen right before switching profiles. Wait a moment: if images of the new profile arrive
	// first, the grid is swapped in one go by `update_image` and nothing needs to be cleared.
	let generation = IMAGE_GENERATION.load(Ordering::SeqCst);
	tauri::async_runtime::spawn(async move {
		tokio::time::sleep(Duration::from_millis(300)).await;
		if IMAGE_GENERATION.load(Ordering::SeqCst) == generation {
			emit("virtual_deck_images", HashMap::<u8, String>::new());
		}
	});
}

/// Brightness 0 means the device is asleep (see `device_sleep`); the window then goes dark until the next press.
pub fn set_brightness(brightness: u8) {
	let sleeping = brightness == 0;
	SLEEPING.store(sleeping, Ordering::SeqCst);
	emit("virtual_deck_sleeping", sleeping);
}

// ---------------------------------------------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------------------------------------------

#[derive(Clone, Serialize)]
pub struct VirtualDeckState {
	rows: u8,
	columns: u8,
	gap: u16,
	mode: Mode,
	always_on_top: bool,
	background_opacity: u8,
	toolbar: Toolbar,
	sleeping: bool,
	images: HashMap<u8, String>,
}

fn state() -> VirtualDeckState {
	let config = config();
	VirtualDeckState {
		rows: config.rows,
		columns: config.columns,
		gap: config.gap,
		mode: config.mode,
		always_on_top: config.always_on_top,
		background_opacity: config.background_opacity,
		toolbar: config.toolbar,
		sleeping: SLEEPING.load(Ordering::SeqCst),
		images: shown_images(),
	}
}

#[tauri::command]
pub fn virtual_deck_get_state() -> VirtualDeckState {
	state()
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

#[tauri::command]
pub fn virtual_deck_get_config() -> VirtualDeckConfig {
	config()
}

#[derive(Serialize)]
pub struct MonitorInfo {
	name: String,
	width: u32,
	height: u32,
	x: i32,
	y: i32,
}

#[tauri::command]
pub fn virtual_deck_list_monitors(app: AppHandle) -> Vec<MonitorInfo> {
	app.available_monitors()
		.unwrap_or_default()
		.iter()
		.filter_map(|monitor| {
			Some(MonitorInfo {
				name: monitor.name()?.clone(),
				width: monitor.size().width,
				height: monitor.size().height,
				x: monitor.position().x,
				y: monitor.position().y,
			})
		})
		.collect()
}

/// Apply new settings from the settings view. Only the parts that changed are reapplied.
#[tauri::command]
pub async fn virtual_deck_set_config(app: AppHandle, new_config: VirtualDeckConfig) -> Result<(), crate::events::frontend::Error> {
	let new = new_config.sanitised();
	let old = {
		let mut current = CONFIG.write().unwrap();
		// Window state is owned by the window itself, not by the settings view.
		let new = VirtualDeckConfig {
			floating: current.floating,
			visible: current.visible,
			..new.clone()
		};
		std::mem::replace(&mut *current, new)
	};
	let new = config();
	save_config(&new);

	let was_registered = old.enabled;
	let layout_changed = old.rows != new.rows || old.columns != new.columns;
	if was_registered && (!new.enabled || layout_changed) {
		deregister().await;
	}
	if new.enabled && (!was_registered || layout_changed) {
		register(&new).await;
	}

	let window_changed = old.enabled != new.enabled || old.mode != new.mode || old.monitor != new.monitor || old.always_on_top != new.always_on_top || has_shadow(&old) != has_shadow(&new);
	if window_changed {
		if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
			window.destroy().map_err(anyhow::Error::from)?;
		}
		if new.enabled {
			create_window(&app, &new).map_err(anyhow::Error::from)?;
		}
	} else {
		emit("virtual_deck_state", state());
	}
	Ok(())
}

/// Show or hide the virtual deck window (tray menu and the window's hide button).
#[tauri::command]
pub fn virtual_deck_set_visible(app: AppHandle, visible: bool) -> Result<(), crate::events::frontend::Error> {
	set_visible(&app, visible).map_err(|error| anyhow::Error::from(error).into())
}

fn set_visible(app: &AppHandle, visible: bool) -> Result<(), tauri::Error> {
	let config = {
		let mut config = CONFIG.write().unwrap();
		config.visible = visible;
		config.clone()
	};
	save_config(&config);
	if !config.enabled {
		return Ok(());
	}
	match app.get_webview_window(WINDOW_LABEL) {
		Some(window) if visible => {
			place_window(app, &window, &config)?;
			window.show()?;
		}
		Some(window) => window.hide()?,
		None if visible => create_window(app, &config)?,
		None => {}
	}
	Ok(())
}

#[tauri::command]
pub fn virtual_deck_set_always_on_top(app: AppHandle, always_on_top: bool) -> Result<(), crate::events::frontend::Error> {
	let config = {
		let mut config = CONFIG.write().unwrap();
		config.always_on_top = always_on_top;
		config.clone()
	};
	save_config(&config);
	if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
		window.set_always_on_top(always_on_top).map_err(anyhow::Error::from)?;
	}
	emit("virtual_deck_state", state());
	Ok(())
}

/// Tray menu entry: toggle the virtual deck window.
pub fn toggle_visible(app: &AppHandle) -> Result<(), tauri::Error> {
	let visible = app.get_webview_window(WINDOW_LABEL).is_some_and(|window| window.is_visible().unwrap_or(false));
	set_visible(app, !visible)
}

// ---------------------------------------------------------------------------------------------------------------
// Device registration
// ---------------------------------------------------------------------------------------------------------------

async fn register(config: &VirtualDeckConfig) {
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

async fn deregister() {
	if let Err(error) = crate::events::inbound::devices::deregister_device("", PayloadEvent { payload: DEVICE_ID.to_owned() }).await {
		log::error!("Failed to deregister virtual deck: {error}");
	}
	IMAGES.clear();
	SHOWN_PROFILE.lock().unwrap().clear();
}

// ---------------------------------------------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------------------------------------------

/// Set up the virtual deck. Must be called from Tauri's `setup` hook, after plugins and device sleep are initialised.
pub fn init(app: &AppHandle) {
	let config = config();
	if !config.enabled {
		return;
	}
	if config.visible
		&& let Err(error) = create_window(app, &config)
	{
		log::error!("Failed to create virtual deck window: {error}");
	}
	tauri::async_runtime::spawn(async move { register(&config).await });
	tauri::async_runtime::spawn(watch_monitors());
}

fn rect_contains(monitor: &Monitor, x: i32, y: i32) -> bool {
	let position = monitor.position();
	let size = monitor.size();
	x >= position.x && y >= position.y && x < position.x + size.width as i32 && y < position.y + size.height as i32
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

/// Where the floating window should go: its remembered geometry if that is still on a connected monitor,
/// otherwise centred on the primary monitor at a size that fits the grid.
fn floating_geometry(app: &AppHandle, config: &VirtualDeckConfig) -> Option<Geometry> {
	let monitors = app.available_monitors().ok()?;
	if let Some(geometry) = config.floating {
		let (cx, cy) = (geometry.x + geometry.width as i32 / 2, geometry.y + 16);
		if monitors.iter().any(|m| rect_contains(m, cx, cy)) {
			return Some(geometry);
		}
	}

	let monitor = app.primary_monitor().ok().flatten().or_else(|| monitors.first().cloned())?;
	let scale = monitor.scale_factor();
	let gap = config.gap as f64;
	let width = config.columns as f64 * DEFAULT_KEY_SIZE + (config.columns as f64 + 1.0) * gap;
	let toolbar = if config.toolbar == Toolbar::Never { 0.0 } else { TOOLBAR_HEIGHT };
	let height = config.rows as f64 * DEFAULT_KEY_SIZE + (config.rows as f64 + 1.0) * gap + toolbar;
	// Never larger than 90% of the monitor.
	let factor = (monitor.size().width as f64 * 0.9 / (width * scale))
		.min(monitor.size().height as f64 * 0.9 / (height * scale))
		.min(1.0);
	let (width, height) = ((width * scale * factor) as u32, (height * scale * factor) as u32);
	Some(Geometry {
		x: monitor.position().x + (monitor.size().width as i32 - width as i32) / 2,
		y: monitor.position().y + (monitor.size().height as i32 - height as i32) / 2,
		width,
		height,
	})
}

fn target_geometry(app: &AppHandle, config: &VirtualDeckConfig) -> Option<Geometry> {
	match config.mode {
		Mode::Floating => floating_geometry(app, config),
		Mode::Monitor => pick_monitor(app, config).map(|monitor| Geometry {
			x: monitor.position().x,
			y: monitor.position().y,
			width: monitor.size().width,
			height: monitor.size().height,
		}),
	}
}

fn place_window(app: &AppHandle, window: &WebviewWindow, config: &VirtualDeckConfig) -> Result<(), tauri::Error> {
	match target_geometry(app, config) {
		Some(geometry) => {
			// Position first: moving onto a monitor with another scale factor can resize the window.
			window.set_position(PhysicalPosition::new(geometry.x, geometry.y))?;
			window.set_size(PhysicalSize::new(geometry.width, geometry.height))?;
		}
		None => {
			log::warn!("No monitor found for virtual deck");
			window.set_size(LogicalSize::new(540.0, 960.0))?;
		}
	}
	Ok(())
}

/// Bumped on every move/resize so only the last one in a burst is written to disk.
static GEOMETRY_GENERATION: AtomicU64 = AtomicU64::new(0);
/// Set while we move the window ourselves, so that is not saved as the user's chosen position.
static PLACING: AtomicBool = AtomicBool::new(false);

fn remember_geometry(window: &WebviewWindow) {
	if PLACING.load(Ordering::SeqCst) || config().mode != Mode::Floating {
		return;
	}
	let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
		return;
	};
	if size.width == 0 || size.height == 0 {
		return; // minimised
	}
	CONFIG.write().unwrap().floating = Some(Geometry {
		x: position.x,
		y: position.y,
		width: size.width,
		height: size.height,
	});
	let generation = GEOMETRY_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
	tauri::async_runtime::spawn(async move {
		tokio::time::sleep(Duration::from_millis(500)).await;
		if GEOMETRY_GENERATION.load(Ordering::SeqCst) == generation {
			save_config(&config());
		}
	});
}

/// Window shadow (and on Windows 11 rounded corners) only for an opaque floating window; on a transparent one the
/// shadow would outline the invisible window instead of the keys.
fn has_shadow(config: &VirtualDeckConfig) -> bool {
	config.mode == Mode::Floating && config.background_opacity == 100
}

/// Create the virtual deck window.
fn create_window(app: &AppHandle, config: &VirtualDeckConfig) -> Result<(), tauri::Error> {
	let floating = config.mode == Mode::Floating;
	let window = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("virtual-deck".into()))
		.title("Virtual Stream Deck")
		.decorations(false)
		.resizable(floating)
		.skip_taskbar(true)
		.always_on_top(config.always_on_top)
		.transparent(true)
		.shadow(has_shadow(config))
		.min_inner_size(120.0, 120.0)
		// Clicking the deck must not steal focus from the app that hotkeys / actions are aimed at.
		.focusable(false)
		.focused(false)
		.visible(false)
		.build()?;

	PLACING.store(true, Ordering::SeqCst);
	let placed = place_window(app, &window, config);
	PLACING.store(false, Ordering::SeqCst);
	placed?;

	let handle = window.clone();
	window.on_window_event(move |event| {
		if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
			remember_geometry(&handle);
		}
	});

	window.show()?;
	// GTK lays out a webview that was sized while hidden at 0x0; size it again now that it is mapped.
	#[cfg(target_os = "linux")]
	{
		PLACING.store(true, Ordering::SeqCst);
		let placed = place_window(app, &window, config);
		PLACING.store(false, Ordering::SeqCst);
		placed?;
	}
	Ok(())
}

/// Monitors come and go (sleep, unplugging, resolution changes). Every few seconds, put the window back where it
/// belongs if it is no longer there.
async fn watch_monitors() {
	loop {
		tokio::time::sleep(Duration::from_secs(3)).await;
		let Some(app) = crate::APP_HANDLE.get() else { continue };
		let config = config();
		if !config.enabled || !config.visible {
			continue;
		}
		let Some(window) = app.get_webview_window(WINDOW_LABEL) else { continue };
		let Some(target) = target_geometry(app, &config) else { continue };
		let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
			continue;
		};
		let current = Geometry {
			x: position.x,
			y: position.y,
			width: size.width,
			height: size.height,
		};
		let misplaced = match config.mode {
			Mode::Monitor => current != target,
			// A floating window stays wherever the user put it; it is only moved back when it is no longer on any
			// connected monitor (e.g. the monitor it was on was unplugged).
			Mode::Floating => {
				let (cx, cy) = (current.x + current.width as i32 / 2, current.y + 16);
				!app.available_monitors().unwrap_or_default().iter().any(|m| rect_contains(m, cx, cy))
			}
		};
		if misplaced {
			log::info!("Repositioning virtual deck window");
			PLACING.store(true, Ordering::SeqCst);
			let _ = window.set_position(PhysicalPosition::new(target.x, target.y));
			let _ = window.set_size(PhysicalSize::new(target.width, target.height));
			PLACING.store(false, Ordering::SeqCst);
			if config.mode == Mode::Floating {
				remember_geometry(&window);
			}
		}
	}
}
