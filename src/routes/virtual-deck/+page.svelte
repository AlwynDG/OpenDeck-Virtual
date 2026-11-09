<script lang="ts">
	// Virtual Stream Deck window: draws the keys of the device's selected profile itself and forwards presses to the
	// backend, so it works without OpenDeck's main window. In floating mode it also has an optional drag bar
	// (pin / hide) and a resize grip, like Elgato's Virtual Stream Deck.
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { Context } from "$lib/Context";
	import type { Profile } from "$lib/Profile";

	import EyeSlash from "phosphor-svelte/lib/EyeSlash";
	import PushPin from "phosphor-svelte/lib/PushPin";
	import VirtualDeckKey from "../../components/VirtualDeckKey.svelte";

	import { initPortBase } from "$lib/ports";

	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import { onMount } from "svelte";

	type VirtualDeckState = {
		rows: number;
		columns: number;
		gap: number;
		mode: "floating" | "monitor";
		always_on_top: boolean;
		background_opacity: number;
		toolbar: "always" | "hover" | "never";
		sleeping: boolean;
		device: string;
	};

	// Keep in sync with TOOLBAR_HEIGHT in src-tauri/src/virtual_deck.rs.
	const TOOLBAR_HEIGHT = 28;

	let rows = 0;
	let columns = 0;
	let gap = 16;
	let floating = false;
	let alwaysOnTop = true;
	let backgroundOpacity = 0;
	let hovering = false;
	let toolbar: "always" | "hover" | "never" = "never";
	let sleeping = false;
	let device = "";
	let profile: Profile | null = null;
	let profileJson = "";
	/// Pressed state reported by the backend, used by the renderer.
	let moved: Record<number, boolean> = {};
	let pressed: Record<number, boolean> = {};
	let width = 0;
	let height = 0;
	let keyArea: HTMLDivElement;

	$: showToolbar = floating && toolbar != "never";
	$: keySize = rows && columns ? Math.max(0, Math.floor(Math.min((width - gap * (columns + 1)) / columns, (height - gap * (rows + 1)) / rows))) : 0;

	async function loadProfile() {
		if (!device) return;
		try {
			const loaded = await invoke<Profile>("get_selected_profile", { device });
			const json = JSON.stringify(loaded);
			if (json == profileJson) return;
			profileJson = json;
			if (profile?.id != loaded.id) moved = {};
			profile = loaded;
		} catch {
			// The device is not registered yet; the next poll or profile event retries.
		}
	}

	function updateSlot(context: string, contents: ActionInstance | null) {
		if (!profile) return;
		const index = profile.keys.findIndex((slot) => slot?.context == context);
		if (index == -1) return;
		profile.keys[index] = contents;
		profile = profile;
		profileJson = JSON.stringify(profile);
	}

	function applyState(state: VirtualDeckState) {
		rows = state.rows;
		columns = state.columns;
		gap = state.gap;
		floating = state.mode == "floating";
		alwaysOnTop = state.always_on_top;
		backgroundOpacity = state.background_opacity;
		toolbar = state.toolbar;
		sleeping = state.sleeping;
		device = state.device;
	}

	onMount(() => {
		// app.html gives every page a grey body; this window draws its own (optionally transparent) background.
		document.body.style.background = "transparent";

		const unlisteners = [
			listen<boolean>("virtual_deck_sleeping", ({ payload }) => (sleeping = payload)),
			listen<VirtualDeckState>("virtual_deck_state", ({ payload }) => applyState(payload)),
			listen("virtual_deck_profile", () => loadProfile()),
			listen<{ context: string; contents: ActionInstance | null }>("update_state", ({ payload }) => updateSlot(payload.context, payload.contents)),
			listen<{ context: Context; pressed: boolean }>("key_moved", ({ payload }) => {
				const { context } = payload;
				if (context.device != device || context.controller != "Keypad" || context.profile != profile?.id) return;
				moved[context.position] = payload.pressed;
				moved = moved;
			}),
			// Profile switches requested by plugins / actions go to the main window, which may be closed; apply them here
			// too (switching to the already selected profile is a no-op).
			listen<{ device: string; profile: string }>("switch_profile", ({ payload }) => {
				if (payload.device == device) invoke("set_selected_profile", { device, id: payload.profile }).catch(() => {});
			}),
		];

		invoke<VirtualDeckState>("virtual_deck_get_state").then((state) => {
			applyState(state);
			loadProfile();
		});
		// Plugin images are served by OpenDeck's webserver, whose port is only known after asking the backend.
		// Reload the profile once it is known so keys drawn with the default port are redrawn.
		initPortBase()
			.then(() => {
				profileJson = "";
				loadProfile();
			})
			.catch((error) => console.error(error));
		// Pick up edits made in the main window (adding, moving or removing actions).
		const poll = setInterval(loadProfile, 1500);
		// bind:client* uses a ResizeObserver, which WebKitGTK does not run for a window it considers hidden (e.g. never
		// focused). Measure on a timer as well so the keys always get a size.
		const measure = setInterval(() => {
			if (!keyArea) return;
			if (keyArea.clientWidth != width) width = keyArea.clientWidth;
			if (keyArea.clientHeight != height) height = keyArea.clientHeight;
		}, 500);

		return () => {
			clearInterval(poll);
			clearInterval(measure);
			unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
		};
	});

	async function press(position: number, down: boolean) {
		if (!!pressed[position] === down) return;
		pressed[position] = down;
		pressed = pressed;
		try {
			await invoke("virtual_deck_press", { position, pressed: down });
		} catch (error) {
			console.error(error);
		}
	}

	function pointerDown(event: PointerEvent, position: number) {
		if (event.button !== 0) return;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
		press(position, true);
	}

	function startDrag(event: PointerEvent) {
		if (event.button !== 0 || (event.target as HTMLElement).closest("button")) return;
		getCurrentWindow().startDragging();
	}

	function startResize(event: PointerEvent) {
		if (event.button !== 0) return;
		getCurrentWindow().startResizeDragging("SouthEast");
	}
</script>

<svelte:window on:contextmenu|preventDefault />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
	class="fixed inset-0 flex flex-col select-none overflow-hidden"
	style={`touch-action: none; background: rgb(0 0 0 / ${floating ? backgroundOpacity : 100}%);`}
	on:pointerenter={() => (hovering = true)}
	on:pointerleave={() => (hovering = false)}
>
	{#if showToolbar}
		<!-- svelte-ignore a11y-no-static-element-interactions -->
		<div
			class="flex flex-row items-center shrink-0 px-2 text-xs text-neutral-400 bg-neutral-950/80 rounded-md cursor-grab active:cursor-grabbing transition-opacity duration-150"
			class:opacity-0={toolbar == "hover" && !hovering}
			style={`height: ${TOOLBAR_HEIGHT}px;`}
			on:pointerdown={startDrag}
		>
			<span class="grow truncate">Virtual Stream Deck</span>
			<button
				class="p-1 rounded hover:bg-neutral-800 transition-colors"
				class:text-neutral-200={alwaysOnTop}
				title={alwaysOnTop ? "Stop keeping on top" : "Keep on top"}
				on:click={() => invoke("virtual_deck_set_always_on_top", { alwaysOnTop: !alwaysOnTop })}
			>
				<PushPin weight={alwaysOnTop ? "fill" : "regular"} />
			</button>
			<button
				class="p-1 rounded hover:bg-neutral-800 transition-colors"
				title="Hide (show again from the tray icon)"
				on:click={() => invoke("virtual_deck_set_visible", { visible: false })}
			>
				<EyeSlash />
			</button>
		</div>
	{/if}

	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<!-- Floating: grab the space between the keys to move the window. -->
	<!-- Measured with a ResizeObserver (bind:client*): the window size at load can be 0 before the webview is laid out. -->
	<div
		class="relative grow min-h-0 flex items-center justify-center"
		bind:this={keyArea}
		bind:clientWidth={width}
		bind:clientHeight={height}
		on:pointerdown={(event) => floating && startDrag(event)}
	>
		<div class="grid" style={`grid-template-columns: repeat(${columns}, ${keySize}px); gap: ${gap}px;`}>
			{#each { length: rows * columns } as _, i}
				<button
					class="relative overflow-hidden p-0 outline-none transition-[filter] duration-75"
					class:brightness-75={pressed[i]}
					style={`width: ${keySize}px; height: ${keySize}px; border-radius: ${Math.round(keySize * 0.12)}px; background: rgb(23 23 23 / 0.85); border: 1px solid rgb(255 255 255 / 0.08);`}
					tabindex="-1"
					aria-label={`Key ${i + 1}`}
					on:pointerdown={(event) => pointerDown(event, i)}
					on:pointerup={() => press(i, false)}
					on:pointercancel={() => press(i, false)}
					on:lostpointercapture={() => press(i, false)}
				>
					<VirtualDeckKey slot={profile?.keys[i] ?? null} pressed={!!moved[i]} />
				</button>
			{/each}
		</div>

		{#if sleeping}
			<!-- Asleep (see the sleep settings): dark until the next press, which only wakes the deck. -->
			<div class="absolute inset-0 bg-black/90 rounded-lg pointer-events-none"></div>
		{/if}

		{#if floating}
			<!-- svelte-ignore a11y-no-static-element-interactions -->
			<div
				class="absolute right-0 bottom-0 w-4 h-4 cursor-se-resize transition-opacity duration-150"
				class:opacity-0={toolbar != "always" && !hovering}
				on:pointerdown|stopPropagation={startResize}
			>
				<svg viewBox="0 0 16 16" class="w-full h-full text-neutral-600"><path d="M14 6 L6 14 M14 10 L10 14" stroke="currentColor" stroke-width="1.5" /></svg>
			</div>
		{/if}
	</div>
</div>
