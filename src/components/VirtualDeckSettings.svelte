<script lang="ts">
	// Settings for the virtual Stream Deck (OpenDeck-Virtual fork). Kept in its own component so SettingsView.svelte
	// stays close to upstream.
	import { invoke } from "@tauri-apps/api/core";
	import { ask } from "@tauri-apps/plugin-dialog";
	import { onMount } from "svelte";

	type Geometry = { x: number; y: number; width: number; height: number };
	type VirtualDeckConfig = {
		enabled: boolean;
		mode: "floating" | "monitor";
		rows: number;
		columns: number;
		gap: number;
		monitor: string | null;
		always_on_top: boolean;
		floating: Geometry | null;
		visible: boolean;
		background_opacity: number;
		toolbar: "always" | "hover" | "never";
		start_in_tray: boolean;
	};
	type MonitorInfo = { name: string; width: number; height: number; x: number; y: number };

	let config: VirtualDeckConfig | null = null;
	let saved = "";
	let monitors: MonitorInfo[] = [];
	let error = "";

	$: dirty = config != null && JSON.stringify(config) != saved;

	async function load() {
		config = await invoke<VirtualDeckConfig>("virtual_deck_get_config");
		saved = JSON.stringify(config);
		monitors = await invoke<MonitorInfo[]>("virtual_deck_list_monitors");
	}

	async function apply() {
		if (!config) return;
		error = "";
		const old = JSON.parse(saved) as VirtualDeckConfig;
		if (config.rows * config.columns < old.rows * old.columns) {
			const removed = old.rows * old.columns - config.rows * config.columns;
			const confirmed = await ask(`The deck gets ${removed} fewer keys. Actions on keys that no longer fit are removed from your profiles. Continue?`, {
				title: "Fewer keys",
				kind: "warning",
			});
			if (!confirmed) return;
		}
		try {
			await invoke("virtual_deck_set_config", { newConfig: config });
			await load();
		} catch (e) {
			error = String((e as { description?: string })?.description ?? e);
		}
	}

	onMount(load);
</script>

{#if config}
	<div class="mx-2 mt-4 mb-2 p-2 border border-neutral-700 rounded-lg">
		<h3 class="mb-1 font-semibold text-neutral-300">Virtual Stream Deck</h3>

		<div class="flex flex-row items-center my-2 space-x-2">
			<label for="vd-enabled" class="text-neutral-400">Enabled</label>
			<input type="checkbox" bind:checked={config.enabled} id="vd-enabled" />
		</div>

		{#if config.enabled}
			<div class="flex flex-row items-center my-2 space-x-2">
				<label for="vd-mode" class="text-neutral-400">Window</label>
				<div class="select-wrapper">
					<select bind:value={config.mode} class="w-auto pr-10!" id="vd-mode">
						<option value="floating">Floating (drag and resize anywhere)</option>
						<option value="monitor">Fill a whole monitor</option>
					</select>
				</div>
			</div>

			{#if config.mode == "monitor"}
				<div class="flex flex-row items-center my-2 space-x-2">
					<label for="vd-monitor" class="text-neutral-400">Monitor</label>
					<div class="select-wrapper">
						<select bind:value={config.monitor} class="w-auto pr-10!" id="vd-monitor">
							<option value={null}>Automatic (rightmost portrait monitor)</option>
							{#each monitors as monitor}
								<option value={monitor.name}>{monitor.name} — {monitor.width}×{monitor.height}</option>
							{/each}
						</select>
					</div>
				</div>
			{/if}

			<div class="flex flex-row items-center my-2 space-x-2">
				<label for="vd-columns" class="text-neutral-400">Keys</label>
				<input
					type="number"
					min="1"
					max="16"
					bind:value={config.columns}
					class="w-12 px-1 text-neutral-300 border border-neutral-600 rounded-lg"
					id="vd-columns"
				/>
				<span class="text-neutral-400">wide ×</span>
				<input type="number" min="1" max="16" bind:value={config.rows} class="w-12 px-1 text-neutral-300 border border-neutral-600 rounded-lg" id="vd-rows" />
				<span class="text-neutral-400">high</span>
			</div>

			<div class="flex flex-row items-center my-2 space-x-2">
				<label for="vd-gap" class="text-neutral-400">Gap between keys</label>
				<input type="range" min="0" max="64" bind:value={config.gap} id="vd-gap" />
				<span class="text-neutral-400 w-10">{config.gap}px</span>
			</div>

			{#if config.mode == "floating"}
				<!-- Floating only: a transparent window still catches clicks, so a transparent full-monitor deck would block the whole monitor. -->
				<div class="flex flex-row items-center my-2 space-x-2">
					<label for="vd-background_opacity" class="text-neutral-400">Background</label>
					<input type="range" min="0" max="100" bind:value={config.background_opacity} id="vd-background_opacity" />
					<span class="text-neutral-400 w-24">{config.background_opacity == 0 ? "transparent" : `${config.background_opacity}%`}</span>
				</div>

				<div class="flex flex-row items-center my-2 space-x-2">
					<label for="vd-toolbar" class="text-neutral-400">Drag bar</label>
					<div class="select-wrapper">
						<select bind:value={config.toolbar} class="w-auto pr-10!" id="vd-toolbar">
							<option value="never">Never (just the keys)</option>
							<option value="hover">When the mouse is over the deck</option>
							<option value="always">Always</option>
						</select>
					</div>
				</div>
				<p class="-mt-1 mb-2 text-xs text-neutral-500">Move the deck by dragging the space between the keys; resize it from the bottom-right corner.</p>
			{/if}

			<div class="flex flex-row items-center my-2 space-x-2">
				<label for="vd-start_in_tray" class="text-neutral-400">Start OpenDeck in the tray</label>
				<input type="checkbox" bind:checked={config.start_in_tray} id="vd-start_in_tray" />
				<span class="text-xs text-neutral-500">only the deck appears; open this window from the tray icon</span>
			</div>

			<div class="flex flex-row items-center my-2 space-x-2">
				<label for="vd-always_on_top" class="text-neutral-400">Always on top</label>
				<input type="checkbox" bind:checked={config.always_on_top} id="vd-always_on_top" />
			</div>
		{/if}

		<div class="flex flex-row items-center mt-3 space-x-2">
			<button
				class="px-2 py-1 text-sm text-neutral-300 bg-neutral-700 hover:bg-neutral-600 disabled:opacity-50 transition-colors border border-neutral-600 rounded-lg"
				disabled={!dirty}
				on:click={apply}
			>
				Apply
			</button>
			{#if config.enabled && !config.visible}
				<button
					class="px-2 py-1 text-sm text-neutral-300 bg-neutral-700 hover:bg-neutral-600 transition-colors border border-neutral-600 rounded-lg"
					on:click={async () => {
						await invoke("virtual_deck_set_visible", { visible: true });
						await load();
					}}
				>
					Show deck
				</button>
			{/if}
			{#if error}<span class="text-sm text-red-400">{error}</span>{/if}
		</div>
		<p class="mt-2 text-xs text-neutral-500">
			Careful: actions are stored in reading order, so changing the number of keys moves them around, and actions on keys that no longer fit are removed.
		</p>
	</div>
{/if}
