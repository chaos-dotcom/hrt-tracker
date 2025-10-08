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

    // Estrogen state
    let estrogenMethod: "injection" | "oral" = $state("injection");
    let injectableEType: InjectableEstradiols = $state(
        InjectableEstradiols.Benzoate,
    );
    let oralEType: OralEstradiols = $state(OralEstradiols.Valerate);
    let eDose = $state(0);
    let eUnit: HormoneUnits = $state(HormoneUnits.mg);
    let injectionFrequency = $state(7);
    let oralEFrequency = $state("");
    let eDateTime = $state("");

    // Antiandrogen state
    let aaType: Antiandrogens | "" = $state("");
    let aaDose = $state(0);
    let aaUnit: HormoneUnits = $state(HormoneUnits.mg);
    let aaFrequency = $state("");
    let aaDateTime = $state("");

    // Progesterone state
    let pType: Progesterones | "" = $state("");
    let pDose = $state(0);
    let pUnit: HormoneUnits = $state(HormoneUnits.mg);
    let pRoute: ProgesteroneRoutes = $state(ProgesteroneRoutes.Oral);
    let pFrequency = $state("");
    let pDateTime = $state("");

    // State for "Record Dose" mode
    let recordEstrogen = $state(true);
    let recordAA = $state(false);
    let recordProg = $state(false);

    $effect(() => {
        if ($page.url.searchParams.get("mode") === "schedule") {
            mode = "schedule";
        }
        // Load schedule data when component mounts or data changes
        // Estrogen
        const injSched = hrtData.data.injectableEstradiol;
        const oralSched = hrtData.data.oralEstradiol;
        if (injSched) {
            estrogenMethod = "injection";
            injectableEType = injSched.type;
            eDose = injSched.dose;
            eUnit = injSched.unit;
            injectionFrequency = injSched.frequency;
        } else if (oralSched) {
            estrogenMethod = "oral";
            oralEType = oralSched.type;
            eDose = oralSched.dose;
            eUnit = oralSched.unit;
            oralEFrequency = oralSched.frequency;
        }

        // AA
        const aaSched = hrtData.data.antiandrogen;
        aaType = aaSched?.type || "";
        aaDose = aaSched?.dose || 0;
        aaUnit = aaSched?.unit || HormoneUnits.mg;
        aaFrequency = aaSched?.frequency || "";

        // Progesterone
        const pSched = hrtData.data.progesterone;
        pType = pSched?.type || "";
        pDose = pSched?.dose || 0;
        pUnit = pSched?.unit || HormoneUnits.mg;
        pRoute = pSched?.route || ProgesteroneRoutes.Oral;
        pFrequency = pSched?.frequency || "";
    });

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
        // Estrogen
        if (estrogenMethod === "injection") {
            hrtData.data.injectableEstradiol = {
                type: injectableEType,
                dose: eDose,
                unit: eUnit,
                frequency: injectionFrequency,
            };
            hrtData.data.oralEstradiol = undefined;
        } else {
            hrtData.data.oralEstradiol = {
                type: oralEType,
                dose: eDose,
                unit: eUnit,
                frequency: oralEFrequency,
            };
            hrtData.data.injectableEstradiol = undefined;
        }

        // Antiandrogen
        if (aaType !== "") {
            hrtData.data.antiandrogen = {
                type: aaType,
                dose: aaDose,
                unit: aaUnit,
                frequency: aaFrequency,
            };
        } else {
            hrtData.data.antiandrogen = undefined;
        }

        // Progesterone
        if (pType !== "") {
            hrtData.data.progesterone = {
                type: pType,
                route: pRoute,
                dose: pDose,
                unit: pUnit,
                frequency: pFrequency,
            };
        } else {
            hrtData.data.progesterone = undefined;
        }
        alert("Schedule saved!");
    }

    function submitDosageForm() {
        if (recordEstrogen) {
            let estrogenRecord: DosageHistoryEntry;
            if (estrogenMethod === "injection") {
                estrogenRecord = {
                    date: new Date(eDateTime).getTime(),
                    medicationType: "injectableEstradiol",
                    type: injectableEType,
                    dose: eDose,
                    unit: eUnit,
                };
            } else {
                estrogenRecord = {
                    date: new Date(eDateTime).getTime(),
                    medicationType: "oralEstradiol",
                    type: oralEType,
                    dose: eDose,
                    unit: eUnit,
                };
            }
            hrtData.addDosageRecord(estrogenRecord);
        }

        if (recordAA && aaType !== "") {
            const aaRecord: DosageHistoryEntry = {
                date: new Date(aaDateTime).getTime(),
                medicationType: "antiandrogen",
                type: aaType,
                dose: aaDose,
                unit: aaUnit,
            };
            hrtData.addDosageRecord(aaRecord);
        }

        if (recordProg && pType !== "") {
            const pRecord: DosageHistoryEntry = {
                date: new Date(pDateTime).getTime(),
                medicationType: "progesterone",
                type: pType,
                route: pRoute,
                dose: pDose,
                unit: pUnit,
            };
            hrtData.addDosageRecord(pRecord);
        }
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

        <div class="space-y-6">
            <!-- Estrogen Section -->
            <div class="p-4 border rounded-lg">
                <h3 class="text-lg font-medium mb-2">Estrogen</h3>
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
                            bind:group={estrogenMethod}
                            value="injection"
                        />
                        <span class="ml-2">Injection</span>
                    </label>
                    <label class="inline-flex items-center">
                        <input
                            type="radio"
                            class="form-radio w-4 h-4 text-latte-rose-pine-foam"
                            bind:group={estrogenMethod}
                            value="oral"
                        />
                        <span class="ml-2">Oral</span>
                    </label>
                </div>

                {#if mode === "record"}
                    <div class="mb-4">
                        <label class="flex items-center">
                            <input type="checkbox" class="form-checkbox" bind:checked={recordEstrogen} />
                            <span class="ml-2">Record Estrogen Dose</span>
                        </label>
                    </div>
                    {#if recordEstrogen}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eDateTime">date / time</label>
                            <input id="eDateTime" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={eDateTime} required />
                        </div>
                    {/if}
                {/if}

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium mb-2" for="eType">type</label>
                        {#if estrogenMethod === 'injection'}
                            <select id="eType" class="border py-2 px-3 rounded w-full leading-tight" bind:value={injectableEType}>
                                {#each injectOptions as option}
                                    <option value={option.value}>{option.label}</option>
                                {/each}
                            </select>
                        {:else}
                            <select id="eType" class="border py-2 px-3 rounded w-full leading-tight" bind:value={oralEType}>
                                {#each oralOptions as option}
                                    <option value={option.value}>{option.label}</option>
                                {/each}
                            </select>
                        {/if}
                    </div>
                    {#if mode === 'schedule'}
                        <div>
                            <label class="block text-sm font-medium mb-2" for="eFrequency">frequency</label>
                            {#if estrogenMethod === 'injection'}
                                <input id="eFrequency" type="number" placeholder="e.g. 7" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={injectionFrequency} required />
                                <span class="text-xs italic">in days</span>
                            {:else}
                                <input id="eFrequency" type="text" placeholder="e.g. daily" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={oralEFrequency} required />
                            {/if}
                        </div>
                    {/if}
                    <div>
                        <label class="block text-sm font-medium mb-2" for="eDose">dose</label>
                        <input id="eDose" type="number" step="any" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={eDose} required />
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-2" for="eUnit">unit</label>
                        <select id="eUnit" class="border py-2 px-3 rounded w-full leading-tight" bind:value={eUnit}>
                            {#each unitOptions as option}
                                <option value={option.value}>{option.label}</option>
                            {/each}
                        </select>
                    </div>
                </div>
            </div>

            <!-- Antiandrogen Section -->
            <div class="p-4 border rounded-lg">
                <h3 class="text-lg font-medium mb-2">Antiandrogen</h3>
                {#if mode === "record"}
                    <div class="mb-4">
                        <label class="flex items-center">
                            <input type="checkbox" class="form-checkbox" bind:checked={recordAA} />
                            <span class="ml-2">Record Antiandrogen Dose</span>
                        </label>
                    </div>
                    {#if recordAA}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="aaDateTime">date / time</label>
                            <input id="aaDateTime" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaDateTime} required />
                        </div>
                    {/if}
                {/if}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium mb-2" for="aaType">type</label>
                        <select id="aaType" class="border py-2 px-3 rounded w-full leading-tight" bind:value={aaType}>
                            <option value="">None</option>
                            {#each aaOptions as option}
                                <option value={option.value}>{option.label}</option>
                            {/each}
                        </select>
                    </div>
                    {#if aaType !== ''}
                        {#if mode === 'schedule'}
                            <div>
                                <label class="block text-sm font-medium mb-2" for="aaFrequency">frequency</label>
                                <input id="aaFrequency" type="text" placeholder="e.g. daily" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaFrequency} />
                            </div>
                        {/if}
                        <div>
                            <label class="block text-sm font-medium mb-2" for="aaDose">dose</label>
                            <input id="aaDose" type="number" step="any" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaDose} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-2" for="aaUnit">unit</label>
                            <select id="aaUnit" class="border py-2 px-3 rounded w-full leading-tight" bind:value={aaUnit}>
                                {#each unitOptions as option}
                                    <option value={option.value}>{option.label}</option>
                                {/each}
                            </select>
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Progesterone Section -->
            <div class="p-4 border rounded-lg">
                <h3 class="text-lg font-medium mb-2">Progesterone</h3>
                 {#if mode === "record"}
                    <div class="mb-4">
                        <label class="flex items-center">
                            <input type="checkbox" class="form-checkbox" bind:checked={recordProg} />
                            <span class="ml-2">Record Progesterone Dose</span>
                        </label>
                    </div>
                    {#if recordProg}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="pDateTime">date / time</label>
                            <input id="pDateTime" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pDateTime} required />
                        </div>
                    {/if}
                {/if}
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                     <div>
                        <label class="block text-sm font-medium mb-2" for="pType">type</label>
                        <select id="pType" class="border py-2 px-3 rounded w-full leading-tight" bind:value={pType}>
                            <option value="">None</option>
                            {#each progesteroneOptions as option}
                                <option value={option.value}>{option.label}</option>
                            {/each}
                        </select>
                    </div>
                    {#if pType !== ''}
                        {#if mode === 'schedule'}
                            <div>
                                <label class="block text-sm font-medium mb-2" for="pFrequency">frequency</label>
                                <input id="pFrequency" type="text" placeholder="e.g. daily" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pFrequency} />
                            </div>
                        {/if}
                        <div>
                            <label class="block text-sm font-medium mb-2" for="pDose">dose</label>
                            <input id="pDose" type="number" step="any" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pDose} />
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-2" for="pUnit">unit</label>
                            <select id="pUnit" class="border py-2 px-3 rounded w-full leading-tight" bind:value={pUnit}>
                                {#each unitOptions as option}
                                    <option value={option.value}>{option.label}</option>
                                {/each}
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium mb-2" for="pRoute">route</label>
                            <select id="pRoute" class="border py-2 px-3 rounded w-full leading-tight" bind:value={pRoute}>
                                {#each progesteroneRouteOptions as option}
                                    <option value={option.value}>{option.label}</option>
                                {/each}
                            </select>
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <div class="flex items-center justify-between mt-6">
            <button
                class="cursor-pointer bg-latte-rose-pine-foam hover:bg-rose-pine-pine text-white font-medium py-2 px-4 rounded transition-colors focus:outline-none focus:shadow-outline"
                type="submit"
            >
                {mode === "record" ? "record dosage" : "save schedule"}
            </button>
        </div>
    </form>
</div>
