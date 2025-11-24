<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Line } from 'svelte-chartjs';
  import {
    Chart as ChartJS,
    Title,
    Tooltip,
    Legend,
    LineElement,
    CategoryScale,
    LinearScale,
    PointElement,
    TimeScale
  } from 'chart.js';
  import zoomPlugin from 'chartjs-plugin-zoom';
  import 'chartjs-adapter-date-fns';
  import { subDays, addDays } from 'date-fns';

  import { PKFunctions } from '../../../vendor/estrannaise/src/models.js';
  import { InjectableEstradiols } from '$lib/types';

  // PROPS
  // Expects an array of injection events.
  // Each event should have a timestamp (Unix ms), dose (in mg), and type.
  export let injections: {
    timestamp: number;
    dose: number;
    type: InjectableEstradiols;
  }[] = [];

  // DATA
  let chartData: any = { labels: [], datasets: [] };
  let options: any = {}; // Make options dynamic

  // Chart.js instance
  let chart: ChartJS;

  let viewMin: number | null = null;
  let viewMax: number | null = null;
  let detachDbl: (() => void) | null = null;

  function setLast30Days() {
    viewMin = subDays(new Date(), 30).getTime();
    viewMax = addDays(new Date(), 2).getTime();
    chart?.update('none');
  }
  function fitAll() {
    const labels = chartData?.labels as Date[] | undefined;
    if (labels && labels.length > 1) {
      viewMin = labels[0].getTime();
      viewMax = labels[labels.length - 1].getTime();
      chart?.resetZoom();
      chart?.update('none');
    }
  }
  function resetView() {
    setLast30Days();
    chart?.resetZoom();
  }

  // This maps your application's estradiol types to the model names used by Estrannaise.
  const estradiolModelMap: Partial<Record<InjectableEstradiols, string>> = {
    [InjectableEstradiols.Valerate]: 'EV im',
    [InjectableEstradiols.Enanthate]: 'EEn im',
    [InjectableEstradiols.Cypionate]: 'EC im',
    [InjectableEstradiols.Undecylate]: 'EUn im',
    [InjectableEstradiols.Benzoate]: 'EB im'
    // Note: PolyestradiolPhosphate is not supported by the Estrannaise model file.
  };

  function generateChartConfig() {
    const pkFunctions = PKFunctions(); // Using default conversion factor (outputs pg/mL)

    if (!injections || injections.length === 0) {
      chartData = { labels: [], datasets: [] };
      // Reset options when there's no data
      options = { plugins: { zoom: { pan: { enabled: false }, zoom: { wheel: { enabled: false } } } } };
      return;
    }

    const sortedInjections = [...injections].sort((a, b) => a.timestamp - b.timestamp);
    const firstInjectionTime = sortedInjections[0].timestamp;
    const lastSimTime = addDays(new Date(), 14).getTime(); // Simulate 14 days into the future
    const totalDays = (lastSimTime - firstInjectionTime) / (1000 * 3600 * 24);

    const labels: Date[] = [];
    const dataPoints: number[] = [];
    const MAX_POINTS = 2000;
    const step = Math.max(totalDays / MAX_POINTS, 0.05);

    for (let day = 0; day <= totalDays; day += step) {
      const currentTime = firstInjectionTime + day * 1000 * 3600 * 24;
      let totalE2 = 0;

      // Sum the contribution of each past injection at the current time point
      for (const injection of sortedInjections) {
        if (injection.timestamp > currentTime) continue;

        const model = estradiolModelMap[injection.type];
        if (!model || !pkFunctions[model]) continue;

        const timeSinceInjectionDays = (currentTime - injection.timestamp) / (1000 * 3600 * 24);
        const pkFunction = pkFunctions[model];
        totalE2 += pkFunction(timeSinceInjectionDays, injection.dose);
      }

      labels.push(new Date(currentTime));
      dataPoints.push(totalE2);
    }

    chartData = {
      labels,
      datasets: [
        {
          label: 'Simulated Estradiol (pg/mL)',
          data: dataPoints,
          borderColor: '#ef4444',
          backgroundColor: '#ef4444',
          pointRadius: 0,
          borderWidth: 2,
          tension: 0.1
        }
      ]
    };

    const defaultMin = subDays(new Date(), 30).getTime();
    const defaultMax = addDays(new Date(), 2).getTime();
    if (viewMin == null) viewMin = defaultMin;
    if (viewMax == null) viewMax = defaultMax;

    options = {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index' as const, intersect: false },
      animation: { duration: 0 },
      scales: {
        x: {
          type: 'time' as const,
          time: {
            unit: 'day' as const
          },
          title: {
            display: true,
            text: 'Date'
          },
          // Default view: last 30 days to 2 days in the future
          min: viewMin,
          max: viewMax
        },
        y: {
          title: {
            display: true,
            text: 'Estradiol (pg/mL)'
          },
          beginAtZero: true
        }
      },
      plugins: {
        legend: {
          display: true
        },
        tooltip: {
          mode: 'index' as const,
          intersect: false,
          callbacks: {
            title: function (tooltipItems: any[]) {
              if (!tooltipItems.length) return '';
              const date = new Date(tooltipItems[0].parsed.x);
              const daysSinceStart = ((date.getTime() - firstInjectionTime) / (1000 * 3600 * 24)).toFixed(
                1
              );
              return `${date.toLocaleString()} (Day ${daysSinceStart})`;
            }
          }
        },
        zoom: {
          zoom: {
            wheel: { enabled: true, speed: 0.1 },
            pinch: { enabled: true },
            drag: {
              enabled: true,
              modifierKey: 'shift',
              borderColor: '#ef4444',
              borderWidth: 1,
              backgroundColor: 'rgba(239,68,68,0.15)'
            },
            mode: 'x' as const
          },
          limits: {
            x: {
              min: labels[0].getTime(),
              max: labels[labels.length - 1].getTime(),
              minRange: 6 * 3600 * 1000 // 6 hours
            }
          },
          onZoomComplete({ chart: c }) {
            const xs = c.scales.x;
            viewMin = xs.min;
            viewMax = xs.max;
          },
          onPanComplete({ chart: c }) {
            const xs = c.scales.x;
            viewMin = xs.min;
            viewMax = xs.max;
          }
        }
      }
    };
  }

  onMount(() => {
    ChartJS.register(
      Title,
      Tooltip,
      Legend,
      LineElement,
      CategoryScale,
      LinearScale,
      PointElement,
      TimeScale,
      zoomPlugin
    );
  });

  $: {
    if (chart && chart.canvas) {
      if (detachDbl) detachDbl();
      const handler = () => resetView();
      chart.canvas.addEventListener('dblclick', handler);
      detachDbl = () => chart.canvas.removeEventListener('dblclick', handler);
    }
  }
  onDestroy(() => { if (detachDbl) detachDbl(); });

  // Regenerate chart config whenever injections change
  $: { injections; viewMin; viewMax; generateChartConfig(); }
</script>

<div class="flex gap-2 items-center mb-2">
  <button type="button" on:click={resetView} class="px-2 py-1 border rounded">Reset view (dbl‑click chart)</button>
  <button type="button" on:click={setLast30Days} class="px-2 py-1 border rounded">Last 30 days</button>
  <button type="button" on:click={fitAll} class="px-2 py-1 border rounded">Fit all</button>
</div>

<div class="chart-container" style="height: 400px; position: relative;">
  {#if chartData.labels.length > 0}
    <Line {chartData} {options} bind:chart />
  {:else}
    <p>No injection data to display.</p>
  {/if}
</div>
