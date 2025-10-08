<script lang="ts">
	import { browser } from '$app/environment';
	import { hrtData } from '$lib/storage.svelte';

	let { children, data } = $props();
	import '../app.css';

	if (browser) {
		hrtData.init(data.initialHrtData);
	}
</script>

<header>
	<nav class="bg-latte-rose-pine-surface dark:bg-rose-pine-surface p-4 shadow-md">
		<div class="container mx-auto flex items-center justify-between">
			<a
				href="/"
				class="text-lg font-bold text-latte-rose-pine-text transition-colors hover:text-rose-pine-love dark:text-rose-pine-text"
				>HRT Tracker</a
			>
			<div class="space-x-4">
				<a
					href="/view"
					class="text-latte-rose-pine-iris transition-colors hover:text-rose-pine-love dark:text-rose-pine-iris"
					>View Data</a
				>
				<a
					href="/create/blood-test"
					class="text-latte-rose-pine-iris transition-colors hover:text-rose-pine-love dark:text-rose-pine-iris"
					>Add Blood Test</a
				>
				<a
					href="/create/dosage"
					class="text-latte-rose-pine-iris transition-colors hover:text-rose-pine-love dark:text-rose-pine-iris"
					>Add Dosage</a
				>
				<a
					href="/backup"
					class="text-latte-rose-pine-iris transition-colors hover:text-rose-pine-love dark:text-rose-pine-iris"
					>Backup/Restore</a
				>
			</div>
		</div>
	</nav>
</header>

<main class="container mx-auto">
	{@render children()}
</main>
