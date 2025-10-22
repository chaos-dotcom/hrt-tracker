<script lang="ts">
	import { hrtData } from '$lib/storage.svelte';
	import {
		HormoneUnits,
		type BloodTest,
		type DosageHistoryEntry,
		ProgesteroneRoutes,
		type Measurement,
		WeightUnit,
		LengthUnit,
		InjectionSites
	} from '$lib/types';

	let { item, close }: { item: BloodTest | DosageHistoryEntry | Measurement; close: () => void } = $props();

	const isDosage = 'medicationType' in item;
	const isMeasurement = 'weight' in item || 'braSize' in item;

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
	let progesteroneLevel = $state(isDosage ? undefined : (item as BloodTest).progesteroneLevel);
	let progesteroneUnit = $state(
		isDosage ? undefined : (item as BloodTest).progesteroneUnit || HormoneUnits.ng_mL
	);
	let fshLevel = $state(isDosage ? undefined : (item as BloodTest).fshLevel);
	let fshUnit = $state(isDosage ? undefined : (item as BloodTest).fshUnit || HormoneUnits.mIU_mL);
	let lhLevel = $state(isDosage ? undefined : (item as BloodTest).lhLevel);
	let lhUnit = $state(isDosage ? undefined : (item as BloodTest).lhUnit || HormoneUnits.mIU_mL);
	let prolactinLevel = $state(isDosage ? undefined : (item as BloodTest).prolactinLevel);
	let prolactinUnit = $state(
		isDosage ? undefined : (item as BloodTest).prolactinUnit || HormoneUnits.ng_mL
	);
	let shbgLevel = $state(isDosage ? undefined : (item as BloodTest).shbgLevel);
	let shbgUnit = $state(
		isDosage ? undefined : (item as BloodTest).shbgUnit || HormoneUnits.T_nmol_L
	);
	let freeAndrogenIndex = $state(isDosage ? undefined : (item as BloodTest).freeAndrogenIndex);
	let estrannaiseNumber = $state(isDosage ? undefined : (item as BloodTest).estrannaiseNumber);
	let notes = $state(isDosage ? undefined : (item as BloodTest).notes);

	// DosageHistoryEntry fields
	let dose = $state(isDosage ? (item as DosageHistoryEntry).dose : undefined);
	let unit = $state(isDosage ? (item as DosageHistoryEntry).unit : undefined);
	let pRoute = $state(
		isDosage && (item as DosageHistoryEntry).medicationType === 'progesterone'
			? (item as any).route
			: undefined
	);
	let note = $state(isDosage ? (item as DosageHistoryEntry).note : undefined);
	let injectionSite = $state(
		isDosage && (item as DosageHistoryEntry).medicationType === 'injectableEstradiol'
			? (item as any).injectionSite
			: undefined
	);

	// Measurement fields
	let weight = $state(isMeasurement ? (item as Measurement).weight : undefined);
	let weightUnit = $state(isMeasurement ? (item as Measurement).weightUnit || WeightUnit.KG : undefined);
	let height = $state(isMeasurement ? (item as Measurement).height : undefined);
	let heightUnit = $state(isMeasurement ? (item as Measurement).heightUnit || LengthUnit.CM : undefined);
	let underbust = $state(isMeasurement ? (item as Measurement).underbust : undefined);
	let bust = $state(isMeasurement ? (item as Measurement).bust : undefined);
	let bideltoid = $state(isMeasurement ? (item as Measurement).bideltoid : undefined);
	let waist = $state(isMeasurement ? (item as Measurement).waist : undefined);
	let hip = $state(isMeasurement ? (item as Measurement).hip : undefined);
	let bodyMeasurementUnit = $state(isMeasurement ? (item as Measurement).bodyMeasurementUnit || LengthUnit.CM : undefined);
	let braSize = $state(isMeasurement ? (item as Measurement).braSize : undefined);

	function enumToDropdownOptions(e: any) {
		return Object.entries(e).map(([, val]) => ({
			value: val as string,
			label: val as string
		}));
	}
	const unitOptions = enumToDropdownOptions(HormoneUnits);
	const progesteroneRouteOptions = enumToDropdownOptions(ProgesteroneRoutes);
	const weightUnitOptions = enumToDropdownOptions(WeightUnit);
	const lengthUnitOptions = enumToDropdownOptions(LengthUnit);
	const injectionSiteOptions = enumToDropdownOptions(InjectionSites);

	function save() {
		item.date = new Date(date).getTime();

		if (isDosage) {
			const dosageItem = item as DosageHistoryEntry;
			dosageItem.dose = dose!;
			dosageItem.unit = unit!;
			dosageItem.note = note?.trim() || undefined;
			if (dosageItem.medicationType === 'progesterone') {
				(dosageItem as any).route = pRoute;
			}
			if (dosageItem.medicationType === 'injectableEstradiol') {
				(dosageItem as any).injectionSite = injectionSite || undefined;
			}
		} else if (isMeasurement) {
			const measurementItem = item as Measurement;
			measurementItem.weight = weight;
			measurementItem.weightUnit = weightUnit;
			measurementItem.height = height;
			measurementItem.heightUnit = heightUnit;
			measurementItem.underbust = underbust;
			measurementItem.bust = bust;
			measurementItem.bideltoid = bideltoid;
			measurementItem.waist = waist;
			measurementItem.hip = hip;
			measurementItem.bodyMeasurementUnit = bodyMeasurementUnit;
			measurementItem.braSize = braSize;
		} else {
			const bloodTestItem = item as BloodTest;
			bloodTestItem.estradiolLevel = estradiolLevel;
			bloodTestItem.testLevel = testLevel;
			bloodTestItem.estradiolUnit = estradiolUnit;
			bloodTestItem.testUnit = testUnit;
			bloodTestItem.progesteroneLevel = progesteroneLevel;
			bloodTestItem.progesteroneUnit = progesteroneUnit;
			bloodTestItem.fshLevel = fshLevel;
			bloodTestItem.fshUnit = fshUnit;
			bloodTestItem.lhLevel = lhLevel;
			bloodTestItem.lhUnit = lhUnit;
			bloodTestItem.prolactinLevel = prolactinLevel;
			bloodTestItem.prolactinUnit = prolactinUnit;
			bloodTestItem.shbgLevel = shbgLevel;
			bloodTestItem.shbgUnit = shbgUnit;
			bloodTestItem.freeAndrogenIndex = freeAndrogenIndex;
			bloodTestItem.estrannaiseNumber = estrannaiseNumber;
			bloodTestItem.notes = notes;
		}

		close();
	}

	function deleteEntry() {
		if (confirm('Are you sure you want to delete this entry?')) {
			if (isDosage) {
				hrtData.deleteDosageRecord(item as DosageHistoryEntry);
			} else if (isMeasurement) {
				hrtData.deleteMeasurement(item as Measurement);
			} else {
				hrtData.deleteBloodTest(item as BloodTest);
			}
			close();
		}
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
			{#if dosageItem.medicationType === 'progesterone'}
				<div class="flex gap-5">
					<div class="w-1/3">
						<label for="dose" class="block text-sm mb-1">Dose</label>
						<input
							id="dose"
							type="number"
							step="any"
							bind:value={dose}
							class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
						/>
					</div>
					<div class="w-1/3">
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
					<div class="w-1/3">
						<label for="pRoute" class="block text-sm mb-1">Route</label>
						<select
							id="pRoute"
							bind:value={pRoute}
							class="border py-2 px-3 rounded w-full leading-tight"
						>
							{#each progesteroneRouteOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
					</div>
				</div>
			{:else}
				<div class="flex gap-5">
					<div class="w-full">
						<label for="dose" class="block text-sm mb-1">Dose</label>
						<input
							id="dose"
							type="number"
							step="any"
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
			{/if}
			{#if isDosage}
				<div class="mb-4">
					<label for="note" class="block text-sm mb-1">Note (optional)</label>
					<textarea
						id="note"
						bind:value={note}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
						rows="3"
						placeholder="Add any notes about this dose"
					></textarea>
				</div>
				{#if (item as DosageHistoryEntry).medicationType === 'injectableEstradiol'}
				<div class="mb-4">
					<label for="injectionSite" class="block text-sm mb-1">Injection Site (optional)</label>
					<select
						id="injectionSite"
						bind:value={injectionSite}
						class="border py-2 px-3 rounded w-full leading-tight"
					>
						<option value="">Select injection site</option>
						{#each injectionSiteOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>
				{/if}
			{/if}
		{:else if isMeasurement}
			<div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
				<div>
					<label class="block text-sm font-medium mb-2" for="weight">Weight</label>
					<div class="flex gap-2">
						<input id="weight" type="number" step="any" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={weight} />
						<select class="border py-2 px-3 rounded leading-tight" bind:value={weightUnit}>
							{#each weightUnitOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
					</div>
				</div>
				<div>
					<label class="block text-sm font-medium mb-2" for="height">Height</label>
					<div class="flex gap-2">
						<input id="height" type="number" step="any" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={height} />
						<select class="border py-2 px-3 rounded leading-tight" bind:value={heightUnit}>
							{#each lengthUnitOptions as option}
								<option value={option.value}>{option.label}</option>
							{/each}
						</select>
					</div>
				</div>
			</div>
			<div class="mb-4">
				<label class="block text-sm font-medium mb-2">Body Measurements</label>
				<div class="flex justify-end mb-2">
					<select class="border py-1 px-2 rounded text-sm" bind:value={bodyMeasurementUnit}>
						{#each lengthUnitOptions as option}
							<option value={option.value}>{option.label}</option>
						{/each}
					</select>
				</div>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
					<input type="number" step="any" placeholder="Underbust" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={underbust} />
					<input type="number" step="any" placeholder="Bust" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={bust} />
					<input type="number" step="any" placeholder="Bideltoid (shoulder)" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={bideltoid} />
					<input type="number" step="any" placeholder="Waist" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={waist} />
					<input type="number" step="any" placeholder="Hip" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={hip} />
				</div>
			</div>
			<div class="mb-4">
				<label class="block text-sm font-medium mb-2" for="braSize">Bra Size</label>
				<input id="braSize" type="text" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={braSize} />
			</div>
		{:else}
			<div class="flex gap-5">
				<div class="w-full">
					<label for="eLevel" class="block text-sm mb-1">Estradiol Level</label>
					<input
						id="eLevel"
						type="number"
						step="any"
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
					<label for="estrannaiseNumber" class="block text-sm mb-1">Estrannaise predicted E2 (pg/mL)</label>
					<input
						id="estrannaiseNumber"
						type="number"
						step="any"
						bind:value={estrannaiseNumber}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<!-- empty div for alignment -->
				</div>
			</div>
			<div class="flex gap-5 mt-4">
				<div class="w-full">
					<label for="testLevel" class="block text-sm mb-1">Testosterone Level</label>
					<input
						id="testLevel"
						type="number"
						step="any"
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
			<div class="flex gap-5 mt-4">
				<div class="w-full">
					<label for="progesteroneLevel" class="block text-sm mb-1">Progesterone Level</label>
					<input
						id="progesteroneLevel"
						type="number"
						step="any"
						bind:value={progesteroneLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="progesteroneUnit" class="block text-sm mb-1">Progesterone Unit</label>
					<select
						id="progesteroneUnit"
						bind:value={progesteroneUnit}
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
					<label for="fshLevel" class="block text-sm mb-1">FSH Level</label>
					<input
						id="fshLevel"
						type="number"
						step="any"
						bind:value={fshLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="fshUnit" class="block text-sm mb-1">FSH Unit</label>
					<select
						id="fshUnit"
						bind:value={fshUnit}
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
					<label for="lhLevel" class="block text-sm mb-1">LH Level</label>
					<input
						id="lhLevel"
						type="number"
						step="any"
						bind:value={lhLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="lhUnit" class="block text-sm mb-1">LH Unit</label>
					<select
						id="lhUnit"
						bind:value={lhUnit}
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
					<label for="prolactinLevel" class="block text-sm mb-1">Prolactin Level</label>
					<input
						id="prolactinLevel"
						type="number"
						step="any"
						bind:value={prolactinLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="prolactinUnit" class="block text-sm mb-1">Prolactin Unit</label>
					<select
						id="prolactinUnit"
						bind:value={prolactinUnit}
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
					<label for="shbgLevel" class="block text-sm mb-1">SHBG Level</label>
					<input
						id="shbgLevel"
						type="number"
						step="any"
						bind:value={shbgLevel}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<label for="shbgUnit" class="block text-sm mb-1">SHBG Unit</label>
					<select
						id="shbgUnit"
						bind:value={shbgUnit}
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
					<label for="freeAndrogenIndex" class="block text-sm mb-1">Free Androgen Index</label>
					<input
						id="freeAndrogenIndex"
						type="number"
						step="any"
						bind:value={freeAndrogenIndex}
						class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight focus:outline-none focus:shadow-outline"
					/>
				</div>
				<div class="w-full">
					<!-- empty div for alignment -->
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

		<div class="flex justify-between items-center mt-6">
			<button
				class="px-4 py-2 rounded transition-colors bg-rose-pine-love text-white hover:bg-red-700"
				onclick={deleteEntry}>Delete</button
			>
			<div class="flex gap-4">
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
</div>
