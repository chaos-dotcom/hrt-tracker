<script lang="ts">
    import { page } from "$app/stores";
    import { hrtData } from "$lib/storage.svelte";
    import {
        type EstrogenType,
        InjectableEstradiols,
        OralEstradiols,
        Antiandrogens,
        HormoneUnits,
        type DosageHistoryEntry,
        Progesterones,
        ProgesteroneRoutes,
    } from "$lib/types";

    let mode: "record" | "schedule" = $state("record");
    let method: "injection" | "oral" = $state("injection");
    let injectionDateTime = $state("");
    let oralDateTime = $state("");
    let eDose = $state(0);
    let eUnit: HormoneUnits = $state(HormoneUnits.E2_pg_mL);
    let aaDose = $state(0);
    let aaUnit: HormoneUnits = $state(HormoneUnits.mg);
    let showAaDosage = $state(false);
    let estrogen: EstrogenType = $state({
        route: "injection",
        type: InjectableEstradiols.Benzoate,
    });

    let injectionFrequency = $state(7);
    let oralEFrequency = $state("");
    let aaFrequency = $state("");
    let pFrequency = $state("");

    let pDose = $state(0);
    let pUnit: HormoneUnits = $state(HormoneUnits.mg);
    let pRoute: ProgesteroneRoutes = $state(ProgesteroneRoutes.Oral);
    let progesterone: Progesterones | "" = $state(Progesterones.Micronized);

    $effect(() => {
        if ($page.url.searchParams.get("mode") === "schedule") {
            mode = "schedule";
            if (hrtData.data.injectableEstradiol) {
                method = "injection";
            } else if (hrtData.data.oralEstradiol) {
                method = "oral";
            }
        }
    });

    $effect(() => {
        if (method === "injection") {
            const sched = hrtData.data.injectableEstradiol;
            estrogen = {
                route: "injection",
                type: sched?.type || InjectableEstradiols.Benzoate,
            };
            eDose = sched?.dose || 0;
            eUnit = sched?.unit || HormoneUnits.mg;
            injectionFrequency = sched?.frequency || 7;
            showAaDosage = false;
        } else {
            // oral
            const oralSched = hrtData.data.oralEstradiol;
            estrogen = {
                route: "oral",
                type: oralSched?.type || OralEstradiols.Valerate,
            };
            eDose = oralSched?.dose || 0;
            eUnit = oralSched?.unit || HormoneUnits.mg;
            oralEFrequency = oralSched?.frequency || "";

            const aaSched = hrtData.data.antiandrogen;
            aa = aaSched?.type || "";
            aaDose = aaSched?.dose || 0;
            aaUnit = aaSched?.unit || HormoneUnits.mg;
            aaFrequency = aaSched?.frequency || "";

            const pSched = hrtData.data.progesterone;
            progesterone = pSched?.type || "";
            pDose = pSched?.dose || 0;
            pUnit = pSched?.unit || HormoneUnits.mg;
            pRoute = pSched?.route || ProgesteroneRoutes.Oral;
            pFrequency = pSched?.frequency || "";

            showAaDosage = true;
        }
    });

    let aa: Antiandrogens | "" = $state(Antiandrogens.CPA);

    function enumToDropdownOptions(e: any) {
        return Object.entries(e).map(([key, val]) => ({
            value: val as string,
            label: val as string,
        }));
    }

    const oralOptions = enumToDropdownOptions(OralEstradiols);
    const aaOptions = enumToDropdownOptions(Antiandrogens);
    const injectOptions = enumToDropdownOptions(InjectableEstradiols);
    const unitOptions = enumToDropdownOptions(HormoneUnits);
    const progesteroneOptions = enumToDropdownOptions(Progesterones);
    const progesteroneRouteOptions = enumToDropdownOptions(ProgesteroneRoutes);

    function handleSubmit(event: Event) {
        event.preventDefault();
        if (mode === "record") {
            submitDosageForm();
        } else {
            saveSchedule();
        }
    }

    function saveSchedule() {
        if (method === "injection") {
            hrtData.data.injectableEstradiol = {
                type: estrogen.type as InjectableEstradiols,
                dose: eDose,
                unit: eUnit,
                frequency: injectionFrequency,
            };
            hrtData.data.oralEstradiol = undefined;
            hrtData.data.antiandrogen = undefined;
            hrtData.data.progesterone = undefined;
        } else {
            // oral
            hrtData.data.oralEstradiol = {
                type: estrogen.type as OralEstradiols,
                dose: eDose,
                unit: eUnit,
                frequency: oralEFrequency,
            };
            if (aa !== "") {
                hrtData.data.antiandrogen = {
                    type: aa,
                    dose: aaDose,
                    unit: aaUnit,
                    frequency: aaFrequency,
                };
            } else {
                hrtData.data.antiandrogen = undefined;
            }
            if (progesterone !== "") {
                hrtData.data.progesterone = {
                    type: progesterone,
                    route: pRoute,
                    dose: pDose,
                    unit: pUnit,
                    frequency: pFrequency,
                };
            } else {
                hrtData.data.progesterone = undefined;
            }
            hrtData.data.injectableEstradiol = undefined;
        }
        alert("Schedule saved!");
    }

    function submitDosageForm() {
        console.log("poo");
        let newDosageRecord: DosageHistoryEntry;
        const currentDateTime =
            method === "injection" ? injectionDateTime : oralDateTime;

        if (method === "injection") {
            newDosageRecord = {
                date: new Date(currentDateTime).getTime(),
                medicationType: "injectableEstradiol",
                type: estrogen.type as InjectableEstradiols,
                dose: eDose,
                unit: eUnit,
            };
        } else {
            newDosageRecord = {
                date: new Date(currentDateTime).getTime(),
                medicationType: "oralEstradiol",
                type: estrogen.type as OralEstradiols,
                dose: eDose,
                unit: eUnit,
            };

            if (aa !== "") {
                const aaRecord: DosageHistoryEntry = {
                    date: Date.now(),
                    medicationType: "antiandrogen",
                    type: aa,
                    dose: aaDose,
                    unit: aaUnit,
                };
                hrtData.addDosageRecord(aaRecord);
            }

            if (progesterone !== "") {
                const pRecord: DosageHistoryEntry = {
                    date: new Date(currentDateTime).getTime(),
                    medicationType: "progesterone",
                    type: progesterone,
                    route: pRoute,
                    dose: pDose,
                    unit: pUnit,
                };
                hrtData.addDosageRecord(pRecord);
            }
        }
        hrtData.addDosageRecord(newDosageRecord);
    }
</script>

<div class="p-10 flex flex-col space-y-2 sm:space-y-10">
    <div
        class="flex flex-col sm:flex-row sm:justify-between space-y-5 sm:space-y-0 mb-0"
    >
        <h1 class="text-4xl">set up / record dosage</h1>
        <a
            class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors"
            href="/view">view dosage history</a
        >
    </div>
    <form onsubmit={handleSubmit} class="shadow-md rounded pt-6 pb-8 mb-4">
        <div class="mb-4">
            <span
                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
            >
                mode
            </span>
            <label class="inline-flex items-center mr-4">
                <input
                    type="radio"
                    class="form-radio w-4 h-4 text-latte-rose-pine-foam"
                    bind:group={mode}
                    value="record"
                />
                <span class="ml-2">Record Dose</span>
            </label>
            <label class="inline-flex items-center">
                <input
                    type="radio"
                    class="form-radio w-4 h-4 text-latte-rose-pine-foam"
                    bind:group={mode}
                    value="schedule"
                />
                <span class="ml-2">Set Schedule</span>
            </label>
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
                <div>
                    {#if mode === "record"}
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
                            required
                        />
                    {:else}
                        <label
                            class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                            for="injectionFrequency"
                        >
                            injection frequency (in days)
                        </label>
                        <input
                            id="injectionFrequency"
                            type="number"
                            placeholder="e.g. 7"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                            bind:value={injectionFrequency}
                            required
                        />
                    {/if}
                </div>
                <div>
                    <label
                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                        for="injectedHormone"
                    >
                        injected hormone
                    </label>
                    <select
                        class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                        id="injectedHormone"
                        bind:value={estrogen.type}
                    >
                        {#each injectOptions as option}
                            <option value={option.value}>{option.label}</option>
                        {/each}
                    </select>
                </div>
            </div>
        {:else}
            <div class="mb-4 space-y-4">
                <div>
                    {#if mode === "record"}
                        <label
                            class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                            for="oralDateTime"
                        >
                            oral intake date / time
                        </label>
                        <input
                            id="oralDateTime"
                            type="datetime-local"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                            bind:value={oralDateTime}
                            required
                        />
                    {/if}
                </div>

                <div class="flex flex-col sm:flex-row md:space-x-4">
                    <!-- Estrogen section -->
                    <div class="w-full sm:w-1/2">
                        <h3 class="text-lg font-medium mb-2">Estrogen</h3>
                        <div class="mb-4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="consumedHormone1"
                            >
                                consumed estrogen
                            </label>
                            <select
                                class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                id="consumedHormone1"
                                bind:value={estrogen.type}
                            >
                                {#each oralOptions as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                        {#if mode === "schedule"}
                            <div class="mb-4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="oralEFrequency"
                                >
                                    frequency
                                </label>
                                <input
                                    id="oralEFrequency"
                                    type="text"
                                    placeholder="e.g. daily"
                                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                    bind:value={oralEFrequency}
                                    required
                                />
                            </div>
                        {/if}
                        <div class="mb-4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="dose"
                            >
                                dose
                            </label>
                            <input
                                id="dose"
                                type="number"
                                step="any"
                                class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                bind:value={eDose}
                                required
                            />
                        </div>

                        <div class="mb-4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="unit"
                            >
                                unit
                            </label>
                            <select
                                class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                id="unit"
                                bind:value={eUnit}
                            >
                                {#each unitOptions as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                    </div>

                    <!-- Antiandrogen section -->
                    <div class="w-full sm:w-1/2 mt-4 md:mt-0">
                        <h3 class="text-lg font-medium mb-2">Antiandrogen</h3>
                        <div class="mb-4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="consumedHormone2"
                            >
                                consumed antiandrogen
                            </label>
                            <select
                                class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                id="consumedHormone2"
                                bind:value={aa}
                            >
                                <option value="">None</option>
                                {#each aaOptions as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                        {#if showAaDosage && aa !== ""}
                            {#if mode === "schedule"}
                                <div class="mb-4">
                                    <label
                                        class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                        for="aaFrequency"
                                    >
                                        frequency
                                    </label>
                                    <input
                                        id="aaFrequency"
                                        type="text"
                                        placeholder="e.g. daily"
                                        class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                        bind:value={aaFrequency}
                                    />
                                </div>
                            {/if}
                            <div class="mb-4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="aaDose"
                                >
                                    antiandrogen dose
                                </label>
                                <input
                                    id="aaDose"
                                    type="number"
                                    step="any"
                                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                    bind:value={aaDose}
                                />
                            </div>

                            <div class="mb-4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="aaUnit"
                                >
                                    antiandrogen unit
                                </label>
                                <select
                                    class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                    id="aaUnit"
                                    bind:value={aaUnit}
                                >
                                    {#each unitOptions as option}
                                        <option value={option.value}
                                            >{option.label}</option
                                        >
                                    {/each}
                                </select>
                            </div>
                        {/if}
                    </div>
                </div>
                <!-- Progesterone section -->
                <div class="w-full mt-4 md:mt-4">
                    <h3 class="text-lg font-medium mb-2">Progesterone</h3>
                    {#if progesterone !== "" && mode === "schedule"}
                        <div class="mb-4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="pFrequency"
                            >
                                frequency
                            </label>
                            <input
                                id="pFrequency"
                                type="text"
                                placeholder="e.g. daily"
                                class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                bind:value={pFrequency}
                            />
                        </div>
                    {/if}
                    <div class="flex flex-col sm:flex-row md:space-x-4">
                        <div class="w-full sm:w-1/4">
                            <label
                                class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                for="progesteroneType"
                            >
                                progesterone type
                            </label>
                            <select
                                class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                id="progesteroneType"
                                bind:value={progesterone}
                            >
                                <option value="">None</option>
                                {#each progesteroneOptions as option}
                                    <option value={option.value}
                                        >{option.label}</option
                                    >
                                {/each}
                            </select>
                        </div>
                        {#if progesterone !== ""}
                            <div class="w-full sm:w-1/4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="pDose"
                                >
                                    dose
                                </label>
                                <input
                                    id="pDose"
                                    type="number"
                                    step="any"
                                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                                    bind:value={pDose}
                                />
                            </div>
                            <div class="w-full sm:w-1/4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="pUnit"
                                >
                                    unit
                                </label>
                                <select
                                    class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                    id="pUnit"
                                    bind:value={pUnit}
                                >
                                    {#each unitOptions as option}
                                        <option value={option.value}
                                            >{option.label}</option
                                        >
                                    {/each}
                                </select>
                            </div>
                            <div class="w-full sm:w-1/4">
                                <label
                                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                                    for="pRoute"
                                >
                                    route
                                </label>
                                <select
                                    class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                                    id="pRoute"
                                    bind:value={pRoute}
                                >
                                    {#each progesteroneRouteOptions as option}
                                        <option value={option.value}
                                            >{option.label}</option
                                        >
                                    {/each}
                                </select>
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        {/if}

        {#if method === "injection"}
            <div class="mb-4">
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="dose"
                >
                    dose
                </label>
                <input
                    id="dose"
                    type="number"
                    step="any"
                    class="shadow appearance-none border rounded w-full py-2 px-3 text-latte-rose-pine-text dark:text-rose-pine-text leading-tight focus:outline-none focus:shadow-outline"
                    bind:value={eDose}
                    required
                />
            </div>

            <div class="mb-4">
                <label
                    class="block text-latte-rose-pine-text dark:text-rose-pine-text text-sm font-medium mb-2"
                    for="unit"
                >
                    unit
                </label>
                <select
                    class="border py-2 px-3 rounded w-full leading-tight bg-white dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text"
                    id="unit"
                    bind:value={eUnit}
                >
                    {#each unitOptions as option}
                        <option value={option.value}>{option.label}</option>
                    {/each}
                </select>
            </div>
        {/if}

        <div class="flex items-center justify-between">
            <button
                class="cursor-pointer bg-latte-rose-pine-foam hover:bg-rose-pine-pine text-white font-medium py-2 px-4 rounded transition-colors focus:outline-none focus:shadow-outline"
                type="submit"
            >
                {mode === "record" ? "record dosage" : "save schedule"}
            </button>
        </div>
    </form>
</div>
