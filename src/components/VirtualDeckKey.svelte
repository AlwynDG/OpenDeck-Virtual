<script lang="ts">
	// One key of the virtual deck window. Renders the action's current state with the same renderer the main window
	// uses for hardware, so the deck does not depend on the main window being open.
	import type { ActionInstance } from "$lib/ActionInstance";

	import { CanvasLock, renderImage } from "$lib/rendererHelper";

	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";

	export let slot: ActionInstance | null;
	/// Pressed as reported by the backend (`key_moved`), which the renderer shows by shrinking the image.
	export let pressed = false;

	// Render at a fixed resolution and let CSS scale it, like a hardware key.
	const RESOLUTION = 288;

	let canvas: HTMLCanvasElement;
	let showOk = false;
	let showAlert = false;
	let timeout: number | undefined;
	const lock = new CanvasLock();

	onMount(() => {
		const flash = (alert: boolean) => {
			clearTimeout(timeout);
			showAlert = alert;
			showOk = !alert;
			timeout = setTimeout(() => (showAlert = showOk = false), 1.5e3);
		};
		const unlisteners = [
			listen<string>("show_alert", ({ payload }) => slot && payload == slot.context && flash(true)),
			listen<string>("show_ok", ({ payload }) => slot && payload == slot.context && flash(false)),
		];
		return () => {
			clearTimeout(timeout);
			unlisteners.forEach((p) => p.then((unlisten) => unlisten()));
		};
	});

	async function draw(slot: ActionInstance | null, pressed: boolean, showOk: boolean, showAlert: boolean) {
		if (!canvas) return;
		const unlock = await lock.lock();
		try {
			const state = slot?.states[slot.current_state];
			if (!slot || !state) {
				canvas.getContext("2d")?.clearRect(0, 0, canvas.width, canvas.height);
				return;
			}
			const fallback = slot.action.states[slot.current_state]?.image ?? slot.action.icon;
			// `active: false`: draw only, never send the image to a device (the main window does that for hardware).
			await renderImage(canvas, null, state, fallback, showOk, showAlert, true, false, pressed);
		} finally {
			unlock();
		}
	}

	$: (draw(slot, pressed, showOk, showAlert), canvas);
</script>

<canvas bind:this={canvas} width={RESOLUTION} height={RESOLUTION} class="w-full h-full pointer-events-none"></canvas>
