<script lang="ts">
    export const ssr = false;

    import { hrtData } from "$lib/storage.svelte";
    import {
        Antiandrogens,
        HormoneUnits,
        type BloodTest,
        type DosageHistoryEntry,
        type EstrogenType,
        InjectableEstradiols,
        OralEstradiols,
        type Measurement,
        type DiaryEntry,
    } from "$lib/types";
    import * as Plot from "@observablehq/plot";
    import EditModal from "$lib/components/EditModal.svelte";

    // Diary / Notes are stored in HRTData via hrtData.data.notes

    // New note form state
    let noteTitle = $state("");
    let noteContent = $state("");
    let noteDate = $state(new Date().toISOString().slice(0, 10)); // yyyy-mm-dd

    function addNote() {
        const content = noteContent.trim();
        const title = noteTitle.trim();
        if (!content) return;
        const id =
            globalThis.crypto?.randomUUID?.() ??
            `${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
        const dateMs = new Date(noteDate).getTime();
        hrtData.data.notes = [
            { id, date: Number.isFinite(dateMs) ? dateMs : Date.now(), title, content },
            ...(hrtData.data.notes ?? []),
        ];
        noteTitle = "";
        noteContent = "";
        noteDate = new Date().toISOString().slice(0, 10);
    }

    function deleteNote(id: string) {
        hrtData.data.notes = (hrtData.data.notes ?? []).filter((n) => n.id !== id);
    }

    // Editing
    let editingId: string | null = $state(null);
    let editingTitle = $state("");
    let editingContent = $state("");
    let editingDate = $state(new Date().toISOString().slice(0, 10));

    function startEdit(note: DiaryEntry) {
        editingId = note.id;
        editingTitle = note.title ?? "";
        editingContent = note.content;
        editingDate = new Date(note.date).toISOString().slice(0, 10);
    }
    function cancelEdit() {
        editingId = null;
    }
    function saveEdit() {
        if (!editingId) return;
        const id = editingId;
        const dateMs = new Date(editingDate).getTime();
        hrtData.data.notes = (hrtData.data.notes ?? []).map((n) =>
            n.id === id
                ? {
                      ...n,
                      title: editingTitle.trim(),
                      content: editingContent.trim(),
                      date: Number.isFinite(dateMs) ? dateMs : n.date,
                  }
                : n,
        );
        editingId = null;
    }

    const sortedNotes = $derived([...(hrtData.data.notes ?? [])].sort((a, b) => b.date - a.date));

    function getLatestFudgeFactor(): number | null {
        const tests = hrtData.data.bloodTests;
        if (!tests || tests.length === 0) return null;
        const latest = [...tests]
            .filter(
                (t) =>
                    typeof t.estradiolLevel === "number" &&
                    typeof t.estrannaiseNumber === "number" &&
                    (t.estrannaiseNumber as number) > 0,
            )
            .sort((a, b) => b.date - a.date)[0];
        if (!latest) return null;
        const unit = latest.estradiolUnit || HormoneUnits.E2_pg_mL;
        const measured =
            unit === HormoneUnits.E2_pmol_L
                ? (latest.estradiolLevel as number) / 3.671
                : (latest.estradiolLevel as number);
        const predicted = latest.estrannaiseNumber as number;
        if (!isFinite(measured) || !isFinite(predicted) || predicted <= 0) return null;
        const ff = measured / predicted;
        if (!isFinite(ff) || ff <= 0) return null;
        return Number(ff.toFixed(3));
    }

    function generateEstrannaiseUrl(): string | null {
        const regimen = hrtData.data.injectableEstradiol;
        const historicalDoses = hrtData.data.dosageHistory
            .filter(
                (d): d is Extract<DosageHistoryEntry, { medicationType: "injectableEstradiol" }> =>
                    d.medicationType === "injectableEstradiol",
            )
            .sort((a, b) => a.date - b.date);

        if (historicalDoses.length === 0 && !regimen) {
            return null;
        }

        const allDoses: { date: number; type: InjectableEstradiols; dose: number }[] =
            historicalDoses.map((d) => ({
                date: d.date,
                type: d.type,
                dose: d.dose,
            }));

        let lastDoseDate: number;
        let totalDurationDays = 0;

        if (historicalDoses.length > 0) {
            const firstDoseDate = historicalDoses[0].date;
            lastDoseDate = historicalDoses[historicalDoses.length - 1].date;
            totalDurationDays = (lastDoseDate - firstDoseDate) / (1000 * 60 * 60 * 24);
        } else if (regimen) {
            // No history, but we have a regimen. Start from today.
            lastDoseDate = Date.now();
            // Add a dose for today to start the projection
            allDoses.push({
                date: lastDoseDate,
                type: regimen.type,
                dose: regimen.dose,
            });
        } else {
            return null; // Should be unreachable
        }

        if (regimen && totalDurationDays < 80) {
            const frequencyMs = regimen.frequency * 24 * 60 * 60 * 1000;
            let nextDoseDate = lastDoseDate + frequencyMs;

            while (totalDurationDays < 80) {
                allDoses.push({
                    date: nextDoseDate,
                    type: regimen.type,
                    dose: regimen.dose,
                });

                const firstDate = allDoses[0].date;
                totalDurationDays = (nextDoseDate - firstDate) / (1000 * 60 * 60 * 24);
                nextDoseDate += frequencyMs;
            }
        }

        if (allDoses.length === 0) {
            return null;
        }

        const doseStrings: string[] = [];
        let lastDateForInterval: number | null = null;

        for (const dose of allDoses) {
            let modelId: number | undefined;
            switch (dose.type) {
                case InjectableEstradiols.Valerate:
                    modelId = 1;
                    break;
                case InjectableEstradiols.Enanthate:
                    modelId = 2;
                    break;
                case InjectableEstradiols.Cypionate:
                    modelId = 3;
                    break;
                case InjectableEstradiols.Benzoate:
                    modelId = 0;
                    break;
                case InjectableEstradiols.Undecylate:
                    modelId = 4;
                    break;
            }

            if (modelId !== undefined) {
                let time: number;
                if (lastDateForInterval === null) {
                    time = 0;
                } else {
                    // time is interval in days
                    time = (dose.date - lastDateForInterval) / (1000 * 60 * 60 * 24);
                }
                lastDateForInterval = dose.date;

                doseStrings.push(`${dose.dose},${parseFloat(time.toFixed(3))},${modelId}`);
            }
        }

        if (doseStrings.length === 0) {
            return null;
        }

        const customDoseString = doseStrings
            .map((ds, i) => (i === 0 ? "cu," + ds : ds))
            .join("-");

        // stateString: i for interval days.
        const stateString = "i";

        const ff = getLatestFudgeFactor();
        const suffix = ff ? `__${ff}` : "_";
        return `https://estrannai.se/#${stateString}_${customDoseString}${suffix}`;
    }

    let fudgeFactor = $derived(getLatestFudgeFactor());
    let estrannaiseUrl = $derived(generateEstrannaiseUrl());

    const DAY_MS = 24 * 60 * 60 * 1000;
    function formatDateLabel(ms: number): string {
        if (xAxisMode === "days" && firstDoseDate !== null) {
            const days = (ms - firstDoseDate) / DAY_MS;
            return `Day ${days.toFixed(1)}`;
        }
        return new Date(ms).toLocaleDateString();
    }
    function formatDateForTooltip(d: Date): string {
        if (xAxisMode === "days" && firstDoseDate !== null) {
            const days = (d.getTime() - firstDoseDate) / DAY_MS;
            return `Day ${days.toFixed(1)}`;
        }
        return d.toLocaleDateString();
    }
    type RegimenKey = 'injectableEstradiol' | 'oralEstradiol' | 'antiandrogen' | 'progesterone';

    let hasAnyRegimen = $derived(
        Boolean(
            hrtData.data.injectableEstradiol ||
            hrtData.data.oralEstradiol ||
            hrtData.data.antiandrogen ||
            hrtData.data.progesterone
        )
    );

    function getLastDoseDateForType(medType: RegimenKey): number | null {
        const recs = (hrtData.data.dosageHistory || []).filter((d) => d.medicationType === medType);
        if (recs.length === 0) return null;
        return Math.max(...recs.map((d) => d.date));
    }

    function getNextScheduledDateFor(medType: RegimenKey): number | null {
        const now = Date.now();
        switch (medType) {
            case 'injectableEstradiol': {
                const cfg = hrtData.data.injectableEstradiol;
                if (!cfg) return null;
                const last = getLastDoseDateForType('injectableEstradiol');
                if (typeof cfg.nextDoseDate === 'number' && (!last || cfg.nextDoseDate > last)) {
                    return cfg.nextDoseDate;
                }
                if (typeof last === 'number') {
                    return last + cfg.frequency * DAY_MS;
                }
                return now;
            }
            case 'oralEstradiol': {
                const cfg = hrtData.data.oralEstradiol;
                if (!cfg) return null;
                const last = getLastDoseDateForType('oralEstradiol');
                if (typeof last === 'number') {
                    return last + cfg.frequency * DAY_MS;
                }
                return now;
            }
            case 'antiandrogen': {
                const cfg = hrtData.data.antiandrogen;
                if (!cfg) return null;
                const last = getLastDoseDateForType('antiandrogen');
                if (typeof last === 'number') {
                    return last + cfg.frequency * DAY_MS;
                }
                return now;
            }
            case 'progesterone': {
                const cfg = hrtData.data.progesterone;
                if (!cfg) return null;
                const last = getLastDoseDateForType('progesterone');
                if (typeof last === 'number') {
                    return last + cfg.frequency * DAY_MS;
                }
                return now;
            }
        }
    }

    function getNextScheduledCandidate(): { medType: RegimenKey; label: string } | null {
        const options: { medType: RegimenKey; date: number; label: string }[] = [];
        const now = Date.now();

        const pushIf = (medType: RegimenKey, labelBuilder: () => string) => {
            const d = getNextScheduledDateFor(medType);
            if (typeof d === 'number') {
                options.push({ medType, date: d, label: labelBuilder() });
            }
        };

        if (hrtData.data.injectableEstradiol) {
            const cfg = hrtData.data.injectableEstradiol;
            pushIf('injectableEstradiol', () => `Injection: ${cfg.type}, ${cfg.dose} ${cfg.unit}`);
        }
        if (hrtData.data.oralEstradiol) {
            const cfg = hrtData.data.oralEstradiol;
            pushIf('oralEstradiol', () => `Oral Estradiol: ${cfg.type}, ${cfg.dose} ${cfg.unit}`);
        }
        if (hrtData.data.antiandrogen) {
            const cfg = hrtData.data.antiandrogen;
            pushIf('antiandrogen', () => `Antiandrogen: ${cfg.type}, ${cfg.dose} ${cfg.unit}`);
        }
        if (hrtData.data.progesterone) {
            const cfg = hrtData.data.progesterone as any;
            pushIf('progesterone', () => `Progesterone (${cfg.route}): ${cfg.type}, ${cfg.dose} ${cfg.unit}`);
        }

        if (options.length === 0) return null;

        const future = options.filter((o) => o.date >= now);
        if (future.length > 0) {
            future.sort((a, b) => a.date - b.date);
            return { medType: future[0].medType, label: future[0].label };
        }
        // If all options are in the past, pick the most recent one
        options.sort((a, b) => b.date - a.date);
        return { medType: options[0].medType, label: options[0].label };
    }

    let nextScheduledCandidate = $derived(getNextScheduledCandidate());

    async function recordNextDoseNow() {
        const c = getNextScheduledCandidate();
        if (!c) return;

        const now = Date.now();
        switch (c.medType) {
            case 'injectableEstradiol': {
                const cfg = hrtData.data.injectableEstradiol!;
                const rec: DosageHistoryEntry = {
                    date: now,
                    medicationType: 'injectableEstradiol',
                    type: cfg.type,
                    dose: cfg.dose,
                    unit: cfg.unit,
                    vialId: cfg.vialId,           // add
                    subVialId: cfg.subVialId,     // add
                    syringeKind: (cfg as any).syringeKind,   // add
                    needleLength: (cfg as any).needleLength, // add
                } as any;
                hrtData.addDosageRecord(rec);
                if (typeof cfg.frequency === 'number' && isFinite(cfg.frequency) && cfg.frequency > 0) {
                    cfg.nextDoseDate = now + cfg.frequency * DAY_MS;
                }
                break;
            }
            case 'oralEstradiol': {
                const cfg = hrtData.data.oralEstradiol!;
                const rec: DosageHistoryEntry = {
                    date: now,
                    medicationType: 'oralEstradiol',
                    type: cfg.type,
                    dose: cfg.dose,
                    unit: cfg.unit,
                } as any;
                hrtData.addDosageRecord(rec);
                break;
            }
            case 'antiandrogen': {
                const cfg = hrtData.data.antiandrogen!;
                const rec: DosageHistoryEntry = {
                    date: now,
                    medicationType: 'antiandrogen',
                    type: cfg.type,
                    dose: cfg.dose,
                    unit: cfg.unit,
                } as any;
                hrtData.addDosageRecord(rec);
                break;
            }
            case 'progesterone': {
                const cfg = hrtData.data.progesterone as any;
                const rec: DosageHistoryEntry = {
                    date: now,
                    medicationType: 'progesterone',
                    type: cfg.type,
                    dose: cfg.dose,
                    unit: cfg.unit,
                    route: cfg.route,
                } as any;
                hrtData.addDosageRecord(rec);
                break;
            }
        }
        try {
            await hrtData.saveNow();
        } catch {
            // ignore persistence errors for UX; ICS will catch up on next save attempt
        }
    }

    let daysSinceFirstDose: number | null = $state(null);
    let firstDoseDate: number | null = $state(null);
    let xAxisMode = $state<"date" | "days">("date");

    $effect(() => {
        const dosageHistory = hrtData.data.dosageHistory;
        if (!dosageHistory || dosageHistory.length === 0) {
            daysSinceFirstDose = null;
            firstDoseDate = null;
            return;
        }

        const firstDate = Math.min(...dosageHistory.map((d) => d.date));
        firstDoseDate = firstDate;
        const now = Date.now();
        const diffTime = Math.abs(now - firstDate);
        const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

        daysSinceFirstDose = diffDays;
    });

    let itemToEdit: BloodTest | DosageHistoryEntry | Measurement | null = $state(null);

    function onEdit(item: BloodTest | DosageHistoryEntry | Measurement) {
        itemToEdit = item;
    }

    function onCloseModal() {
        itemToEdit = null;
        renderChart();
        hrtData.saveNow().catch(() => {});
    }

    // Chart related code
    let chartDiv: HTMLElement | undefined;
    let timeRangeInDays = $state(90); // Default to showing last 90 days
    let showMedications = $state(true); // Filter for medication data

    // Hormone visibility toggles
    let showE2 = $state(true);
    let showT = $state(true);
    let showProg = $state(false);
    let showFSH = $state(false);
    let showLH = $state(false);
    let showProlactin = $state(false);
    let showSHBG = $state(false);
    let showFAI = $state(false);

    // Process data for charting
    function processDataForChart() {
        const now = Date.now();
        const startTime = now - timeRangeInDays * 24 * 60 * 60 * 1000;

        // Filter blood tests based on time range and filter setting
        const filteredBloodTests = hrtData.data.bloodTests
            .filter((test) => test.date >= startTime)
            .map((test) => {
                // Raw units with sensible defaults
                const estradiolUnitRaw = test.estradiolUnit || HormoneUnits.E2_pg_mL;
                const testUnitRaw = test.testUnit || HormoneUnits.T_ng_dL;
                const progesteroneUnitRaw = test.progesteroneUnit || HormoneUnits.ng_mL;
                const fshUnitRaw = test.fshUnit || HormoneUnits.mIU_mL;
                const lhUnitRaw = test.lhUnit || HormoneUnits.mIU_mL;
                const prolactinUnitRaw = test.prolactinUnit || HormoneUnits.ng_mL;
                const shbgUnitRaw = test.shbgUnit || HormoneUnits.T_nmol_L;

                // Normalized values for plotting
                const estradiolLevelPlot =
                    test.estradiolLevel !== undefined
                        ? estradiolUnitRaw === HormoneUnits.E2_pmol_L
                            ? Number((test.estradiolLevel / 3.671).toFixed(2)) // pmol/L -> pg/mL
                            : test.estradiolLevel
                        : undefined;

                const testLevelPlot =
                    test.testLevel !== undefined
                        ? testUnitRaw === HormoneUnits.T_nmol_L
                            ? Number((test.testLevel * 28.818).toFixed(2)) // nmol/L -> ng/dL
                            : test.testLevel
                        : undefined;

                const fshLevelPlot =
                    test.fshLevel !== undefined
                        ? fshUnitRaw === HormoneUnits.mIU_L
                            ? Number((test.fshLevel / 1000).toFixed(3)) // mIU/L -> mIU/mL
                            : fshUnitRaw === HormoneUnits.U_L
                            ? test.fshLevel // IU/L == mIU/mL numerically
                            : test.fshLevel
                        : undefined;

                const lhLevelPlot =
                    test.lhLevel !== undefined
                        ? lhUnitRaw === HormoneUnits.mIU_L
                            ? Number((test.lhLevel / 1000).toFixed(3)) // mIU/L -> mIU/mL
                            : lhUnitRaw === HormoneUnits.U_L
                            ? test.lhLevel // IU/L == mIU/mL numerically
                            : test.lhLevel
                        : undefined;

                const progesteroneLevelPlot =
                    test.progesteroneLevel !== undefined
                        ? progesteroneUnitRaw === HormoneUnits.T_nmol_L
                            ? Number((test.progesteroneLevel * 0.31).toFixed(2)) // nmol/L -> ng/mL (0.31 ng/mL per 1 nmol/L)
                            : test.progesteroneLevel
                        : undefined;
                const prolactinLevelPlot = test.prolactinLevel;
                const shbgLevelPlot = test.shbgLevel;

                return {
                    date: new Date(test.date),
                    xDays: firstDoseDate !== null ? (test.date - firstDoseDate) / (1000 * 60 * 60 * 24) : undefined,
                    type: "Blood Test",

                    // Raw values and units (for tooltips)
                    estradiolLevel: test.estradiolLevel,
                    testLevel: test.testLevel,
                    progesteroneLevel: test.progesteroneLevel,
                    fshLevel: test.fshLevel,
                    lhLevel: test.lhLevel,
                    prolactinLevel: test.prolactinLevel,
                    shbgLevel: test.shbgLevel,
                    freeAndrogenIndex: test.freeAndrogenIndex,
                    estradiolUnit: estradiolUnitRaw,
                    testUnit: testUnitRaw,
                    progesteroneUnit: progesteroneUnitRaw,
                    fshUnit: fshUnitRaw,
                    lhUnit: lhUnitRaw,
                    prolactinUnit: prolactinUnitRaw,
                    shbgUnit: shbgUnitRaw,

                    // Normalized values for plotting (standard units)
                    // E2: pg/mL, T: ng/dL, Prog: ng/mL, FSH/LH: mIU/mL, PRL: ng/mL, SHBG: nmol/L
                    estradiolLevelPlot,
                    testLevelPlot,
                    progesteroneLevelPlot,
                    fshLevelPlot,
                    lhLevelPlot,
                    prolactinLevelPlot,
                    shbgLevelPlot,
                };
            });

        // Filter dosages based on time range and filter setting
        const filteredDosages = showMedications
            ? hrtData.data.dosageHistory
                  .filter((dose) => dose.date >= startTime)
                  .map((dose) => ({
                      date: new Date(dose.date),
                      xDays: firstDoseDate !== null ? (dose.date - firstDoseDate) / (1000 * 60 * 60 * 24) : undefined,
                      type: dose.medicationType,
                      name: dose.type,
                      dose: dose.dose,
                      unit: dose.unit,
                  }))
            : [];

        // Detect the units being used
        const estradiolUnits = [
            ...new Set(
                filteredBloodTests
                    .filter((test) => test.estradiolLevel !== undefined)
                    .map((test) => test.estradiolUnit),
            ),
        ];

        const testUnits = [
            ...new Set(
                filteredBloodTests
                    .filter((test) => test.testLevel !== undefined)
                    .map((test) => test.testUnit),
            ),
        ];

        return {
            bloodTests: filteredBloodTests,
            dosages: filteredDosages,
            estradiolUnits,
            testUnits,
        };
    }

    // Function to redraw chart when window resizes
    function redrawChart() {
        if (!chartDiv) return;
        renderChart();
    }

    // Add window resize listener
    $effect(() => {
        if (!chartDiv) return;

        window.addEventListener("resize", redrawChart);
        return () => window.removeEventListener("resize", redrawChart);
    });

    // Function to render the chart
    function renderChart() {
        if (!chartDiv) return;

        chartDiv.firstChild?.remove(); // Remove old chart

        const { bloodTests, dosages } = processDataForChart();

        const useDaysAxis = xAxisMode === "days" && firstDoseDate !== null;
        const xKey: "date" | "xDays" = useDaysAxis ? "xDays" : "date";

        // Axis grouping and domains for dual-axis support
        const leftKeys: string[] = [];
        if (showE2) leftKeys.push("estradiolLevelPlot");
        if (showProg) leftKeys.push("progesteroneLevelPlot");
        if (showFSH) leftKeys.push("fshLevelPlot");
        if (showLH) leftKeys.push("lhLevelPlot");
        if (showProlactin) leftKeys.push("prolactinLevelPlot");
        if (showFAI) leftKeys.push("freeAndrogenIndex"); // include for domain when visible
        if (showT) leftKeys.push("testLevelPlot");
        if (showSHBG) leftKeys.push("shbgLevelPlot");

        const extractValues = (keys: string[]) =>
            keys.flatMap((k) =>
                bloodTests
                    .map((d: any) => d[k])
                    .filter((v: any) => typeof v === "number" && isFinite(v) && v > 0),
            ) as number[];

        const leftVals = extractValues(leftKeys);

        let yLeftMin = leftVals.length ? Math.min(...leftVals) : 0;
        let yLeftMax = leftVals.length ? Math.max(...leftVals) : 1;
        if (yLeftMin === yLeftMax) {
            yLeftMin = Math.max(0, yLeftMin - 1);
            yLeftMax = yLeftMax + 1;
        } else {
            const pad = 0.08 * (yLeftMax - yLeftMin);
            yLeftMin = Math.max(0, yLeftMin - pad);
            yLeftMax = yLeftMax + pad;
        }


        // Only create chart if we have data
        if (bloodTests.length === 0 && dosages.length === 0) {
            chartDiv.textContent =
                "No data available for the selected time range";
            return;
        }

        // Helper to create hormone plot marks
        const createHormoneMarks = (
            data: any[],
            valuePlotKey: string,
            valueRawKey: string,
            unitRawKey: string,
            normalizedUnit: string,
            color: string,
            label: string,
            xKey: "date" | "xDays",
        ) => {
            if (!data.some((d) => d[valuePlotKey] !== undefined && d[valuePlotKey] > 0)) return [];
            return [
                Plot.line(
                    data.filter((d) => d[valuePlotKey] !== undefined && d[valuePlotKey] > 0),
                    {
                        x: xKey,
                        y: valuePlotKey,
                        stroke: color,
                        strokeWidth: 2,
                        curve: "monotone-x",
                    },
                ),
                Plot.dot(
                    data.filter((d) => d[valuePlotKey] !== undefined && d[valuePlotKey] > 0),
                    {
                        x: xKey,
                        y: valuePlotKey,
                        fill: color,
                        r: 5,
                        title: (d: any) => {
                            const rawVal = d[valueRawKey];
                            const rawUnit = d[unitRawKey];
                            const plotVal = d[valuePlotKey];
                            const showRaw =
                                rawVal !== undefined &&
                                rawUnit !== undefined &&
                                `${normalizedUnit}` !== `${rawUnit}`;
                            const dayPrefix =
                                xKey === "xDays" && typeof d.xDays === "number"
                                    ? `Day ${d.xDays.toFixed(1)} – `
                                    : "";
                            const dateLabel =
                                d.date && typeof d.date.toLocaleDateString === "function"
                                    ? d.date.toLocaleDateString()
                                    : "";
                            return showRaw
                                ? `${label}: ${rawVal} ${rawUnit} → ${plotVal} ${normalizedUnit} (${dayPrefix}${dateLabel})`
                                : `${label}: ${plotVal} ${normalizedUnit} (${dayPrefix}${dateLabel})`;
                        },
                    },
                ),
            ];
        };

        const createFAIMarks = (data: any[], xKey: "date" | "xDays") => {
            if (
                !data.some(
                    (d) => d.freeAndrogenIndex !== undefined && d.freeAndrogenIndex > 0,
                )
            )
                return [];
            return [
                Plot.line(
                    data.filter(
                        (d) => d.freeAndrogenIndex !== undefined && d.freeAndrogenIndex > 0,
                    ),
                    {
                        x: xKey,
                        y: "freeAndrogenIndex",
                        stroke: "black",
                        strokeWidth: 2,
                        curve: "monotone-x",
                    },
                ),
                Plot.dot(
                    data.filter(
                        (d) => d.freeAndrogenIndex !== undefined && d.freeAndrogenIndex > 0,
                    ),
                    {
                        x: xKey,
                        y: "freeAndrogenIndex",
                        fill: "black",
                        r: 5,
                        title: (d: any) => {
                            const dayPrefix =
                                xKey === "xDays" && typeof d.xDays === "number"
                                    ? `Day ${d.xDays.toFixed(1)} – `
                                    : "";
                            const dateLabel =
                                d.date && typeof d.date.toLocaleDateString === "function"
                                    ? d.date.toLocaleDateString()
                                    : "";
                            return `FAI: ${d.freeAndrogenIndex} (${dayPrefix}${dateLabel})`;
                        },
                    },
                ),
            ];
        };

        // Get container width for responsive sizing
        const containerWidth = chartDiv.clientWidth || window.innerWidth - 50;
        const chart = Plot.plot({
            title: "Hormone Levels and Dosages Over Time",
            width: Math.max(300, containerWidth - 20), // Ensure minimum width but adapt to container
            height: Math.min(500, Math.max(300, containerWidth * 0.5)), // Responsive height
            marginLeft: 60,
            marginRight: 60,
            marginBottom: 60,
            grid: true,
            x: useDaysAxis
                ? {
                      label: "Days since first dose",
                      type: "linear",
                  }
                : {
                      label: "Date",
                      type: "utc",
                  },
            y: {
                label: "Levels",
                grid: true,
                domain: [yLeftMin, yLeftMax],
            },
            color: {
                legend: true,
            },
            marks: [
                ...[],
                ...(showE2
                    ? createHormoneMarks(
                          bloodTests,
                          "estradiolLevelPlot",
                          "estradiolLevel",
                          "estradiolUnit",
                          "pg/mL",
                          "steelblue",
                          "Estradiol",
                          xKey,
                      )
                    : []),
                ...(showT
                    ? createHormoneMarks(
                          bloodTests,
                          "testLevelPlot",
                          "testLevel",
                          "testUnit",
                          "ng/dL",
                          "orangered",
                          "Testosterone",
                          xKey,
                      )
                    : []),
                ...(showProg
                    ? createHormoneMarks(
                          bloodTests,
                          "progesteroneLevelPlot",
                          "progesteroneLevel",
                          "progesteroneUnit",
                          "ng/mL",
                          "darkviolet",
                          "Progesterone",
                          xKey,
                      )
                    : []),
                ...(showFSH
                    ? createHormoneMarks(
                          bloodTests,
                          "fshLevelPlot",
                          "fshLevel",
                          "fshUnit",
                          "mIU/mL",
                          "forestgreen",
                          "FSH",
                          xKey,
                      )
                    : []),
                ...(showLH
                    ? createHormoneMarks(
                          bloodTests,
                          "lhLevelPlot",
                          "lhLevel",
                          "lhUnit",
                          "mIU/mL",
                          "darkcyan",
                          "LH",
                          xKey,
                      )
                    : []),
                ...(showProlactin
                    ? createHormoneMarks(
                          bloodTests,
                          "prolactinLevelPlot",
                          "prolactinLevel",
                          "prolactinUnit",
                          "ng/mL",
                          "saddlebrown",
                          "Prolactin",
                          xKey,
                      )
                    : []),
                ...(showSHBG
                    ? createHormoneMarks(
                          bloodTests,
                          "shbgLevelPlot",
                          "shbgLevel",
                          "shbgUnit",
                          "nmol/L",
                          "deeppink",
                          "SHBG",
                          xKey,
                      )
                    : []),
                ...(showFAI ? createFAIMarks(bloodTests, xKey) : []),

                // Medication dosages
                ...(showMedications &&
                dosages.some((d) => d.type === "injectableEstradiol")
                    ? [
                          Plot.dot(
                              dosages.filter(
                                  (d) => d.type === "injectableEstradiol",
                              ),
                              {
                                  x: xKey,
                                  y: (d) => Math.min(d.dose * 10, 200), // Scale for visibility
                                  fill: "limegreen",
                                  symbol: "triangle",
                                  r: 8,
                                  title: (d) =>
                                      `Injection: ${d.name}, ${d.dose} ${d.unit || "mg"} (${formatDateForTooltip(d.date)})`,
                              },
                          ),
                      ]
                    : []),
                ...(showMedications &&
                dosages.some((d) => d.type === "oralEstradiol")
                    ? [
                          Plot.dot(
                              dosages.filter((d) => d.type === "oralEstradiol"),
                              {
                                  x: xKey,
                                  y: (d) => Math.min(d.dose * 10, 200),
                                  fill: "blueviolet",
                                  symbol: "square",
                                  r: 7,
                                  title: (d) =>
                                      `Oral E: ${d.name}, ${d.dose} ${d.unit || "mg"} (${formatDateForTooltip(d.date)})`,
                              },
                          ),
                      ]
                    : []),
                ...(showMedications &&
                dosages.some((d) => d.type === "antiandrogen")
                    ? [
                          Plot.dot(
                              dosages.filter((d) => d.type === "antiandrogen"),
                              {
                                  x: xKey,
                                  y: (d) => Math.min(d.dose * 10, 200),
                                  fill: "darkorange",
                                  symbol: "diamond",
                                  r: 7,
                                  title: (d) =>
                                      `AA: ${d.name}, ${d.dose} ${d.unit || "mg"} (${formatDateForTooltip(d.date)})`,
                              },
                          ),
                      ]
                    : []),
                ...(showMedications &&
                dosages.some((d) => d.type === "progesterone")
                    ? [
                          Plot.dot(
                              dosages.filter(
                                  (d) => d.type === "progesterone",
                              ),
                              {
                                  x: xKey,
                                  y: (d) => Math.min(d.dose, 400), // Prog doses are high
                                  fill: "gold",
                                  symbol: "hexagon",
                                  r: 7,
                                  title: (d) =>
                                      `Progesterone: ${d.name}, ${d.dose} ${d.unit || "mg"} (${formatDateForTooltip(d.date)})`,
                              },
                          ),
                      ]
                    : []),
            ],
        });

        chartDiv.append(chart);
    }

    $effect(() => {
        // Rerender chart when inputs or data change
        timeRangeInDays;
        showMedications;
        showE2; showT; showProg; showFSH; showLH; showProlactin; showSHBG; showFAI;
        xAxisMode; firstDoseDate;
        hrtData.data.bloodTests;
        hrtData.data.dosageHistory;
        renderChart();
    });

    function updateTimeRange(days: number) {
        timeRangeInDays = days;
    }

    function toggleMedications() {
        showMedications = !showMedications;
    }
</script>

{#if itemToEdit}
    <EditModal item={itemToEdit} close={onCloseModal} />
{/if}

<div class="flex justify-between items-center px-4 pt-4">
    <h1 class="text-3xl font-bold mb-2">HRT Tracking Data</h1>
    <a href="/create/measurement" class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors">Add Measurement</a>
</div>
<p class="mb-4 px-4 text-sm opacity-75">
    This chart shows your hormone levels from blood tests along with your dosage
    history over time.
</p>
<div class="flex flex-col p-4 w-full max-w-[100vw]">
    <div
        class="mb-4 border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md"
    >
        <div class="flex justify-between items-center mb-2">
            <h2 class="text-xl font-medium">Current Regimen</h2>
            <div class="flex gap-2 items-center">
                {#if estrannaiseUrl}
                    <a
                        href={estrannaiseUrl}
                        target="_blank"
                        rel="noopener noreferrer"
                        class="px-3 py-1 text-sm rounded bg-latte-rose-pine-iris text-white hover:bg-rose-pine-pine transition-colors"
                        >View on Estrannaise</a
                    >
                {/if}
                <button
                    class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    onclick={recordNextDoseNow}
                    disabled={!hasAnyRegimen}
                    title={nextScheduledCandidate ? `Record: ${nextScheduledCandidate.label}` : ''}
                >
                    Record next dose now
                </button>
                <a
                    href="/create/dosage?mode=schedule"
                    class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors"
                    >Edit Schedule</a
                >
            </div>
        </div>
        <div class="space-y-1 text-sm">
            {#if daysSinceFirstDose !== null}
                <p>
                    <strong>Days since first dose:</strong> {daysSinceFirstDose}
                </p>
            {/if}
            {#if fudgeFactor !== null}
                <p>
                    <strong>Estrannaise fudge factor:</strong> {fudgeFactor}
                </p>
            {/if}
            {#if hrtData.data.injectableEstradiol}
                <p>
                    <strong>Injectable Estradiol:</strong>
                    {hrtData.data.injectableEstradiol.type}, {hrtData.data.injectableEstradiol.dose}
                    {hrtData.data.injectableEstradiol.unit} every {hrtData.data.injectableEstradiol.frequency} days
                </p>
            {/if}
            {#if hrtData.data.oralEstradiol}
                <p>
                    <strong>Oral Estradiol:</strong>
                    {hrtData.data.oralEstradiol.type}, {hrtData.data.oralEstradiol.dose}
                    {hrtData.data.oralEstradiol.unit} every {hrtData.data.oralEstradiol.frequency} {hrtData.data.oralEstradiol.frequency === 1 ? 'day' : 'days'}
                </p>
            {/if}
            {#if hrtData.data.antiandrogen}
                <p>
                    <strong>Antiandrogen:</strong>
                    {hrtData.data.antiandrogen.type}, {hrtData.data.antiandrogen.dose}
                    {hrtData.data.antiandrogen.unit} every {hrtData.data.antiandrogen.frequency} {hrtData.data.antiandrogen.frequency === 1 ? 'day' : 'days'}
                </p>
            {/if}
            {#if hrtData.data.progesterone}
                <p>
                    <strong>Progesterone:</strong>
                    {hrtData.data.progesterone.type} ({hrtData.data.progesterone.route}), {hrtData.data.progesterone.dose}
                    {hrtData.data.progesterone.unit} every {hrtData.data.progesterone.frequency} {hrtData.data.progesterone.frequency === 1 ? 'day' : 'days'}
                </p>
            {/if}
            {#if !hrtData.data.injectableEstradiol && !hrtData.data.oralEstradiol && !hrtData.data.antiandrogen && !hrtData.data.progesterone}
                <p class="italic text-gray-500 dark:text-gray-400">
                    No regimen set up. You can set one on the dosage page.
                </p>
            {/if}
        </div>
    </div>
    <div class="mb-4 flex flex-wrap gap-2">
        <div class="flex gap-2">
            <span class="self-center text-sm">X-Axis:</span>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={xAxisMode === "date"}
                class:dark:bg-rose-pine-iris={xAxisMode === "date"}
                class:text-latte-rose-pine-base={xAxisMode === "date"}
                class:dark:text-rose-pine-base={xAxisMode === "date"}
                onclick={() => (xAxisMode = "date")}
            >
                Date
            </button>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay disabled:opacity-50 disabled:cursor-not-allowed"
                class:bg-latte-rose-pine-iris={xAxisMode === "days"}
                class:dark:bg-rose-pine-iris={xAxisMode === "days"}
                class:text-latte-rose-pine-base={xAxisMode === "days"}
                class:dark:text-rose-pine-base={xAxisMode === "days"}
                onclick={() => (xAxisMode = "days")}
                disabled={firstDoseDate === null}
            >
                Days since first dose
            </button>
        </div>
        <div class="ml-auto flex gap-2">
            <span class="self-center text-sm">Time Range:</span>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={timeRangeInDays === 30}
                class:dark:bg-rose-pine-iris={timeRangeInDays === 30}
                class:text-latte-rose-pine-base={timeRangeInDays === 30}
                class:dark:text-rose-pine-base={timeRangeInDays === 30}
                onclick={() => updateTimeRange(30)}
            >
                30 days
            </button>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={timeRangeInDays === 90}
                class:dark:bg-rose-pine-iris={timeRangeInDays === 90}
                class:text-latte-rose-pine-base={timeRangeInDays === 90}
                class:dark:text-rose-pine-base={timeRangeInDays === 90}
                onclick={() => updateTimeRange(90)}
            >
                90 days
            </button>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={timeRangeInDays === 180}
                class:dark:bg-rose-pine-iris={timeRangeInDays === 180}
                class:text-latte-rose-pine-base={timeRangeInDays === 180}
                class:dark:text-rose-pine-base={timeRangeInDays === 180}
                onclick={() => updateTimeRange(180)}
            >
                180 days
            </button>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={timeRangeInDays === 365}
                class:dark:bg-rose-pine-iris={timeRangeInDays === 365}
                class:text-latte-rose-pine-base={timeRangeInDays === 365}
                class:dark:text-rose-pine-base={timeRangeInDays === 365}
                onclick={() => updateTimeRange(365)}
            >
                1 year
            </button>
            <button
                class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                class:bg-latte-rose-pine-iris={timeRangeInDays === 9999}
                class:dark:bg-rose-pine-iris={timeRangeInDays === 9999}
                class:text-latte-rose-pine-base={timeRangeInDays === 9999}
                class:dark:text-rose-pine-base={timeRangeInDays === 9999}
                onclick={() => updateTimeRange(9999)}
            >
                All
            </button>
        </div>
    </div>

    <div class="mb-4 flex flex-wrap gap-2">
        <span class="self-center text-sm">Show Levels:</span>
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showE2}
            class:dark:bg-rose-pine-iris={showE2}
            class:text-latte-rose-pine-base={showE2}
            class:dark:text-rose-pine-base={showE2}
            onclick={() => (showE2 = !showE2)}>E2</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showT}
            class:dark:bg-rose-pine-iris={showT}
            class:text-latte-rose-pine-base={showT}
            class:dark:text-rose-pine-base={showT}
            onclick={() => (showT = !showT)}>T</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showProg}
            class:dark:bg-rose-pine-iris={showProg}
            class:text-latte-rose-pine-base={showProg}
            class:dark:text-rose-pine-base={showProg}
            onclick={() => (showProg = !showProg)}>Prog</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showFSH}
            class:dark:bg-rose-pine-iris={showFSH}
            class:text-latte-rose-pine-base={showFSH}
            class:dark:text-rose-pine-base={showFSH}
            onclick={() => (showFSH = !showFSH)}>FSH</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showLH}
            class:dark:bg-rose-pine-iris={showLH}
            class:text-latte-rose-pine-base={showLH}
            class:dark:text-rose-pine-base={showLH}
            onclick={() => (showLH = !showLH)}>LH</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showProlactin}
            class:dark:bg-rose-pine-iris={showProlactin}
            class:text-latte-rose-pine-base={showProlactin}
            class:dark:text-rose-pine-base={showProlactin}
            onclick={() => (showProlactin = !showProlactin)}>Prolactin</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showSHBG}
            class:dark:bg-rose-pine-iris={showSHBG}
            class:text-latte-rose-pine-base={showSHBG}
            class:dark:text-rose-pine-base={showSHBG}
            onclick={() => (showSHBG = !showSHBG)}>SHBG</button
        >
        <button
            class="px-3 py-1 text-sm transition-colors rounded"
            class:bg-latte-rose-pine-iris={showFAI}
            class:dark:bg-rose-pine-iris={showFAI}
            class:text-latte-rose-pine-base={showFAI}
            class:dark:text-rose-pine-base={showFAI}
            onclick={() => (showFAI = !showFAI)}>FAI</button
        >
    </div>
    <div class="mb-4 flex flex-wrap gap-3">
        <span class="self-center text-sm">Show Dosages:</span>
        <button
            class="px-3 py-1 text-sm transition-colors rounded hover:bg-latte-rose-pine-overlay dark:hover:bg-rose-pine-overlay hover:text-latte-rose-pine-text dark:hover:text-rose-pine-text"
            class:bg-latte-rose-pine-iris={showMedications}
            class:dark:bg-rose-pine-iris={showMedications}
            class:text-latte-rose-pine-base={showMedications}
            class:dark:text-rose-pine-base={showMedications}
            onclick={toggleMedications}
        >
            {showMedications ? "✓" : ""} Medication Dosages
        </button>
    </div>

    <div
        class="mb-4 border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md w-full"
    >
        <div
            bind:this={chartDiv}
            class="w-full min-w-0 overflow-x-auto"
            role="img"
        ></div>
        <div class="mt-4 text-sm text-gray-500 dark:text-gray-400 italic">
            <p>* Dosage values are scaled for visibility on the chart.</p>
            <p>* Hover over data points for details.</p>
            {#if hrtData.data.bloodTests.length > 0}
                <p>* Hormone measurements are normalized to standard units for charting; hover shows recorded units.</p>
            {/if}
        </div>
    </div>

    <div
        class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md w-full mb-4"
    >
        <h2 class="text-xl font-medium mb-2">Diary / Notes</h2>
        <div class="space-y-2">
            <div class="flex flex-wrap gap-2">
                <input
                    type="date"
                    class="border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                    bind:value={noteDate}
                    aria-label="Note date"
                />
                <input
                    type="text"
                    class="flex-1 min-w-0 border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                    placeholder="Title (optional)"
                    bind:value={noteTitle}
                    aria-label="Note title"
                />
            </div>
            <textarea
                class="w-full border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                rows="3"
                placeholder="Write a note..."
                bind:value={noteContent}
                aria-label="Note content"
            ></textarea>
            <div class="flex justify-end">
                <button
                    class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    onclick={addNote}
                    disabled={!noteContent.trim()}
                >
                    Add Note
                </button>
            </div>
        </div>

        <div class="mt-4">
            {#if sortedNotes.length === 0}
                <p class="text-gray-500 dark:text-gray-400 italic">
                    No notes yet.
                </p>
            {:else}
                <ul class="space-y-2 max-h-60 overflow-y-auto">
                    {#each sortedNotes as n (n.id)}
                        <li class="p-2 border rounded">
                            {#if editingId === n.id}
                                <div class="flex flex-wrap gap-2 mb-2">
                                    <input
                                        type="date"
                                        class="border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                                        bind:value={editingDate}
                                        aria-label="Edit note date"
                                    />
                                    <input
                                        type="text"
                                        class="flex-1 min-w-0 border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                                        placeholder="Title (optional)"
                                        bind:value={editingTitle}
                                        aria-label="Edit note title"
                                    />
                                </div>
                                <textarea
                                    class="w-full border rounded px-2 py-1 bg-white dark:bg-rose-pine-base text-latte-rose-pine-text dark:text-rose-pine-text"
                                    rows="4"
                                    bind:value={editingContent}
                                    aria-label="Edit note content"
                                ></textarea>
                                <div class="flex gap-2 justify-end mt-2">
                                    <button
                                        class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors"
                                        onclick={saveEdit}
                                    >
                                        Save
                                    </button>
                                    <button
                                        class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                                        onclick={cancelEdit}
                                    >
                                        Cancel
                                    </button>
                                </div>
                            {:else}
                                <div class="flex justify-between items-start gap-3">
                                    <div class="min-w-0">
                                        <div class="font-medium">
                                            {formatDateLabel(n.date)}
                                        </div>
                                        {#if n.title}
                                            <div class="text-sm opacity-80 break-words">{n.title}</div>
                                        {/if}
                                        <div class="mt-1 whitespace-pre-wrap text-sm break-words">
                                            {n.content}
                                        </div>
                                    </div>
                                    <div class="flex gap-2 shrink-0">
                                        <button
                                            class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors"
                                            onclick={() => startEdit(n)}
                                        >
                                            Edit
                                        </button>
                                        <button
                                            class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-rose-pine-overlay hover:bg-latte-rose-pine-overlay"
                                            onclick={() => deleteNote(n.id)}
                                        >
                                            Delete
                                        </button>
                                    </div>
                                </div>
                            {/if}
                        </li>
                    {/each}
                </ul>
            {/if}
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div
            class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md"
        >
            <h2 class="text-xl font-medium mb-2">Blood Test History</h2>
            <div class="max-h-60 overflow-y-auto">
                {#if hrtData.data.bloodTests.length === 0}
                    <p class="text-gray-500 dark:text-gray-400 italic">
                        No blood test data available.
                    </p>
                {:else}
                    <ul class="space-y-2">
                        {#each [...hrtData.data.bloodTests].sort((a, b) => b.date - a.date) as t}
                            <li
                                class="p-2 border rounded flex justify-between items-center"
                            >
                                <div>
                                    <div class="font-medium">
                                        {formatDateLabel(t.date)}
                                    </div>
                                    <div
                                        class="text-sm flex flex-wrap gap-x-2 gap-y-1"
                                    >
                                        {#if t.estradiolLevel !== undefined}
                                            <span
                                                >E2: {t.estradiolLevel}
                                                {t.estradiolUnit || "pg/mL"}</span
                                            >
                                        {/if}
                                        {#if t.testLevel !== undefined}
                                            <span
                                                >T: {t.testLevel}
                                                {t.testUnit || "ng/dL"}</span
                                            >
                                        {/if}
                                        {#if t.progesteroneLevel !== undefined}
                                            <span
                                                >Prog: {t.progesteroneLevel}
                                                {t.progesteroneUnit || "ng/mL"}</span
                                            >
                                        {/if}
                                        {#if t.fshLevel !== undefined}
                                            <span
                                                >FSH: {t.fshLevel}
                                                {t.fshUnit || "mIU/mL"}</span
                                            >
                                        {/if}
                                        {#if t.lhLevel !== undefined}
                                            <span
                                                >LH: {t.lhLevel}
                                                {t.lhUnit || "mIU/mL"}</span
                                            >
                                        {/if}
                                        {#if t.prolactinLevel !== undefined}
                                            <span
                                                >PRL: {t.prolactinLevel}
                                                {t.prolactinUnit || "ng/mL"}</span
                                            >
                                        {/if}
                                        {#if t.shbgLevel !== undefined}
                                            <span
                                                >SHBG: {t.shbgLevel}
                                                {t.shbgUnit || "nmol/L"}</span
                                            >
                                        {/if}
                                        {#if t.freeAndrogenIndex !== undefined}
                                            <span
                                                >FAI: {t.freeAndrogenIndex}</span
                                            >
                                        {/if}
                                    </div>
                                </div>
                                <button
                                    class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors"
                                    onclick={() => onEdit(t)}>Edit</button
                                >
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>

        <div
            class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md"
        >
            <h2 class="text-xl font-medium mb-2">Measurement History</h2>
            <div class="max-h-60 overflow-y-auto">
                {#if hrtData.data.measurements.length === 0}
                    <p class="text-gray-500 dark:text-gray-400 italic">
                        No measurement data available.
                    </p>
                {:else}
                    <ul class="space-y-2">
                        {#each [...hrtData.data.measurements].sort((a, b) => b.date - a.date) as m}
                            <li class="p-2 border rounded flex justify-between items-center">
                                <div>
                                    <div class="font-medium">
                                        {formatDateLabel(m.date)}
                                    </div>
                                    <div class="text-sm flex flex-wrap gap-x-2 gap-y-1">
                                        {#if m.weight}<span>Weight: {m.weight}{m.weightUnit}</span>{/if}
                                        {#if m.height}<span>Height: {m.height}{m.heightUnit}</span>{/if}
                                        {#if m.braSize}<span>Bra: {m.braSize}</span>{/if}
                                        {#if m.underbust}<span>Underbust: {m.underbust}{m.bodyMeasurementUnit}</span>{/if}
                                        {#if m.bust}<span>Bust: {m.bust}{m.bodyMeasurementUnit}</span>{/if}
                                        {#if m.waist}<span>Waist: {m.waist}{m.bodyMeasurementUnit}</span>{/if}
                                        {#if m.hip}<span>Hip: {m.hip}{m.bodyMeasurementUnit}</span>{/if}
                                        {#if m.bideltoid}<span>Shoulder: {m.bideltoid}{m.bodyMeasurementUnit}</span>{/if}
                                    </div>
                                </div>
                                <button class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors" onclick={() => onEdit(m)}>Edit</button>
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>
        <div
            class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md"
        >
            <h2 class="text-xl font-medium mb-2">Medication Dosage History</h2>
            <div class="max-h-60 overflow-y-auto">
                {#if hrtData.data.dosageHistory.length === 0}
                    <p class="text-gray-500 dark:text-gray-400 italic">
                        No dosage data available.
                    </p>
                {:else}
                    <ul class="space-y-2">
                        {#each [...hrtData.data.dosageHistory].sort((a, b) => b.date - a.date) as t}
                            <li
                                class="p-2 border rounded flex justify-between items-center"
                            >
                                <div class="flex-1">
                                    <div class="font-medium">
                                        {formatDateLabel(t.date)}
                                    </div>
                                    <div class="text-sm flex gap-2">
                                        <span class="capitalize"
                                            >{t.medicationType ===
                                            "injectableEstradiol"
                                                ? "Injection"
                                                : t.medicationType ===
                                                    "oralEstradiol"
                                                  ? "Oral E"
                                                  : t.medicationType ===
                                                      "progesterone"
                                                    ? `Progesterone (${(t as any).route})`
                                                    : "AA"}</span
                                        >
                                        <span>{t.type}</span>
                                        <span>{t.dose} {t.unit || "mg"}</span>
                                    </div>
                                    {#if t.medicationType === "injectableEstradiol" && t.injectionSite}
                                        <div class="text-sm mt-1 text-gray-600 dark:text-gray-400">
                                            Site: {t.injectionSite}
                                        </div>
                                    {/if}
                                    {#if t.medicationType === "injectableEstradiol" && (t as any).vialId}
                                        {@const v = hrtData.data.vials.find(v => v.id === (t as any).vialId)}
                                        <div class="text-sm mt-1 text-gray-600 dark:text-gray-400">
                                            Vial:
                                            {#if v}
                                                {v.esterKind || '—'}
                                                {#if v.batchNumber} · {v.batchNumber}{/if}
                                                {#if v.source} · {v.source}{/if}
                                                {#if (t as any).subVialId}
                                                    {@const s = v.subVials.find(s => s.id === (t as any).subVialId)}
                                                    {#if s} — sub‑vial #{s.personalNumber}{/if}
                                                {/if}
                                            {:else}
                                                —
                                            {/if}
                                        </div>
                                    {/if}
                                    {#if t.medicationType === "injectableEstradiol" && ((t as any).syringeKind || (t as any).needleLength)}
                                        <div class="text-sm mt-1 text-gray-600 dark:text-gray-400">
                                            Syringe: {(t as any).syringeKind || '—'}{#if (t as any).needleLength} · Needle: {(t as any).needleLength}{/if}
                                        </div>
                                    {/if}
                                    {#if t.note}
                                        <div class="text-sm mt-1 text-gray-600 dark:text-gray-400">
                                            Note: {t.note}
                                        </div>
                                    {/if}
                                </div>
                                <button
                                    class="px-3 py-1 text-sm rounded bg-latte-rose-pine-foam text-white hover:bg-rose-pine-pine transition-colors"
                                    onclick={() => onEdit(t)}>Edit</button
                                >
                            </li>
                        {/each}
                    </ul>
                {/if}
            </div>
        </div>
    </div>
</div>
