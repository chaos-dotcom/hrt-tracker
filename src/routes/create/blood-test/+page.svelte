<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import { type BloodTest, HormoneUnits } from "$lib/types";

    let testDateTime = $state(Date.now());
    let testLevel = $state(0);
    let eLevel = $state(0);
    let testUnit: HormoneUnits = $state(HormoneUnits.T_ng_dL);
    let eUnit: HormoneUnits = $state(HormoneUnits.E2_pg_mL);
    let notes = $state("");

    function enumToDropdownOptions(e: any) {
        return Object.entries(e).map(([key, val]) => ({
            value: val as string,
            label: val as string,
        }));
    }
    const unitOptions = enumToDropdownOptions(HormoneUnits);
    function handleSubmit(e: Event) {
        e.preventDefault();
        submitForm();
    }
    function submitForm() {
        const newBloodTest: BloodTest = {
            date: testDateTime,
            estradiolLevel: eLevel,
            testLevel: testLevel,
            testUnit: testUnit,
            estradiolUnit: eUnit,
            notes: notes,
        };
        hrtData.addBloodTest(newBloodTest);
    }
</script>

<div class="p-10 flex flex-col space-y-2 sm:space-y-10">
    <div
        class="flex flex-col sm:flex-row sm:justify-between space-y-5 sm:space-y-0 mb-0"
    >
        <h1 class="text-4xl">create blood test entry</h1>
        <a
            class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors"
            href="/backup">view all tests</a
        >
    </div>
    <form onsubmit={handleSubmit} class="shadow-md rounded pt-6 pb-8 mb-4">
        <div class="mb-4">
            <label
                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                for="testDate"
            >
                test date / time
            </label>
            <input
                class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                id="testDate"
                type="datetime-local"
                bind:value={testDateTime}
            />
        </div>
        <div class="mb-4">
            <label
                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                for="hormone-levels"
            >
                hormone levels
            </label>
            <div class="flex gap-5">
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="eLevel"
                    >
                        estradiol level
                    </label>
                    <input
                        id="eLevel"
                        type="number"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={eLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="eUnit"
                    >
                        estradiol unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="eUnit"
                        bind:value={eUnit}
                    >
                        {#each unitOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
            </div>

            <div class="flex gap-5 mt-4">
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="testLevel"
                    >
                        testosterone level
                    </label>
                    <input
                        id="testLevel"
                        type="number"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={testLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="testUnit"
                    >
                        testosterone unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="testUnit"
                        bind:value={testUnit}
                    >
                        {#each unitOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
            </div>
        </div>
        <div class="mb-4">
            <textarea
                class="mb-2 shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                bind:value={notes}
                placeholder="notes..."
            ></textarea>
        </div>
        <div class="flex items-center justify-between">
            <button
                class="cursor-pointer bg-latte-rose-pine-foam hover:bg-rose-pine-pine text-white font-medium py-2 px-4 rounded transition-colors focus:outline-none focus:shadow-outline"
                type="button"
            >
                create test
            </button>
        </div>
    </form>
</div>
