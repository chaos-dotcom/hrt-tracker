<script lang="ts">
	import { hrtData } from '$lib/storage.svelte';

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
</script>

<div class="p-10 flex flex-col space-y-2 sm:space-y-10">
	<h1 class="text-4xl">backup & restore</h1>
	<button
		class="w-fit cursor-pointer rounded bg-latte-rose-pine-foam px-4 py-2 font-medium text-white transition-colors hover:bg-rose-pine-pine focus:outline-none focus:shadow-outline"
		onclick={exportToJson}
	>
		export to json
	</button>
</div>
