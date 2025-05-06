<script lang="ts">
    import {
        HRT_STORAGE_KEY,
        type HRTData,
        type BloodTest,
        type EstrogenType,
        InjectableEstradiols,
        OralEstradiols,
        Antiandrogens,
    } from "$lib/types";
    import { getContext, setContext } from "svelte";

    let method: "injection" | "oral" = $state("injection");
    let injectionDateTime = $state("");
    let oralDateTime = $state("");
    let testDateTime = $state(Date.now());
    let testLevel = $state(0);
    let eLevel = $state(0);
    let testUnit = $state("");
    let eUnit = $state("");
    let notes = $state("");
    let estrogen: EstrogenType = $state({
        route: "injection",
        type: InjectableEstradiols.Benzoate,
    });
    $effect(() => {
        if (method === "injection") {
            estrogen = {
                route: "injection",
                type: InjectableEstradiols.Benzoate,
            };
        } else {
            estrogen = {
                route: "oral",
                type: OralEstradiols.Valerate,
            };
        }
    });
    let aa: Antiandrogens = $state(Antiandrogens.CPA);
    let hrtData = getContext(HRT_STORAGE_KEY) as HRTData;
    function enumToDropdownOptions(e: any) {
        return Object.entries(e).map(([key, val]) => ({
            value: val as string,
            label: val as string,
        }));
    }
    const oralOptions = enumToDropdownOptions(OralEstradiols);
    const aaOptions = enumToDropdownOptions(Antiandrogens);
    const injectOptions = enumToDropdownOptions(InjectableEstradiols);
    function submitForm() {
        const newBloodTest: BloodTest = {
            date: testDateTime,
            estradiolLevel: eLevel,
            testLevel: testLevel,
            testUnit: testUnit,
            estradiolUnit: eUnit,
            notes: notes,
            estrogenType: estrogen,
        };
        const updatedBloodTests = [...hrtData.bloodTests, newBloodTest];
        const data: HRTData = {
            bloodTests: updatedBloodTests,
            dosageHistory: [],
        };
        setContext(HRT_STORAGE_KEY, data);
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
    <form onsubmit={submitForm} class="shadow-md rounded pt-6 pb-8 mb-4">
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
            <span
                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
            >
                administration method
            </span>
            <label class="inline-flex items-center mr-4">
                <input
                    type="radio"
                    class="form-radio w-4 h-4 text-latte-rose-pine-foam"
                    bind:group={method}
                    value="injection"
                    id="injectionOption"
                />
                <span class="ml-2">Injection</span>
            </label>
            <label class="inline-flex items-center">
                <input
                    type="radio"
                    class="form-radio w-4 h-4 text-latte-rose-pine-foam"
                    bind:group={method}
                    value="oral"
                    id="oralOption"
                />
                <span class="ml-2">Oral</span>
            </label>
        </div>
        {#if method === "injection"}
            <div class="mb-4 space-y-4">
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="injectionDateTime"
                >
                    injection date / time
                </label>
                <input
                    id="injectionDateTime"
                    type="datetime-local"
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={injectionDateTime}
                />
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="injectedHormone"
                >
                    injected hormone
                </label>
                <!-- <input
                    id="injectedHormone"
                    type="datetime-local"
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={injectionDateTime}
                /> -->
                <select
                    class="border py-2 px-3 rounded w-full leading-tight"
                    id="injectedHormone"
                    bind:value={estrogen.type}
                >
                    {#each injectOptions as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </select>
            </div>
        {:else}
            <div class="mb-4">
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="oralDateTime"
                >
                    oral date / time
                </label>
                <input
                    id="oralDateTime"
                    type="datetime-local"
                    class="mb-2 shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={oralDateTime}
                />
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="consumedHormone1"
                >
                    consumed hormone(s)
                </label>
                <div class="flex gap-5">
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="consumedHormone1"
                        bind:value={estrogen.type}
                    >
                        {#each oralOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                    <!-- for any antiandrogen -->
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight"
                        id="consumedHormone2"
                        bind:value={aa}
                    >
                        {#each aaOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
                <!-- <input
                    id="consumedHormone1"
                    type="datetime-local"
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={injectionDateTime}
                /> -->
                <!-- <input
                    id="consumedHormone2"
                    type="datetime-local"
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={injectionDateTime}
                /> -->
            </div>
        {/if}
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
