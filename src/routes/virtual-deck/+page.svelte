<script lang="ts">
	// Virtual Stream Deck window: shows only the key grid and forwards presses to the backend.
	// In floating mode it also has a small drag bar (pin / hide) and a resize grip, like Elgato's Virtual Stream Deck.
	import EyeSlash from "phosphor-svelte/lib/EyeSlash";
	import PushPin from "phosphor-svelte/lib/PushPin";

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
		images: Record<string, string>;
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
	let images: Record<number, string> = {};
	let pressed: Record<number, boolean> = {};
	let width = 0;
	let height = 0;

	$: showToolbar = floating && toolbar != "never";
	$: gridHeight = height - (showToolbar ? TOOLBAR_HEIGHT : 0);
	$: keySize = rows && columns ? Math.max(0, Math.floor(Math.min((width - gap * (columns + 1)) / columns, (gridHeight - gap * (rows + 1)) / rows))) : 0;

	function toImages(record: Record<string, string>): Record<number, string> {
		return Object.fromEntries(Object.entries(record).map(([k, v]) => [Number(k), v]));
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
	}

	onMount(() => {
		// app.html gives every page a grey body; this window draws its own (optionally transparent) background.
		document.body.style.background = "transparent";

		const unlisteners = [
			listen<{ position: number; image: string | null }>("virtual_deck_image", ({ payload }) => {
				if (payload.image) images[payload.position] = payload.image;
				else delete images[payload.position];
				images = images;
			}),
			listen<Record<string, string>>("virtual_deck_images", ({ payload }) => (images = toImages(payload))),
			listen<boolean>("virtual_deck_sleeping", ({ payload }) => (sleeping = payload)),
			listen<VirtualDeckState>("virtual_deck_state", ({ payload }) => applyState(payload)),
		];

		invoke<VirtualDeckState>("virtual_deck_get_state").then((state) => {
			applyState(state);
			// Images that arrived while we were loading are newer than the snapshot.
			images = { ...toImages(state.images), ...images };
		});

		return () => unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
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

<svelte:window bind:innerWidth={width} bind:innerHeight={height} on:contextmenu|preventDefault />

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
	class="fixed inset-0 flex flex-col select-none overflow-hidden"
	style={`touch-action: none; background: rgb(0 0 0 / ${backgroundOpacity}%);`}
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
	<div class="relative grow flex items-center justify-center" on:pointerdown={(event) => floating && startDrag(event)}>
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
					{#if images[i]}
						<img src={images[i]} alt="" draggable="false" class="w-full h-full pointer-events-none" />
					{/if}
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
