<script lang="ts">
	import { hrtData } from '$lib/storage.svelte';

	// Ensure settings object exists
	if (!hrtData.data.settings) {
		hrtData.data.settings = {
			enableAutoBackfill: true,
		};
	}
	const s = hrtData.data.settings;

	let saveMessage = $state('');
	async function saveAll() {
		const ok = await hrtData.saveNow();
		saveMessage = ok ? 'Saved settings and data' : 'Failed to save';
		setTimeout(() => (saveMessage = ''), 3000);
	}

	// Auto-advance can be triggered when saving; removed live effect to avoid constant file writes.

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
 
	// ICS calendar subscription URL (defaults to 1-year horizon, includes past entries)
	let icsUrl = $state('');
	$effect(() => {
		s.icsSecret;
		if (typeof window !== 'undefined') {
			icsUrl = s.icsSecret && s.icsSecret.trim().length > 0
				? `${location.origin}/api/ics/${encodeURIComponent(s.icsSecret.trim())}?horizonDays=365&includePast=1`
				: `${location.origin}/api/ics?horizonDays=365&includePast=1`;
		}
	});
	function copyIcsUrl() {
		navigator.clipboard?.writeText(icsUrl);
	}
 
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
			<span>Enable automatic schedule filling</span>
		</label>
		<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
			<label class="block">
				<span class="text-sm">ICS URL secret (optional)</span>
				<input type="text" placeholder="e.g. my-private-feed" class="border rounded px-2 py-1 w-full" bind:value={s.icsSecret} />
				<p class="text-xs opacity-75 mt-1">When set, your ICS URL becomes /api/ics/&#123;secret&#125;. Keep it hard to guess.</p>
			</label>
		</div>
		<div class="flex items-center gap-3">
			<button
				class="w-fit cursor-pointer rounded bg-latte-rose-pine-foam px-4 py-2 font-medium text-white transition-colors hover:bg-rose-pine-pine focus:outline-none focus:shadow-outline"
				onclick={saveAll}
			>
				Save
			</button>
			{#if saveMessage}
				<p class="text-sm text-latte-rose-pine-text dark:text-rose-pine-text">{saveMessage}</p>
			{/if}
		</div>
		<p class="text-sm text-gray-500">Click Save to persist settings and data to the server.</p>
	</div>

	<!-- ICS Calendar Section -->
	<div class="p-4 border rounded space-y-3 bg-white dark:bg-rose-pine-surface">
		<h2 class="text-2xl mb-2">ICS Calendar</h2>
		<p class="text-sm opacity-75 mb-2">
			Subscribe in your calendar app using this URL. It includes your recorded doses and future scheduled doses.
		</p>
		<div class="flex items-center gap-2">
			<input class="flex-1 border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text" type="text" readonly value={icsUrl} />
			<a class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors" href={icsUrl} target="_blank" rel="noopener noreferrer">Open</a>
			<button class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay" onclick={copyIcsUrl}>Copy</button>
		</div>
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
