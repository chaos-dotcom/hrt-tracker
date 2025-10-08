<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import {
        Antiandrogens,
        HormoneUnits,
        type BloodTest,
        type DosageHistoryEntry,
        type EstrogenType,
        InjectableEstradiols,
        OralEstradiols,
    } from "$lib/types";
    import * as Plot from "@observablehq/plot";
    import EditModal from "$lib/components/EditModal.svelte";

    let itemToEdit: BloodTest | DosageHistoryEntry | null = $state(null);

    function onEdit(item: BloodTest | DosageHistoryEntry) {
        itemToEdit = item;
    }

    function onCloseModal() {
        itemToEdit = null;
        renderChart();
    }

    // Chart related code
    let chartDiv: HTMLElement | undefined;
    let timeRangeInDays = $state(90); // Default to showing last 90 days
    let showBloodTests = $state(true); // Filter for blood test data
    let showMedications = $state(true); // Filter for medication data

    // Process data for charting
    function processDataForChart() {
        const now = Date.now();
        const startTime = now - timeRangeInDays * 24 * 60 * 60 * 1000;

        // Filter blood tests based on time range and filter setting
        const filteredBloodTests = showBloodTests
            ? hrtData.data.bloodTests
                  .filter((test) => test.date >= startTime)
                  .map((test) => ({
                      date: new Date(test.date),
                      type: "Blood Test",
                      estradiolLevel: test.estradiolLevel,
                      testLevel: test.testLevel,
                      estradiolUnit:
                          test.estradiolUnit || HormoneUnits.E2_pg_mL,
                      testUnit: test.testUnit || HormoneUnits.T_ng_dL,
                  }))
            : [];

        // Filter dosages based on time range and filter setting
        const filteredDosages = showMedications
            ? hrtData.data.dosageHistory
                  .filter((dose) => dose.date >= startTime)
                  .map((dose) => ({
                      date: new Date(dose.date),
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

        const now = Date.now();
        const { bloodTests, dosages, estradiolUnits, testUnits } =
            processDataForChart();

        // Only create chart if we have data
        if (bloodTests.length === 0 && dosages.length === 0) {
            chartDiv.textContent =
                "No data available for the selected time range";
            return;
        }

        // Prepare data for the main chart
        // Create dynamic y-axis label based on detected units
        const estradiolUnitsLabel = bloodTests.some(
            (d) => d.estradiolLevel !== undefined,
        )
            ? `Estradiol (${bloodTests.find((d) => d.estradiolLevel !== undefined)?.estradiolUnit || "pg/mL"})`
            : "";

        const testUnitsLabel = bloodTests.some((d) => d.testLevel !== undefined)
            ? `Testosterone (${bloodTests.find((d) => d.testLevel !== undefined)?.testUnit || "ng/dL"})`
            : "";

        const medicationUnitsLabel =
            dosages.length > 0 ? "Medication (mg)" : "";

        // Combine labels if multiple types are shown
        let yAxisLabel = [
            estradiolUnitsLabel,
            testUnitsLabel,
            medicationUnitsLabel,
        ]
            .filter((label) => label !== "")
            .join(", ");

        if (!yAxisLabel) {
            yAxisLabel = "Hormone Levels";
        }

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
            x: {
                label: "Date",
                type: "utc",
            },
            y: {
                label: yAxisLabel,
                grid: true,
            },
            color: {
                legend: true,
            },
            marks: [
                // Blood test estradiol levels as a line
                ...(showBloodTests &&
                bloodTests.some((d) => d.estradiolLevel !== undefined)
                    ? [
                          Plot.line(
                              bloodTests.filter(
                                  (d) => d.estradiolLevel !== undefined,
                              ),
                              {
                                  x: "date",
                                  y: "estradiolLevel",
                                  stroke: "steelblue",
                                  strokeWidth: 2,
                                  curve: "monotone-x",
                              },
                          ),
                          // Blood test estradiol points
                          Plot.dot(
                              bloodTests.filter(
                                  (d) => d.estradiolLevel !== undefined,
                              ),
                              {
                                  x: "date",
                                  y: "estradiolLevel",
                                  fill: "steelblue",
                                  r: 5,
                                  title: (d) =>
                                      `Estradiol: ${d.estradiolLevel} ${d.estradiolUnit || "pg/mL"} (${d.date.toLocaleDateString()})`,
                              },
                          ),
                      ]
                    : []),

                // Blood test testosterone levels as a line (if available)
                ...(showBloodTests &&
                bloodTests.some((d) => d.testLevel !== undefined)
                    ? [
                          Plot.line(
                              bloodTests.filter(
                                  (d) => d.testLevel !== undefined,
                              ),
                              {
                                  x: "date",
                                  y: "testLevel",
                                  stroke: "orangered",
                                  strokeWidth: 2,
                                  curve: "monotone-x",
                              },
                          ),
                          // Blood test testosterone points
                          Plot.dot(
                              bloodTests.filter(
                                  (d) => d.testLevel !== undefined,
                              ),
                              {
                                  x: "date",
                                  y: "testLevel",
                                  fill: "orangered",
                                  r: 5,
                                  title: (d) =>
                                      `Testosterone: ${d.testLevel} ${d.testUnit || "ng/dL"} (${d.date.toLocaleDateString()})`,
                              },
                          ),
                      ]
                    : []),

                // Injectable estradiol dosages as triangles
                ...(showMedications &&
                dosages.some((d) => d.type === "injectableEstradiol")
                    ? [
                          Plot.dot(
                              dosages.filter(
                                  (d) => d.type === "injectableEstradiol",
                              ),
                              {
                                  x: "date",
                                  y: (d) => Math.min(d.dose * 10, 200), // Scale for visibility but cap at 200
                                  fill: "green",
                                  symbol: "triangle",
                                  r: 8,
                                  title: (d) =>
                                      `Injection: ${d.name}, ${d.dose} ${d.unit || "mg"} (${d.date.toLocaleDateString()})`,
                              },
                          ),
                      ]
                    : []),

                // Oral estradiol dosages as squares
                ...(showMedications &&
                dosages.some((d) => d.type === "oralEstradiol")
                    ? [
                          Plot.dot(
                              dosages.filter((d) => d.type === "oralEstradiol"),
                              {
                                  x: "date",
                                  y: (d) => Math.min(d.dose * 10, 200),
                                  fill: "purple",
                                  symbol: "square",
                                  r: 7,
                                  title: (d) =>
                                      `Oral E: ${d.name}, ${d.dose} ${d.unit || "mg"} (${d.date.toLocaleDateString()})`,
                              },
                          ),
                      ]
                    : []),

                // Antiandrogen dosages as diamonds
                ...(showMedications &&
                dosages.some((d) => d.type === "antiandrogen")
                    ? [
                          Plot.dot(
                              dosages.filter((d) => d.type === "antiandrogen"),
                              {
                                  x: "date",
                                  y: (d) => Math.min(d.dose * 10, 200),
                                  fill: "darkorange",
                                  symbol: "diamond",
                                  r: 7,
                                  title: (d) =>
                                      `AA: ${d.name}, ${d.dose} ${d.unit || "mg"} (${d.date.toLocaleDateString()})`,
                              },
                          ),
                      ]
                    : []),

                // Add a legend for the data types
                Plot.rectY([0], {
                    x1: "min",
                    x2: "max",
                    y1: 0,
                    y2: 0,
                    stroke: "gray",
                    strokeWidth: 1,
                }),

                // Add a legend based on what's being shown
                Plot.text(
                    [
                        // Only show blood test legends if blood tests are displayed
                        ...(showBloodTests
                            ? [
                                  {
                                      x: new Date(
                                          now - 2 * 24 * 60 * 60 * 1000,
                                      ),
                                      y: 220,
                                      text: "● Estradiol (E2)",
                                      color: "steelblue",
                                  },
                                  {
                                      x: new Date(
                                          now - 2 * 24 * 60 * 60 * 1000,
                                      ),
                                      y: 205,
                                      text: "● Testosterone (T)",
                                      color: "orangered",
                                  },
                              ]
                            : []),
                        // Only show medication legends if medications are displayed
                        ...(showMedications
                            ? [
                                  {
                                      x: new Date(
                                          now - 2 * 24 * 60 * 60 * 1000,
                                      ),
                                      y: 190,
                                      text: "▲ Injection",
                                      color: "green",
                                  },
                                  {
                                      x: new Date(
                                          now - 2 * 24 * 60 * 60 * 1000,
                                      ),
                                      y: 175,
                                      text: "■ Oral E",
                                      color: "purple",
                                  },
                                  {
                                      x: new Date(
                                          now - 2 * 24 * 60 * 60 * 1000,
                                      ),
                                      y: 160,
                                      text: "◆ Anti-androgen",
                                      color: "darkorange",
                                  },
                              ]
                            : []),
                    ],
                    {
                        x: "x",
                        y: "y",
                        text: "text",
                        fill: (d) => d.color,
                        fontSize: 12,
                    },
                ),
            ],
        });

        chartDiv.append(chart);
    }

    $effect(() => {
        renderChart();
    });

    function updateTimeRange(days: number) {
        timeRangeInDays = days;
    }

    function toggleBloodTests() {
        showBloodTests = !showBloodTests;
    }

    function toggleMedications() {
        showMedications = !showMedications;
    }
</script>

{#if itemToEdit}
    <EditModal item={itemToEdit} close={onCloseModal} />
{/if}

<h1 class="text-3xl font-bold mb-2 px-4 pt-4">HRT Tracking Data</h1>
<p class="mb-4 px-4 text-sm opacity-75">
    This chart shows your hormone levels from blood tests along with your dosage
    history over time.
</p>
<div class="flex flex-col p-4 w-full max-w-[100vw]">
    <div class="mb-4 flex flex-wrap gap-2">
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

    <div class="mb-4 flex flex-wrap gap-3">
        <span class="self-center text-sm">Show:</span>
        <button
            class="px-3 py-1 text-sm transition-colors rounded hover:bg-latte-rose-pine-overlay dark:hover:bg-rose-pine-overlay hover:text-latte-rose-pine-text dark:hover:text-rose-pine-text"
            class:bg-latte-rose-pine-iris={showBloodTests}
            class:dark:bg-rose-pine-iris={showBloodTests}
            class:text-latte-rose-pine-base={showBloodTests}
            class:dark:text-rose-pine-base={showBloodTests}
            onclick={toggleBloodTests}
        >
            {showBloodTests ? "✓" : ""} Blood Test Levels
        </button>
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
            {#if showBloodTests && hrtData.data.bloodTests.length > 0}
                <p>* Hormone measurements shown in their recorded units.</p>
            {/if}
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
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
                                        {new Date(t.date).toLocaleDateString()}
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
                                <div>
                                    <div class="font-medium">
                                        {new Date(t.date).toLocaleDateString()}
                                    </div>
                                    <div class="text-sm flex gap-2">
                                        <span class="capitalize"
                                            >{t.medicationType ===
                                            "injectableEstradiol"
                                                ? "Injection"
                                                : t.medicationType ===
                                                    "oralEstradiol"
                                                  ? "Oral E"
                                                  : "AA"}</span
                                        >
                                        <span>{t.type}</span>
                                        <span>{t.dose} {t.unit || "mg"}</span>
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
    </div>
</div>
