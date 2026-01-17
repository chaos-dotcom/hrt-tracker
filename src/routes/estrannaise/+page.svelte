<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import { HormoneUnits, type BloodTest, type DosageHistoryEntry, InjectableEstradiols } from "$lib/types";
    import * as Plot from "@observablehq/plot";
    import { e2multidose3C, type EstrannaiseModel } from "$lib/estrannaise-model";

    function convertEstradiolToDisplayUnit(value: number, unit: HormoneUnits, displayUnit: HormoneUnits) {
        if (displayUnit === HormoneUnits.E2_pmol_L) {
            return unit === HormoneUnits.E2_pmol_L ? value : value * 3.6713;
        }
        return unit === HormoneUnits.E2_pmol_L ? value / 3.6713 : value;
    }

    function estradiolConversionFactor(displayUnit: HormoneUnits) {
        return displayUnit === HormoneUnits.E2_pmol_L ? 3.6713 : 1;
    }

    function getFudgeFactorSeries() {
        const tests = hrtData.data.bloodTests as BloodTest[] | undefined;
        const withFudge = [...(tests ?? [])]
            .filter((t) => typeof (t as any).fudgeFactor === "number" && isFinite((t as any).fudgeFactor as number))
            .sort((a, b) => a.date - b.date)
            .map((t) => ({ date: t.date, value: Number(((t as any).fudgeFactor as number).toFixed(3)) }));
        if (withFudge.length > 0) return withFudge;
        console.warn("[Estrannaise] No fudge factor data found, using fallback 1.0");
        return [{ date: Date.now(), value: 1.0 }];
    }

    function blendFudgeFactor(series: { date: number; value: number }[], targetDate: number) {
        if (series.length === 0) return 1;
        if (targetDate <= series[0].date) return series[0].value;
        const last = series[series.length - 1];
        if (targetDate >= last.date) return last.value;

        for (let i = 1; i < series.length; i++) {
            const prev = series[i - 1];
            const next = series[i];
            if (targetDate <= next.date) {
                const span = next.date - prev.date;
                if (span <= 0) return prev.value;
                const ratio = (targetDate - prev.date) / span;
                return prev.value + (next.value - prev.value) * ratio;
            }
        }

        return last.value;
    }

    function stepFudgeFactor(series: { date: number; value: number }[], targetDate: number) {
        if (series.length === 0) return 1;
        if (targetDate <= series[0].date) return series[0].value;

        for (let i = 1; i < series.length; i++) {
            const next = series[i];
            if (targetDate < next.date) {
                return series[i - 1].value;
            }
        }

        return series[series.length - 1].value;
    }

    function mapEstradiolModel(type: InjectableEstradiols | string): EstrannaiseModel | null {
        const normalized = String(type || "").trim().toLowerCase();
        if (normalized.includes("benzoate") || normalized === InjectableEstradiols.Benzoate.toLowerCase()) {
            return "EB im";
        }
        if (normalized.includes("valerate") || normalized === InjectableEstradiols.Valerate.toLowerCase()) {
            return "EV im";
        }
        if (normalized.includes("enanthate") || normalized.includes("een") || normalized === InjectableEstradiols.Enanthate.toLowerCase()) {
            return "EEn im";
        }
        if (normalized.includes("cypionate") || normalized.includes("ec") || normalized === InjectableEstradiols.Cypionate.toLowerCase()) {
            return "EC im";
        }
        if (normalized.includes("undecylate") || normalized.includes("eun") || normalized === InjectableEstradiols.Undecylate.toLowerCase()) {
            return "EUn im";
        }
        return null;
    }

    function buildEstrannaiseSeries(
        firstDoseDate: number | null,
        extraDoses: Extract<DosageHistoryEntry, { medicationType: "injectableEstradiol" }>[] = [],
        forecastEndDate?: number,
    ) {
        const history = hrtData.data.dosageHistory
            .filter(
                (d): d is Extract<DosageHistoryEntry, { medicationType: "injectableEstradiol" }> =>
                    d.medicationType === "injectableEstradiol",
            )
            .sort((a, b) => a.date - b.date);

        if (history.length === 0 && extraDoses.length === 0) {
            return {
                blended: [] as { date: Date; xDays?: number; value: number }[],
                step: [] as { date: Date; xDays?: number; value: number }[],
            };
        }

        const combinedHistory = [...history, ...extraDoses].sort((a, b) => a.date - b.date);
        const startDate = combinedHistory[0].date;
        const lastDoseDate = combinedHistory[combinedHistory.length - 1].date;
        const baseEndDate = Math.max(Date.now(), lastDoseDate + 30 * 24 * 60 * 60 * 1000);
        const endDate = forecastEndDate ? Math.max(baseEndDate, forecastEndDate) : baseEndDate;
        const series = getFudgeFactorSeries();
        const blendedPoints: { date: Date; xDays?: number; value: number }[] = [];
        const stepPoints: { date: Date; xDays?: number; value: number }[] = [];
        const fallbackStepFudge = series.length ? series[series.length - 1] : { date: startDate, value: 1 };
        const baseStepFudgeEntry = series.length > 1 ? series[series.length - 2] : fallbackStepFudge;
        const displayUnit =
            (hrtData.data.settings as any)?.displayEstradiolUnit || HormoneUnits.E2_pmol_L;
        const conversionFactor = estradiolConversionFactor(displayUnit);

        const stepMs = 6 * 60 * 60 * 1000; // 6h resolution
        for (let t = startDate; t <= endDate; t += stepMs) {
            const doses = combinedHistory.filter((d) => d.date <= t);
            if (doses.length === 0) continue;

            const doseAmounts = doses.map((d) => d.dose);
            const times = doses.map((d) => (d.date - startDate) / (1000 * 60 * 60 * 24));
            const models = doses
                .map((d) => mapEstradiolModel(d.type))
                .filter((m): m is EstrannaiseModel => Boolean(m));
            if (models.length === 0) continue;

            const blendedFudge = blendFudgeFactor(series, t);
            const baseStepFudge = t >= baseStepFudgeEntry.date
                ? baseStepFudgeEntry.value
                : stepFudgeFactor(series, t);
            const stepFudge = baseStepFudge;
            const dayValue = (t - startDate) / (1000 * 60 * 60 * 24);
            const blendedValue = e2multidose3C(
                dayValue,
                doseAmounts,
                times,
                models,
                blendedFudge * conversionFactor,
                false,
            );
            const stepValue = e2multidose3C(
                dayValue,
                doseAmounts,
                times,
                models,
                stepFudge * conversionFactor,
                false,
            );

            blendedPoints.push({
                date: new Date(t),
                xDays: firstDoseDate !== null ? (t - firstDoseDate) / (1000 * 60 * 60 * 24) : undefined,
                value: blendedValue,
            });
            stepPoints.push({
                date: new Date(t),
                xDays: firstDoseDate !== null ? (t - firstDoseDate) / (1000 * 60 * 60 * 24) : undefined,
                value: stepValue,
            });
        }

        return { blended: blendedPoints, step: stepPoints };
    }

    let estrannaiseChartDiv: HTMLElement | undefined;
    let simulatorDiv: HTMLElement | undefined;

    let firstDoseDate: number | null = $state(null);
    let xAxisMode = $state<"date" | "days">("date");

    let forecastEnabled = $state(true);
    let forecastWeeks = $state(8);
    let forecastDoseOverride = $state("");
    let forecastFrequencyOverride = $state("");
    let forecastStepFudgeOverride = $state("1");

    const DAY_MS = 24 * 60 * 60 * 1000;
    $effect(() => {
        const dosageHistory = hrtData.data.dosageHistory;
        if (!dosageHistory || dosageHistory.length === 0) {
            firstDoseDate = null;
            return;
        }
        firstDoseDate = Math.min(...dosageHistory.map((d) => d.date));
    });

    function redraw() {
        if (estrannaiseChartDiv) {
            renderEstrannaiseChart(estrannaiseChartDiv);
        }
    }

    $effect(() => {
        if (!estrannaiseChartDiv) return;
        window.addEventListener("resize", redraw);
        return () => window.removeEventListener("resize", redraw);
    });

    $effect(() => {
        if (!estrannaiseChartDiv) return;
        xAxisMode; firstDoseDate; forecastEnabled; forecastWeeks; forecastDoseOverride; forecastFrequencyOverride; forecastStepFudgeOverride;
        hrtData.data.bloodTests;
        hrtData.data.dosageHistory;
        hrtData.data.injectableEstradiol;
        hrtData.data.settings;
        renderEstrannaiseChart(estrannaiseChartDiv);
    });

    function renderEstrannaiseChart(container: HTMLElement) {
        const schedule = hrtData.data.injectableEstradiol;
        const forecastWeeksNum = Math.max(4, Math.min(8, Number(forecastWeeks) || 8));
        const forecastHorizonMs = forecastWeeksNum * 7 * DAY_MS;
        const forecastStart = Date.now();
        const forecastEnd = forecastStart + forecastHorizonMs;
        const overrideDose = Number(forecastDoseOverride);
        const overrideFrequency = Number(forecastFrequencyOverride);
        const overrideStepFudge = Number(forecastStepFudgeOverride);
        const forecastDose = Number.isFinite(overrideDose) && overrideDose > 0 ? overrideDose : schedule?.dose;
        const forecastFrequency = Number.isFinite(overrideFrequency) && overrideFrequency > 0 ? overrideFrequency : schedule?.frequency;
        const forecastStepFudge = Number.isFinite(overrideStepFudge) && overrideStepFudge > 0 ? overrideStepFudge : 1;
        const lastInjectable = hrtData.data.dosageHistory
            .filter((d): d is Extract<DosageHistoryEntry, { medicationType: "injectableEstradiol" }> =>
                d.medicationType === "injectableEstradiol",
            )
            .at(-1);
        const forecastType = schedule?.type || lastInjectable?.type;
        const forecastUnit = schedule?.unit || HormoneUnits.mg;
        const forecastStartDate = Math.max(forecastStart, schedule?.nextDoseDate ?? forecastStart);

        const forecastDoses: Extract<DosageHistoryEntry, { medicationType: "injectableEstradiol" }>[] = [];
        if (
            forecastEnabled &&
            forecastDose &&
            forecastFrequency &&
            forecastType &&
            Number.isFinite(forecastStartDate)
        ) {
            for (let t = forecastStartDate; t <= forecastEnd; t += forecastFrequency * DAY_MS) {
                forecastDoses.push({
                    date: t,
                    dose: forecastDose,
                    unit: forecastUnit,
                    type: forecastType,
                    medicationType: "injectableEstradiol",
                });
            }
        }

        const series = buildEstrannaiseSeries(
            firstDoseDate,
            forecastDoses,
            forecastEnabled ? forecastEnd : undefined,
        );        const useDaysAxis = xAxisMode === "days" && firstDoseDate !== null;
        const displayUnit =
            (hrtData.data.settings as any)?.displayEstradiolUnit || HormoneUnits.E2_pmol_L;
        const bloodTests = hrtData.data.bloodTests
            .filter((t) => t.estradiolLevel !== undefined)
            .map((t) => {
                const rawUnit = t.estradiolUnit || HormoneUnits.E2_pg_mL;
                const value = convertEstradiolToDisplayUnit(
                    t.estradiolLevel as number,
                    rawUnit,
                    displayUnit,
                );
                const date = new Date(t.date);
                return {
                    date,
                    x: useDaysAxis && firstDoseDate !== null
                        ? (t.date - firstDoseDate) / (1000 * 60 * 60 * 24)
                        : date,
                    value,
                };
            });

        if (!series.blended.length && !series.step.length && !bloodTests.length) {
            container.innerHTML = `
                <div class="p-4 text-sm">
                    <p class="mb-2 font-medium">No Estrannaise data available</p>
                    <p class="text-gray-600 dark:text-gray-400 mb-2">To see model lines:</p>
                    <ul class="list-disc list-inside text-gray-600 dark:text-gray-400 space-y-1">
                        <li>Add dose history (injectable estradiol)</li>
                        <li>Add blood tests with Estrannaise predicted E2 values</li>
                        <li>Fudge factors will be computed automatically</li>
                    </ul>
                </div>
            `;
            return;
        }

        const containerWidth = container.clientWidth || window.innerWidth - 50;
        const isValidPoint = (p: { value: number; x?: number | Date }) => {
            if (typeof p.value !== "number" || !isFinite(p.value)) return false;
            if (useDaysAxis) {
                return typeof p.x === "number" && isFinite(p.x);
            }
            return p.x instanceof Date && !isNaN(p.x.getTime());
        };
        const validBlended = series.blended
            .map((p) => ({
                ...p,
                x: useDaysAxis && firstDoseDate !== null
                    ? (p.date.getTime() - firstDoseDate) / (1000 * 60 * 60 * 24)
                    : p.date,
            }))
            .filter(isValidPoint);
        const validStep = series.step
            .map((p) => ({
                ...p,
                x: useDaysAxis && firstDoseDate !== null
                    ? (p.date.getTime() - firstDoseDate) / (1000 * 60 * 60 * 24)
                    : p.date,
            }))
            .filter(isValidPoint);
        const validBloodTests = bloodTests
            .map((b) => ({
                ...b,
                x: useDaysAxis && firstDoseDate !== null
                    ? (b.date.getTime() - firstDoseDate) / (1000 * 60 * 60 * 24)
                    : b.date,
            }))
            .filter(isValidPoint);

        const values = [
            ...validBlended.map((p) => p.value),
            ...validStep.map((p) => p.value),
            ...validBloodTests.map((b) => b.value),
        ];
        let yMin = values.length ? Math.min(...values) : 0;
        let yMax = values.length ? Math.max(...values) : 1;
        if (yMin === yMax) {
            yMin = yMin - 1;
            yMax += 1;
        } else {
            const pad = 0.08 * (yMax - yMin);
            yMin = yMin - pad;
            yMax += pad;
        }

        const forecastBand = forecastEnabled && (!useDaysAxis || firstDoseDate !== null)
            ? Plot.rect([
                  {
                      x1: useDaysAxis ? (forecastStart - firstDoseDate!) / DAY_MS : new Date(forecastStart),
                      x2: useDaysAxis ? (forecastEnd - firstDoseDate!) / DAY_MS : new Date(forecastEnd),
                      y1: yMin,
                      y2: yMax,
                  },
              ], {
                  x1: "x1",
                  x2: "x2",
                  y1: "y1",
                  y2: "y2",
                  fill: "#f6c177",
                  fillOpacity: 0.12,
                  stroke: "none",
              })
            : null;

        const splitPoint = forecastEnabled ? forecastStart : Number.POSITIVE_INFINITY;
        const blendedHistorical = validBlended.filter((p) => p.date.getTime() < splitPoint);
        const blendedForecast = forecastEnabled
            ? validBlended.filter((p) => p.date.getTime() >= splitPoint)
            : [];
        const fudgeSeries = getFudgeFactorSeries();
        const stepBaselineDate = fudgeSeries.length > 1
            ? fudgeSeries[fudgeSeries.length - 2]?.date
            : fudgeSeries.at(-1)?.date ?? forecastStart;
        const stepSplitPoint = forecastEnabled ? stepBaselineDate : Number.POSITIVE_INFINITY;
        const stepHistorical = validStep.filter((p) => p.date.getTime() < stepSplitPoint);
        const stepForecast = forecastEnabled
            ? validStep
                  .filter((p) => p.date.getTime() >= stepSplitPoint)
                  .map((p) => ({
                      ...p,
                      value: p.value * forecastStepFudge,
                  }))
            : [];


        const toSegments = (points: Array<{ x: number | Date; value: number }>) =>
            points.slice(1).map((point, index) => ({
                x1: points[index].x,
                y1: points[index].value,
                x2: point.x,
                y2: point.value,
            }));

        const blendedHistoricalSegments = toSegments(blendedHistorical);
        const blendedForecastSegments = toSegments(blendedForecast);
        const stepHistoricalSegments = toSegments(stepHistorical);
        const stepForecastSegments = toSegments(stepForecast);

        const chart = Plot.plot({
            title: "Estrannaise Model (blended vs. step)",
            width: Math.max(300, containerWidth - 20),
            height: Math.min(420, Math.max(280, containerWidth * 0.45)),
            marginLeft: 60,
            marginRight: 40,
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
                label: `E2 (${displayUnit})`,
                grid: true,
                domain: [yMin, yMax],
            },
            marks: [
                ...(forecastBand ? [forecastBand] : []),
                ...(blendedHistoricalSegments.length
                    ? [
                          Plot.link(blendedHistoricalSegments, {
                              x1: "x1",
                              y1: "y1",
                              x2: "x2",
                              y2: "y2",
                              stroke: "#2E86AB",
                              strokeWidth: 2,
                          }),
                      ]
                    : []),
                ...(blendedForecastSegments.length
                    ? [
                          Plot.link(blendedForecastSegments, {
                              x1: "x1",
                              y1: "y1",
                              x2: "x2",
                              y2: "y2",
                              stroke: "#2E86AB",
                              strokeWidth: 2,
                          }),
                      ]
                    : []),
                ...(stepHistoricalSegments.length
                    ? [
                          Plot.link(stepHistoricalSegments, {
                              x1: "x1",
                              y1: "y1",
                              x2: "x2",
                              y2: "y2",
                              stroke: "#A23B72",
                              strokeWidth: 2,
                              strokeDasharray: "6,4",
                          }),
                      ]
                    : []),
                ...(stepForecastSegments.length
                    ? [
                          Plot.link(stepForecastSegments, {
                              x1: "x1",
                              y1: "y1",
                              x2: "x2",
                              y2: "y2",
                              stroke: "#A23B72",
                              strokeWidth: 2,
                              strokeDasharray: "2,6",
                          }),
                      ]
                    : []),
                ...(validBloodTests.length
                    ? [
                          Plot.ruleX(validBloodTests, {
                              x: "x",
                              stroke: "orange",
                              strokeDasharray: "4,4",
                              strokeWidth: 1.5,
                              opacity: 0.8,
                          }),
                          Plot.dot(validBloodTests, {
                              x: "x",
                              y: "value",
                              fill: "orange",
                              r: 4,
                              title: (d: { x: number | Date; value: number }) => {
                                  const dayPrefix =
                                      useDaysAxis && typeof d.x === "number"
                                          ? `Day ${d.x.toFixed(1)} – `
                                          : "";
                                  const dateLabel =
                                      !useDaysAxis && d.x instanceof Date
                                          ? d.x.toLocaleDateString()
                                          : "";
                                  return `Blood test: ${Number(d.value).toFixed(1)} ${displayUnit} (${dayPrefix}${dateLabel})`;
                              },
                          }),
                      ]
                    : []),
            ],
        });

        container.firstChild?.remove();
        container.append(chart);
    }

    $effect(() => {
        if (!simulatorDiv) return;
        simulatorDiv.innerHTML = "";
        const iframe = document.createElement("iframe");
        iframe.src = "https://estrannai.se/#";
        iframe.title = "Estrannaise repeated doses simulator";
        iframe.className = "w-full min-h-[700px] border rounded";
        iframe.loading = "lazy";
        simulatorDiv.appendChild(iframe);
    });
</script>

<div class="flex justify-between items-center px-4 pt-4">
    <h1 class="text-3xl font-bold mb-2">Estrannaise</h1>
</div>
<p class="mb-4 px-4 text-sm opacity-75">
    Estrannaise-style modeling with blended vs. step fudge factors, plus the original
    repeated-dose simulator.
</p>
<div class="flex flex-col p-4 w-full max-w-[100vw] gap-4">
    <div class="mb-2 flex flex-wrap gap-2 items-center">
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
            class="px-3 py-1 text-sm transition-colors bg-latte-rose-pine-surface dark:bg-rose-pine-surface text-latte-rose-pine-text dark:text-rose-pine-text rounded dark:hover:bg-latte-rose-pine-overlay hover:bg-latte-rose-pine-overlay disabled:opacity-50 disabled:cursor-not-allowed"
            class:bg-latte-rose-pine-iris={xAxisMode === "days"}
            class:dark:bg-rose-pine-iris={xAxisMode === "days"}
            class:text-latte-rose-pine-base={xAxisMode === "days"}
            class:dark:text-rose-pine-base={xAxisMode === "days"}
            onclick={() => (xAxisMode = "days")}
            disabled={firstDoseDate === null}
        >
            Days since first dose
        </button>
        <div class="flex items-center gap-2 ml-2">
            <label class="text-sm">Forecast</label>
            <input type="checkbox" bind:checked={forecastEnabled} />
        </div>
        <div class="flex items-center gap-2">
            <label class="text-sm">Weeks</label>
            <select bind:value={forecastWeeks} class="px-2 py-1 text-sm rounded bg-latte-rose-pine-surface dark:bg-rose-pine-surface">
                <option value="4">4</option>
                <option value="6">6</option>
                <option value="8">8</option>
            </select>
        </div>
        <div class="flex items-center gap-2">
            <label class="text-sm">Dose</label>
            <input
                type="number"
                min="0"
                step="0.1"
                class="w-20 px-2 py-1 text-sm rounded bg-latte-rose-pine-surface dark:bg-rose-pine-surface"
                placeholder="auto"
                bind:value={forecastDoseOverride}
            />
        </div>
        <div class="flex items-center gap-2">
            <label class="text-sm">Every (days)</label>
            <input
                type="number"
                min="1"
                step="1"
                class="w-20 px-2 py-1 text-sm rounded bg-latte-rose-pine-surface dark:bg-rose-pine-surface"
                placeholder="auto"
                bind:value={forecastFrequencyOverride}
            />
        </div>
        <div class="flex items-center gap-2">
            <label class="text-sm">Step fudge</label>
            <input
                type="number"
                min="0.1"
                step="0.1"
                class="w-20 px-2 py-1 text-sm rounded bg-latte-rose-pine-surface dark:bg-rose-pine-surface"
                placeholder="1.0"
                bind:value={forecastStepFudgeOverride}
            />
        </div>
    </div>

    <div class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md w-full">
        <div bind:this={estrannaiseChartDiv} class="w-full min-w-0 overflow-x-auto" role="img"></div>
        <div class="mt-4 text-sm text-gray-500 dark:text-gray-400 italic">
            <p>* Blue line blends fudge factor between blood tests.</p>
            <p>* Pink dashed line steps to each test's fudge factor.</p>
            <p>* Orange points show measured E2 in display units.</p>
            <p>* Shaded region is forecasted schedule window.</p>
            <p>* Forecast step fudge scales pink future line.</p>
        </div>
    </div>

    <div class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow-md w-full">
        <h2 class="text-xl font-medium mb-2">Repeated Doses Simulator</h2>
        <div bind:this={simulatorDiv}></div>
        <div class="mt-4 text-sm text-gray-500 dark:text-gray-400 italic">
            <p>* Embedded from estrannai.se for interactive simulations.</p>
        </div>
    </div>
</div>
