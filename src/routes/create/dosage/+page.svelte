<script lang="ts">
    import { page } from "$app/stores";
    import { hrtData } from "$lib/storage.svelte";
    import { goto } from "$app/navigation";
    import {
        type EstrogenType,
        InjectableEstradiols,
        OralEstradiols,
        Antiandrogens,
        HormoneUnits,
        type DosageHistoryEntry,
        Progesterones,
        ProgesteroneRoutes,
        InjectionSites,
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
    let injectionFrequency = $state(hrtData.data.settings?.defaultInjectionFrequencyDays ?? 7);
    let oralEFrequency = $state(hrtData.data.settings?.defaultOralFrequencyDays ?? 1);
    let eDateTime = $state("");
    let eNextDoseDate = $state("");

    // Antiandrogen state
    let aaType: Antiandrogens | "" = $state("");
    let aaDose = $state(0);
    let aaUnit: HormoneUnits = $state(HormoneUnits.mg);
    let aaFrequency = $state(hrtData.data.settings?.defaultAntiandrogenFrequencyDays ?? 1);
    let aaDateTime = $state("");
    let aaNextDoseDate = $state("");

    // Progesterone state
    let pType: Progesterones | "" = $state("");
    let pDose = $state(0);
    let pUnit: HormoneUnits = $state(HormoneUnits.mg);
    let pRoute: ProgesteroneRoutes = $state(ProgesteroneRoutes.Oral);
    let pFrequency = $state(hrtData.data.settings?.defaultProgesteroneFrequencyDays ?? 1);
    let pDateTime = $state("");
    let pNextDoseDate = $state("");

    // State for "Record Dose" mode
    let recordEstrogen = $state(true);
    let recordAA = $state(false);
    let recordProg = $state(false);
    
    // Note state for each medication type
    let eNote = $state("");
    let aaNote = $state("");
    let pNote = $state("");
    
    // Injection site for injectable estrogen
    let eInjectionSite: InjectionSites | "" = $state("");

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
            eNextDoseDate = injSched.nextDoseDate ? toLocalInputValue(injSched.nextDoseDate) : "";
        } else if (oralSched) {
            estrogenMethod = "oral";
            oralEType = oralSched.type;
            eDose = oralSched.dose;
            eUnit = oralSched.unit;
            oralEFrequency = oralSched.frequency || 1;
            eNextDoseDate = oralSched.nextDoseDate ? toLocalInputValue(oralSched.nextDoseDate) : "";
        }

        // AA
        const aaSched = hrtData.data.antiandrogen;
        aaType = aaSched?.type || "";
        aaDose = aaSched?.dose || 0;
        aaUnit = aaSched?.unit || HormoneUnits.mg;
        aaFrequency = aaSched?.frequency || 1;
        aaNextDoseDate = aaSched?.nextDoseDate ? toLocalInputValue(aaSched.nextDoseDate) : "";

        // Progesterone
        const pSched = hrtData.data.progesterone;
        pType = pSched?.type || "";
        pDose = pSched?.dose || 0;
        pUnit = pSched?.unit || HormoneUnits.mg;
        pRoute = pSched?.route || ProgesteroneRoutes.Oral;
        pFrequency = pSched?.frequency || 1;
        pNextDoseDate = pSched?.nextDoseDate ? toLocalInputValue(pSched.nextDoseDate) : "";
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
            goto("/view");
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
                nextDoseDate: eNextDoseDate ? new Date(eNextDoseDate).getTime() : undefined,
            };
            hrtData.data.oralEstradiol = undefined;
        } else {
            hrtData.data.oralEstradiol = {
                type: oralEType,
                dose: eDose,
                unit: eUnit,
                frequency: oralEFrequency,
                nextDoseDate: eNextDoseDate ? new Date(eNextDoseDate).getTime() : undefined,
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
                nextDoseDate: aaNextDoseDate ? new Date(aaNextDoseDate).getTime() : undefined,
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
                nextDoseDate: pNextDoseDate ? new Date(pNextDoseDate).getTime() : undefined,
            };
        } else {
            hrtData.data.progesterone = undefined;
        }
        hrtData.backfillScheduledDoses();
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
                    note: eNote.trim() || undefined,
                    injectionSite: eInjectionSite || undefined,
                };
            } else {
                estrogenRecord = {
                    date: new Date(eDateTime).getTime(),
                    medicationType: "oralEstradiol",
                    type: oralEType,
                    dose: eDose,
                    unit: eUnit,
                    note: eNote.trim() || undefined,
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
                note: aaNote.trim() || undefined,
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
                note: pNote.trim() || undefined,
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
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eNote">note (optional)</label>
                            <textarea id="eNote" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={eNote} rows="2" placeholder="Add any notes about this dose"></textarea>
                        </div>
                        {#if estrogenMethod === 'injection'}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eInjectionSite">injection site (optional)</label>
                            <select id="eInjectionSite" class="border py-2 px-3 rounded w-full leading-tight" bind:value={eInjectionSite}>
                                <option value="">Select injection site</option>
                                <option value={InjectionSites.StomachRight}>Stomach right</option>
                                <option value={InjectionSites.StomachLeft}>Stomach left</option>
                                <option value={InjectionSites.ThighRight}>Thigh right</option>
                                <option value={InjectionSites.ThighLeft}>Thigh left</option>
                                <option value={InjectionSites.ButtockRight}>Buttock right</option>
                                <option value={InjectionSites.ButtockLeft}>Buttock left</option>
                            </select>
                        </div>
                        {/if}
                    {/if}
                {/if}

                {#if mode === 'schedule'}
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="eNextDoseDate">Next Dose Date</label>
                        <input id="eNextDoseDate" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={eNextDoseDate} />
                    </div>
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
                            <label class="block text-sm font-medium mb-2" for="eFrequency">frequency (in days)</label>
                            {#if estrogenMethod === 'injection'}
                                <input id="eFrequency" type="number" placeholder="e.g. 7" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={injectionFrequency} required />
                            {:else}
                                <input id="eFrequency" type="number" placeholder="e.g. 1" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={oralEFrequency} required />
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
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="aaNote">note (optional)</label>
                            <textarea id="aaNote" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaNote} rows="2" placeholder="Add any notes about this dose"></textarea>
                        </div>
                    {/if}
                {/if}
                {#if mode === 'schedule' && aaType !== ''}
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="aaNextDoseDate">Next Dose Date</label>
                        <input id="aaNextDoseDate" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaNextDoseDate} />
                    </div>
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
                                <label class="block text-sm font-medium mb-2" for="aaFrequency">frequency (in days)</label>
                                <input id="aaFrequency" type="number" placeholder="e.g. 1" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={aaFrequency} />
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
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="pNote">note (optional)</label>
                            <textarea id="pNote" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pNote} rows="2" placeholder="Add any notes about this dose"></textarea>
                        </div>
                    {/if}
                {/if}
                {#if mode === 'schedule' && pType !== ''}
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="pNextDoseDate">Next Dose Date</label>
                        <input id="pNextDoseDate" type="datetime-local" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pNextDoseDate} />
                    </div>
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
                                <label class="block text-sm font-medium mb-2" for="pFrequency">frequency (in days)</label>
                                <input id="pFrequency" type="number" placeholder="e.g. 1" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pFrequency} />
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
