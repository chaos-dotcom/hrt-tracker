<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import { type BloodTest, HormoneUnits } from "$lib/types";

    function toLocalInputValue(ms: number) {
        const d = new Date(ms);
        const pad = (n: number) => String(n).padStart(2, "0");
        const yyyy = d.getFullYear();
        const mm = pad(d.getMonth() + 1);
        const dd = pad(d.getDate());
        const hh = pad(d.getHours());
        const mi = pad(d.getMinutes());
        return `${yyyy}-${mm}-${dd}T${hh}:${mi}`;
    }
    let testDateTime = $state(toLocalInputValue(Date.now()));
    let testLevel = $state(0);
    let eLevel = $state(0);
    let testUnit: HormoneUnits = $state(HormoneUnits.T_ng_dL);
    let eUnit: HormoneUnits = $state(HormoneUnits.E2_pg_mL);
    let progesteroneLevel = $state(0);
    let progesteroneUnit: HormoneUnits = $state(HormoneUnits.ng_mL);
    let fshLevel = $state(0);
    let fshUnit: HormoneUnits = $state(HormoneUnits.mIU_mL);
    let lhLevel = $state(0);
    let lhUnit: HormoneUnits = $state(HormoneUnits.mIU_mL);
    let prolactinLevel = $state(0);
    let prolactinUnit: HormoneUnits = $state(HormoneUnits.ng_mL);
    let shbgLevel = $state(0);
    let shbgUnit: HormoneUnits = $state(HormoneUnits.T_nmol_L);
    let freeAndrogenIndex = $state(0);
    let estrannaiseNumber = $state(0);
    let notes = $state("");
    let showFeedback = $state(false);

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
        const measuredE2 =
            eUnit === HormoneUnits.E2_pmol_L
                ? eLevel / 3.671
                : eLevel;
        const computedFudgeFactor =
            isFinite(measuredE2) && isFinite(estrannaiseNumber) && estrannaiseNumber > 0
                ? measuredE2 / estrannaiseNumber
                : undefined;
        const newBloodTest: BloodTest = {
            date: new Date(testDateTime).getTime(),
            estradiolLevel: eLevel,
            testLevel: testLevel,
            testUnit: testUnit,
            estradiolUnit: eUnit,
            progesteroneLevel: progesteroneLevel,
            progesteroneUnit: progesteroneUnit,
            fshLevel: fshLevel,
            fshUnit: fshUnit,
            lhLevel: lhLevel,
            lhUnit: lhUnit,
            prolactinLevel: prolactinLevel,
            prolactinUnit: prolactinUnit,
            shbgLevel: shbgLevel,
            shbgUnit: shbgUnit,
            freeAndrogenIndex: freeAndrogenIndex,
            estrannaiseNumber: estrannaiseNumber,
            fudgeFactor:
                typeof computedFudgeFactor === 'number'
                    ? Number(computedFudgeFactor.toFixed(3))
                    : undefined,
            notes: notes,
        };
        hrtData.addBloodTest(newBloodTest);
        showFeedback = true;
        setTimeout(() => (showFeedback = false), 3000);
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
    <form on:submit={handleSubmit} class="shadow-md rounded pt-6 pb-8 mb-4">
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
                        step="any"
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
                        for="estrannaiseNumber"
                    >
                        estrannaise predicted E2 (pg/mL)
                    </label>
                    <input
                        id="estrannaiseNumber"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={estrannaiseNumber}
                    />
                </div>
                <div class="w-full">
                    <!-- empty div for alignment -->
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
                        step="any"
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

            <div class="flex gap-5 mt-4">
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="progesteroneLevel"
                    >
                        progesterone level
                    </label>
                    <input
                        id="progesteroneLevel"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={progesteroneLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="progesteroneUnit"
                    >
                        progesterone unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="progesteroneUnit"
                        bind:value={progesteroneUnit}
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
                        for="fshLevel"
                    >
                        FSH level
                    </label>
                    <input
                        id="fshLevel"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={fshLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="fshUnit"
                    >
                        FSH unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="fshUnit"
                        bind:value={fshUnit}
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
                        for="lhLevel"
                    >
                        LH level
                    </label>
                    <input
                        id="lhLevel"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={lhLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="lhUnit"
                    >
                        LH unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="lhUnit"
                        bind:value={lhUnit}
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
                        for="prolactinLevel"
                    >
                        prolactin level
                    </label>
                    <input
                        id="prolactinLevel"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={prolactinLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="prolactinUnit"
                    >
                        prolactin unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="prolactinUnit"
                        bind:value={prolactinUnit}
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
                        for="shbgLevel"
                    >
                        SHBG level
                    </label>
                    <input
                        id="shbgLevel"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={shbgLevel}
                    />
                </div>
                <div class="w-full">
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm mb-1"
                        for="shbgUnit"
                    >
                        SHBG unit
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="shbgUnit"
                        bind:value={shbgUnit}
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
                        for="freeAndrogenIndex"
                    >
                        Free Androgen Index
                    </label>
                    <input
                        id="freeAndrogenIndex"
                        type="number"
                        step="any"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                        bind:value={freeAndrogenIndex}
                    />
                </div>
                <div class="w-full">
                    <!-- empty div for alignment -->
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
                type="submit"
            >
                create test
            </button>
            {#if showFeedback}
                <p
                    class="text-latte-rose-pine-text dark:text-rose-pine-text transition-opacity"
                >
                    blood test added!
                </p>
            {/if}
        </div>
    </form>
</div>
