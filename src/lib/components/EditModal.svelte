<script lang="ts">
	import {
		HormoneUnits,
		type BloodTest,
		type DosageHistoryEntry
	} from '$lib/types';

	let { item, close }: { item: BloodTest | DosageHistoryEntry; close: () => void } = $props();

	const isDosage = 'medicationType' in item;

	// Common fields
	let date = $state(new Date(item.date).toISOString().slice(0, 16));

	// BloodTest fields
	let estradiolLevel = $state(isDosage ? undefined : (item as BloodTest).estradiolLevel);
	let testLevel = $state(isDosage ? undefined : (item as BloodTest).testLevel);
	let estradiolUnit = $state(
		isDosage ? undefined : (item as BloodTest).estradiolUnit || HormoneUnits.E2_pg_mL
	);
	let testUnit = $state(
		isDosage ? undefined : (item as BloodTest).testUnit || HormoneUnits.T_ng_dL
	);
	let notes = $state(isDosage ? undefined : (item as BloodTest).notes);

	// DosageHistoryEntry fields
	let dose = $state(isDosage ? (item as DosageHistoryEntry).dose : undefined);
	let unit = $state(isDosage ? (item as DosageHistoryEntry).unit : undefined);

	function enumToDropdownOptions(e: any) {
		return Object.entries(e).map(([, val]) => ({
			value: val as string,
			label: val as string
		}));
	}
	const unitOptions = enumToDropdownOptions(HormoneUnits);

	function save() {
		item.date = new Date(date).getTime();

		if (isDosage) {
			const dosageItem = item as DosageHistoryEntry;
			dosageItem.dose = dose!;
			dosageItem.unit = unit!;
		} else {
			const bloodTestItem = item as BloodTest;
			bloodTestItem.estradiolLevel = estradiolLevel;
			bloodTestItem.testLevel = testLevel;
			bloodTestItem.estradiolUnit = estradiolUnit;
			bloodTestItem.testUnit = testUnit;
			bloodTestItem.notes = notes;
		}

		close();
	}
</script>

<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
	onclick={close}
>
	<div
		class="bg-latte-rose-pine-base dark:bg-rose-pine-base max-w-md w-full rounded-lg p-6 shadow-xl"
		onclick={(e) => e.stopPropagation()}
	>
		<h2 class="mb-4 text-2xl font-bold">Edit Entry</h2>

		<div class="mb-4">
			<label for="edit-date" class="mb-2 block text-sm font-medium">Date / Time</label>
			<input
				id="edit-date"
				type="datetime-local"
				bind:value={date}
				class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
			/>
		</div>

		{#if isDosage}
			{@const dosageItem = item as DosageHistoryEntry}
			<div class="mb-4">
				<p><strong>Medication:</strong> {dosageItem.type}</p>
			</div>
			<div class="flex gap-5">
				<div class="w-full">
					<label for="dose" class="block text-sm mb-1">Dose</label>
					<input
						id="dose"
						type="number"
						bind:value={dose}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="unit" class="block text-sm mb-1">Unit</label>
					<select
						id="unit"
						bind:value={unit}
						class="border py-2 px-3 rounded w-full leading-tight"
					>
						{#each unitOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>
			</div>
		{:else}
			<div class="flex gap-5">
				<div class="w-full">
					<label for="eLevel" class="block text-sm mb-1">Estradiol Level</label>
					<input
						id="eLevel"
						type="number"
						bind:value={estradiolLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="eUnit" class="block text-sm mb-1">Estradiol Unit</label>
					<select
						id="eUnit"
						bind:value={estradiolUnit}
						class="border py-2 px-3 rounded w-full leading-tight"
					>
						{#each unitOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="flex gap-5 mt-4">
				<div class="w-full">
					<label for="testLevel" class="block text-sm mb-1">Testosterone Level</label>
					<input
						id="testLevel"
						type="number"
						bind:value={testLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="testUnit" class="block text-sm mb-1">Testosterone Unit</label>
					<select
						id="testUnit"
						bind:value={testUnit}
						class="border py-2 px-3 rounded w-full leading-tight"
					>
						{#each unitOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>
			</div>
			<div class="mt-4">
				<textarea
					bind:value={notes}
					placeholder="Notes..."
					class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
				></textarea>
			</div>
		{/if}

		<div class="flex justify-end gap-4 mt-6">
			<button
				class="px-4 py-2 rounded transition-colors bg-gray-200 dark:bg-gray-600 hover:bg-gray-300 dark:hover:bg-gray-500"
				onclick={close}>Cancel</button
			>
			<button
				class="px-4 py-2 rounded transition-colors bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine"
				onclick={save}>Save Changes</button
			>
		</div>
	</div>
</div>
