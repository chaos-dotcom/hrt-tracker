<script lang="ts">
	import { hrtData } from '$lib/storage.svelte';

	// Ensure settings object exists
	if (!hrtData.data.settings) {
		hrtData.data.settings = {
			enableAutoBackfill: true,
			defaultInjectionFrequencyDays: 7,
			defaultOralFrequencyDays: 1,
			defaultAntiandrogenFrequencyDays: 1,
			defaultProgesteroneFrequencyDays: 1,
		};
	}
	const s = hrtData.data.settings;

	$effect(() => {
		s.enableAutoBackfill;
		if (s.enableAutoBackfill) {
			hrtData.backfillScheduledDoses();
		}
	});

	function exportToJson() {
		const dataStr = JSON.stringify(hrtData.data, null, 2);
		const blob = new Blob([dataStr], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `hrt-data-backup-${new Date().toISOString().split('T')[0]}.json`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
		URL.revokeObjectURL(url);
	}

	let restoreMessage = $state('');

	async function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			const file = input.files[0];
			const reader = new FileReader();
			reader.onload = async (e) => {
				try {
					const text = e.target?.result;
					if (typeof text === 'string') {
						const jsonData = JSON.parse(text);
						// A simple validation to check for expected keys
						if (jsonData.bloodTests && jsonData.dosageHistory) {
							const response = await fetch('/api/data', {
								method: 'POST',
								headers: {
									'Content-Type': 'application/json',
								},
								body: JSON.stringify(jsonData),
							});

							if (response.ok) {
								hrtData.data = { ...hrtData.data, ...jsonData, notes: jsonData.notes ?? [] };
								restoreMessage = 'Data restored successfully!';
							} else {
								restoreMessage = 'Failed to restore data on the server.';
							}
						} else {
							restoreMessage = 'Invalid JSON file format.';
						}
					}
				} catch (error) {
					restoreMessage = 'Error reading or parsing file.';
					console.error(error);
				}
				setTimeout(() => (restoreMessage = ''), 3000);
			};
			reader.readAsText(file);
		}
	}
</script>

<div class="p-10 flex flex-col space-y-2 sm:space-y-10">
	<h1 class="text-4xl">backup & restore</h1>

	<!-- Settings Section -->
	<div class="p-4 border rounded space-y-3 bg-white dark:bg-rose-pine-surface">
		<h2 class="text-2xl mb-2">Settings</h2>
		<label class="flex items-center gap-2">
			<input type="checkbox" bind:checked={s.enableAutoBackfill} />
			<span>Enable automatic backfill (auto-schedule)</span>
		</label>
		<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
			<label class="block">
				<span class="text-sm">Default Injectable Estradiol Frequency (days)</span>
				<input type="number" min="1" class="border rounded px-2 py-1 w-full" bind:value={s.defaultInjectionFrequencyDays} />
			</label>
			<label class="block">
				<span class="text-sm">Default Oral Estradiol Frequency (days)</span>
				<input type="number" min="1" class="border rounded px-2 py-1 w-full" bind:value={s.defaultOralFrequencyDays} />
			</label>
			<label class="block">
				<span class="text-sm">Default Antiandrogen Frequency (days)</span>
				<input type="number" min="1" class="border rounded px-2 py-1 w-full" bind:value={s.defaultAntiandrogenFrequencyDays} />
			</label>
			<label class="block">
				<span class="text-sm">Default Progesterone Frequency (days)</span>
				<input type="number" min="1" class="border rounded px-2 py-1 w-full" bind:value={s.defaultProgesteroneFrequencyDays} />
			</label>
		</div>
		<p class="text-sm text-gray-500">Changes auto-save and are stored server-side as YAML.</p>
	</div>

	<div class="flex flex-col space-y-4">
		<div>
			<h2 class="text-2xl mb-2">Export Data</h2>
			<button
				class="w-fit cursor-pointer rounded bg-latte-rose-pine-foam px-4 py-2 font-medium text-white transition-colors hover:bg-rose-pine-pine focus:outline-none focus:shadow-outline"
				onclick={exportToJson}
			>
				export to json
			</button>
		</div>

		<div class="pt-6">
			<h2 class="text-2xl mb-2">Restore Data</h2>
			<p class="text-sm opacity-75 mb-4">
				Select a JSON backup file to restore your data. This will overwrite any existing data.
			</p>
			<input
				type="file"
				accept=".json"
				onchange={handleFileSelect}
				class="block w-full max-w-xs text-sm
            file:mr-4 file:py-2 file:px-4
            file:rounded-full file:border-0
            file:text-sm file:font-semibold
            file:bg-latte-rose-pine-foam file:text-white
            hover:file:bg-rose-pine-pine"
			/>
			{#if restoreMessage}
				<p class="mt-4 text-latte-rose-pine-text dark:text-rose-pine-text">
					{restoreMessage}
				</p>
			{/if}
		</div>
	</div>
</div>
