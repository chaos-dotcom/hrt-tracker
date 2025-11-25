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
        SyringeKinds,   // ADD
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
    let injectionFrequency = $state(hrtData.data.injectableEstradiol?.frequency ?? 7);
    let oralEFrequency = $state(hrtData.data.oralEstradiol?.frequency ?? 1);
    let eNextDoseDate = $state("");

    // Injection helper: dose/conc <-> volume
    let injConvDoseMg = $state(4);
    let injConvConcMgPerMl = $state(40);
    const injConvVolMl = $derived(injConvConcMgPerMl > 0 ? injConvDoseMg / injConvConcMgPerMl : NaN);

    let injConvVol2Ml = $state(0.1);
    let injConvConc2MgPerMl = $state(40);
    const injConvDose2Mg = $derived(injConvConc2MgPerMl > 0 ? injConvVol2Ml * injConvConc2MgPerMl : NaN);

    // Antiandrogen state
    let aaType: Antiandrogens | "" = $state("");
    let aaDose = $state(0);
    let aaUnit: HormoneUnits = $state(HormoneUnits.mg);
    let aaFrequency = $state(hrtData.data.antiandrogen?.frequency ?? 1);
    let aaNextDoseDate = $state("");

    // Progesterone state
    let pType: Progesterones | "" = $state("");
    let pDose = $state(0);
    let pUnit: HormoneUnits = $state(HormoneUnits.mg);
    let pRoute: ProgesteroneRoutes = $state(ProgesteroneRoutes.Oral);
    let pFrequency = $state(hrtData.data.progesterone?.frequency ?? 1);
    let pNextDoseDate = $state("");

    // State for "Record Dose" mode
    let recordEstrogen = $state(true);
    let recordAA = $state(false);
    let recordProg = $state(false);
    
    // Note state for each medication type
    let eNote = $state("");
    let aaNote = $state("");
    let pNote = $state("");
    let ePillQty = $state(1);     // for oral estradiol
    let pPillQty = $state(1);     // for progesterone
    
    // Injection site for injectable estrogen
    let eInjectionSite: InjectionSites | "" = $state("");
    let syringeKind: SyringeKinds | '' = $state('');
    let needleLength = $state('');
    let needleGauge = $state('');

    // Selected vial/sub‑vial (for injections)
    let selectedVialId = $state('');
    let selectedSubVialId = $state('');
    $effect(() => {
        const v = hrtData.data.vials.find((x) => x.id === selectedVialId);
        if (!v || !v.subVials.some((s) => s.id === selectedSubVialId)) {
            selectedSubVialId = '';
        }
    });

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

    let recordDateTime = $state("");

    $effect(() => {
        if (mode === "record" && !recordDateTime) {
            recordDateTime = toLocalInputValue(Date.now());
        }
    });

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
            selectedVialId = injSched.vialId || '';          // ADDED
            selectedSubVialId = injSched.subVialId || '';    // ADDED
            syringeKind = (injSched as any).syringeKind || '';
            needleLength = (injSched as any).needleLength || '';
            needleGauge = (injSched as any).needleGauge || '';
        } else if (oralSched) {
            estrogenMethod = "oral";
            oralEType = oralSched.type;
            eDose = oralSched.dose;
            eUnit = oralSched.unit;
            oralEFrequency = oralSched.frequency || 1;
            eNextDoseDate = oralSched.nextDoseDate ? toLocalInputValue(oralSched.nextDoseDate) : "";
            selectedVialId = '';          // ADDED
            selectedSubVialId = '';       // ADDED
            syringeKind = '';
            needleLength = '';
            needleGauge = '';
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
                vialId: selectedVialId || undefined,         // ADDED
                subVialId: selectedSubVialId || undefined,   // ADDED
                syringeKind: syringeKind || undefined,         // ADDED
                needleLength: needleLength.trim() || undefined, // ADDED
                needleGauge: needleGauge.trim() || undefined,
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
        // Persist to server files explicitly
        hrtData.saveNow();
        alert("Schedule saved!");
    }

    function submitDosageForm() {
        const recordMs = new Date(recordDateTime).getTime();
        if (recordEstrogen) {
            let estrogenRecord: DosageHistoryEntry;
            if (estrogenMethod === "injection") {
                estrogenRecord = {
                    date: recordMs,
                    medicationType: "injectableEstradiol",
                    type: injectableEType,
                    dose: eDose,
                    unit: eUnit,
                    note: eNote.trim() || undefined,
                    injectionSite: eInjectionSite || undefined,
                    vialId: selectedVialId || undefined,       // ADDED
                    subVialId: selectedSubVialId || undefined, // ADDED
                    syringeKind: syringeKind || undefined,         // ADDED
                    needleLength: needleLength.trim() || undefined, // ADDED
                    needleGauge: needleGauge.trim() || undefined,
                };
            } else {
                estrogenRecord = {
                    date: recordMs,
                    medicationType: "oralEstradiol",
                    type: oralEType,
                    dose: eDose,
                    unit: eUnit,
                    pillQuantity: Number.isFinite(+ePillQty) && +ePillQty > 0 ? +ePillQty : undefined,
                    note: eNote.trim() || undefined,
                };
            }
            hrtData.addDosageRecord(estrogenRecord);
        }

        if (recordAA && aaType !== "") {
            const aaRecord: DosageHistoryEntry = {
                date: recordMs,
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
                date: recordMs,
                medicationType: "progesterone",
                type: pType,
                route: pRoute,
                dose: pDose,
                unit: pUnit,
                pillQuantity: Number.isFinite(+pPillQty) && +pPillQty > 0 ? +pPillQty : undefined,
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
        {#if estrogenMethod === 'injection'}
        <div class="mb-6 p-4 border rounded-lg">
            <h3 class="text-lg font-medium mb-3">Injection helper</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <div class="text-sm font-medium mb-2">Dose and Concentration to Volume</div>
                    <div class="flex items-center gap-2 flex-wrap">
                        <label class="flex items-center gap-2">
                            <span>Dose</span>
                            <input type="number" step="any" class="shadow appearance-none border rounded px-3 py-2 leading-tight w-32" bind:value={injConvDoseMg} /> mg
                        </label>
                        <label class="flex items-center gap-2">
                            <span>Concentration</span>
                            <input type="number" step="any" class="shadow appearance-none border rounded px-3 py-2 leading-tight w-32" bind:value={injConvConcMgPerMl} /> mg/mL
                        </label>
                    </div>
                    <div class="mt-2 text-sm">
                        Volume = Dose ÷ Concentration = <strong>{Number.isFinite(injConvVolMl) ? injConvVolMl.toFixed(3).replace(/\.?0+$/, '') : '—'}</strong> mL {#if Number.isFinite(injConvVolMl)}(≈ <strong>{Math.round(injConvVolMl * 100)}</strong> IU){/if}
                    </div>
                </div>
                <div>
                    <div class="text-sm font-medium mb-2">Volume and Concentration to Dose</div>
                    <div class="flex items-center gap-2 flex-wrap">
                        <label class="flex items-center gap-2">
                            <span>Volume</span>
                            <input type="number" step="any" class="shadow appearance-none border rounded px-3 py-2 leading-tight w-32" bind:value={injConvVol2Ml} /> mL
                            <span style="opacity:0.7; font-size: 0.9em;">(≈ {Number.isFinite(injConvVol2Ml) ? Math.round(injConvVol2Ml * 100) : '—'} IU)</span>
                        </label>
                        <label class="flex items-center gap-2">
                            <span>Concentration</span>
                            <input type="number" step="any" class="shadow appearance-none border rounded px-3 py-2 leading-tight w-32" bind:value={injConvConc2MgPerMl} /> mg/mL
                        </label>
                    </div>
                    <div class="mt-2 text-sm">
                        Dose = Volume × Concentration = <strong>{Number.isFinite(injConvDose2Mg) ? injConvDose2Mg.toFixed(3).replace(/\.?0+$/, '') : '—'}</strong> mg
                    </div>
                </div>
            </div>
        </div>
        {/if}
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

        {#if mode === "record"}
            <div class="mb-4">
                <label class="block text-sm font-medium mb-2" for="recordDateTime">date / time</label>
                <input
                    id="recordDateTime"
                    type="datetime-local"
                    class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight"
                    bind:value={recordDateTime}
                    required
                />
            </div>
        {/if}

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
                                <option value={InjectionSites.TopThighRight}>Thigh right</option>
                                <option value={InjectionSites.TopThighLeft}>Thigh right</option>
                                <option value={InjectionSites.InnerThighRight}>Thigh right</option>
                                <option value={InjectionSites.InnerThighLeft}>Thigh right</option>
                                <option value={InjectionSites.OuterThighLeft}>Thigh right</option>
                                <option value={InjectionSites.OuterThighRight}>Thigh right</option>

                                <option value={InjectionSites.ThighLeft}>Thigh left</option>
                                <option value={InjectionSites.ButtockRight}>Buttock right</option>
                                <option value={InjectionSites.ButtockLeft}>Buttock left</option>
                            </select>
                        </div>
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eVial">vial (optional)</label>
                            <div class="flex items-center gap-2">
                                <select id="eVial" class="border py-2 px-3 rounded w-full leading-tight" bind:value={selectedVialId}>
                                    <option value="">None</option>
                                    {#each hrtData.data.vials.filter(v => !v.isSpent || v.id === selectedVialId) as v}
                                        <option value={v.id}>
                                            {(v.esterKind || 'Unknown ester') + ' · ' + (v.batchNumber || 'batch ?') + (v.source ? ' · ' + v.source : '')}
                                        </option>
                                    {/each}
                                </select>
                                <a class="text-latte-rose-pine-iris hover:text-rose-pine-love whitespace-nowrap" href="/vials/create">New…</a>
                            </div>
                        </div>
                        {#if selectedVialId}
                            {#each hrtData.data.vials.filter(v => v.id === selectedVialId) as v}
                                {#if v.subVials.length > 0}
                                    <div class="mb-4">
                                        <label class="block text-sm font-medium mb-2" for="eSubVial">sub‑vial / cartridge (optional)</label>
                                        <select id="eSubVial" class="border py-2 px-3 rounded w-full leading-tight" bind:value={selectedSubVialId}>
                                            <option value="">None</option>
                                            {#each v.subVials as s}
                                                <option value={s.id}>#{s.personalNumber}</option>
                                            {/each}
                                        </select>
                                    </div>
                                {/if}
                            {/each}
                        {/if}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eSyringeKind">syringe kind (optional)</label>
                            <select id="eSyringeKind" class="border py-2 px-3 rounded w-full leading-tight" bind:value={syringeKind}>
                                <option value="">Select...</option>
                                {#each Object.values(SyringeKinds) as opt}
                                    <option value={opt}>{opt}</option>
                                {/each}
                            </select>
                        </div>
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eNeedleLen">needle length (optional)</label>
                            <input id="eNeedleLen" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" placeholder='e.g., 4mm or 1"' bind:value={needleLength} />
                        </div>
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="eNeedleGauge">needle gauge (optional)</label>
                            <input id="eNeedleGauge" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" placeholder="e.g., 32g or 30G" bind:value={needleGauge} />
                        </div>
                        {/if}
                        {#if estrogenMethod === 'oral'}
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="ePillQty">pill quantity</label>
                            <input id="ePillQty" type="number" min="1" step="1" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={ePillQty} />
                            <div class="text-sm opacity-80">
                                Total = {(+eDose || 0)} mg/pill × {(+ePillQty || 0)} = <strong>{Number.isFinite(+eDose) && Number.isFinite(+ePillQty) ? (+eDose) * (+ePillQty) : 0}</strong> mg
                            </div>
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

                {#if mode === 'schedule' && estrogenMethod === 'injection'}
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="schedVial">vial (optional)</label>
                        <div class="flex items-center gap-2">
                            <select id="schedVial" class="border py-2 px-3 rounded w-full leading-tight" bind:value={selectedVialId}>
                                <option value="">None</option>
                                {#each hrtData.data.vials.filter(v => !v.isSpent || v.id === selectedVialId) as v}
                                    <option value={v.id}>
                                        {(v.esterKind || 'Unknown ester') + ' · ' + (v.batchNumber || 'batch ?') + (v.source ? ' · ' + v.source : '')}
                                    </option>
                                {/each}
                            </select>
                            <a class="text-latte-rose-pine-iris hover:text-rose-pine-love whitespace-nowrap" href="/vials/create">New…</a>
                        </div>
                    </div>
                    {#if selectedVialId}
                        {#each hrtData.data.vials.filter(v => v.id === selectedVialId) as v}
                            {#if v.subVials.length > 0}
                                <div class="mb-4">
                                    <label class="block text-sm font-medium mb-2" for="schedSubVial">sub‑vial / cartridge (optional)</label>
                                    <select id="schedSubVial" class="border py-2 px-3 rounded w-full leading-tight" bind:value={selectedSubVialId}>
                                        <option value="">None</option>
                                        {#each v.subVials as s}
                                            <option value={s.id}>#{s.personalNumber}</option>
                                        {/each}
                                    </select>
                                </div>
                            {/if}
                        {/each}
                    {/if}
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="schedSyringeKind">syringe kind (optional)</label>
                        <select id="schedSyringeKind" class="border py-2 px-3 rounded w-full leading-tight" bind:value={syringeKind}>
                            <option value="">Select...</option>
                            {#each Object.values(SyringeKinds) as opt}
                                <option value={opt}>{opt}</option>
                            {/each}
                        </select>
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="schedNeedleLen">needle length (optional)</label>
                        <input id="schedNeedleLen" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" placeholder='e.g., 4mm or 1"' bind:value={needleLength} />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-2" for="schedNeedleGauge">needle gauge (optional)</label>
                        <input id="schedNeedleGauge" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" placeholder="e.g., 32g or 30G" bind:value={needleGauge} />
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
                            <label class="block text-sm font-medium mb-2" for="pNote">note (optional)</label>
                            <textarea id="pNote" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pNote} rows="2" placeholder="Add any notes about this dose"></textarea>
                        </div>
                        <div class="mb-4">
                            <label class="block text-sm font-medium mb-2" for="pPillQty">pill quantity</label>
                            <input id="pPillQty" type="number" min="1" step="1" class="shadow appearance-none border rounded w-full py-2 px-3 leading-tight" bind:value={pPillQty} />
                            <div class="text-sm opacity-80">
                                Total = {(+pDose || 0)} mg/pill × {(+pPillQty || 0)} = <strong>{Number.isFinite(+pDose) && Number.isFinite(+pPillQty) ? (+pDose) * (+pPillQty) : 0}</strong> mg
                            </div>
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
