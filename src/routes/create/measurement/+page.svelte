<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import { type Measurement, WeightUnit, LengthUnit } from "$lib/types";

    let measurementDateTime = $state(new Date().toISOString().slice(0, 16));
    let weight = $state<number | undefined>(undefined);
    let weightUnit = $state(WeightUnit.KG);
    let height = $state<number | undefined>(undefined);
    let heightUnit = $state(LengthUnit.CM);
    let underbust = $state<number | undefined>(undefined);
    let bust = $state<number | undefined>(undefined);
    let bideltoid = $state<number | undefined>(undefined);
    let waist = $state<number | undefined>(undefined);
    let hip = $state<number | undefined>(undefined);
    let bodyMeasurementUnit = $state(LengthUnit.CM);
    let braSize = $state("");
    let showFeedback = $state(false);

    function enumToDropdownOptions(e: any) {
        return Object.entries(e).map(([, val]) => ({
            value: val as string,
            label: val as string,
        }));
    }
    const weightUnitOptions = enumToDropdownOptions(WeightUnit);
    const lengthUnitOptions = enumToDropdownOptions(LengthUnit);

    function handleSubmit(e: Event) {
        e.preventDefault();
        submitForm();
    }

    function submitForm() {
        const newMeasurement: Measurement = {
            date: new Date(measurementDateTime).getTime(),
            weight,
            weightUnit,
            height,
            heightUnit,
            underbust,
            bust,
            bideltoid,
            waist,
            hip,
            bodyMeasurementUnit,
            braSize,
        };
        hrtData.addMeasurement(newMeasurement);
        showFeedback = true;
        setTimeout(() => (showFeedback = false), 3000);
    }
</script>

<div class="p-10 flex flex-col space-y-2 sm:space-y-10">
    <div class="flex flex-col sm:flex-row sm:justify-between space-y-5 sm:space-y-0 mb-0">
        <h1 class="text-4xl">Create Measurement Entry</h1>
        <a class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors" href="/view">View History</a>
    </div>
    <form onsubmit={handleSubmit} class="shadow-md rounded pt-6 pb-8 mb-4">
        <div class="mb-4">
            <label class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2" for="measurementDate">
                Measurement Date / Time
            </label>
            <input class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline" id="measurementDate" type="datetime-local" bind:value={measurementDateTime} />
        </div>

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

        <div class="flex items-center justify-between">
            <button class="cursor-pointer bg-latte-rose-pine-foam hover:bg-rose-pine-pine text-white font-medium py-2 px-4 rounded transition-colors focus:outline-none focus:shadow-outline" type="submit">
                Create Measurement
            </button>
            {#if showFeedback}
                <p class="text-latte-rose-pine-text dark:text-rose-pine-text transition-opacity">
                    Measurement added!
                </p>
            {/if}
        </div>
    </form>
</div>
