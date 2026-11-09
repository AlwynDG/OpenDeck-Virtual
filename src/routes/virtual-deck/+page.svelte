<script lang="ts">
	// Virtual Stream Deck window: shows only the key grid and forwards presses to the backend.
	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";

	type VirtualDeckState = { rows: number; columns: number; gap: number; images: Record<string, string> };

	let rows = 0;
	let columns = 0;
	let gap = 16;
	let images: Record<number, string> = {};
	let pressed: Record<number, boolean> = {};
	let width = 0;
	let height = 0;

	$: keySize = rows && columns ? Math.max(0, Math.floor(Math.min((width - gap * (columns + 1)) / columns, (height - gap * (rows + 1)) / rows))) : 0;

	onMount(() => {
		const unlisteners = [
			listen<{ position: number; image: string | null }>("virtual_deck_image", ({ payload }) => {
				if (payload.image) images[payload.position] = payload.image;
				else delete images[payload.position];
				images = images;
			}),
			listen("virtual_deck_clear", () => (images = {})),
		];

		invoke<VirtualDeckState>("virtual_deck_get_state").then((state) => {
			rows = state.rows;
			columns = state.columns;
			gap = state.gap;
			images = { ...Object.fromEntries(Object.entries(state.images).map(([k, v]) => [Number(k), v])), ...images };
		});

		return () => unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
	});

	async function press(position: number, down: boolean) {
		if (!!pressed[position] === down) return;
		pressed[position] = down;
		pressed = pressed;
		await invoke("virtual_deck_press", { position, pressed: down });
	}

	function pointerDown(event: PointerEvent, position: number) {
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
		press(position, true);
	}
</script>

<svelte:window bind:innerWidth={width} bind:innerHeight={height} on:contextmenu|preventDefault />

<div class="fixed inset-0 bg-black flex items-center justify-center select-none overflow-hidden" style="touch-action: none;">
	<div class="grid" style={`grid-template-columns: repeat(${columns}, ${keySize}px); gap: ${gap}px;`}>
		{#each { length: rows * columns } as _, i}
			<button
				class="relative overflow-hidden bg-neutral-900 p-0 border-0 outline-none transition-transform duration-75"
				class:scale-90={pressed[i]}
				style={`width: ${keySize}px; height: ${keySize}px; border-radius: ${Math.round(keySize * 0.12)}px;`}
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
</div>
