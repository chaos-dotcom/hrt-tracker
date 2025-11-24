<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  // Defer Chart.js and plugins to the client to avoid SSR import issues
  let Chart: any = null;
  let zoomPlugin: any = null;
  // Adapter is loaded dynamically in onMount for side-effects

  let getPKFunctions: ((cf?: number) => any) | null = null;
  let pkReady = false;
  import { hrtData } from '$lib/storage.svelte';

  // PROPS
  // Expects an array of injection events.
  // Each event should have a timestamp (Unix ms), dose (in mg), and type.
  let { injections = [] } = $props<{ injections?: { timestamp: number; dose: number; type: string }[] }>();

  let derivedInjections = $state([] as { timestamp: number; dose: number; type: string }[]);
  $effect(() => {
    const hist = hrtData.data?.dosageHistory ?? [];
    derivedInjections = hist
      .filter((e: any) =>
        e &&
        e.medicationType === 'injectableEstradiol' &&
        typeof e.dose === 'number' &&
        typeof e.date === 'number' &&
        typeof e.type === 'string'
      )
      .map((e: any) => ({ timestamp: e.date, dose: e.dose, type: e.type }));
  });

  // DATA
  let chartData: any = { labels: [], datasets: [] };
  let options: any = {}; // Make options dynamic

  // Chart.js instance
  let chart: any = null;
  let canvasEl: HTMLCanvasElement | null = null;

  let viewMin: number | null = null;
  let viewMax: number | null = null;
  let detachDbl: (() => void) | null = null;

  const DAY_MS = 24 * 60 * 60 * 1000;
  const addDaysMs = (ms: number, days: number) => ms + days * DAY_MS;
  const subDaysMs = (ms: number, days: number) => ms - days * DAY_MS;

  function setLast30Days() {
    {
      const now = Date.now();
      viewMin = subDaysMs(now, 30);
      viewMax = addDaysMs(now, 2);
      chart?.update('none');
    }
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
  const estradiolModelMap: Record<string, string> = {
    'Estradiol Valerate': 'EV im',
    'Estradiol Enanthate': 'EEn im',
    'Estradiol Cypionate': 'EC im',
    'Estradiol Undecylate': 'EUn im',
    'Estradiol Benzoate': 'EB im'
    // Note: PolyestradiolPhosphate is not supported by the Estrannaise model file.
  };

  function toModelKey(name: string): string | undefined {
    const n = (name || '').toLowerCase();
    if (n.includes('valerate')) return 'EV im';
    if (n.includes('enanthate')) return 'EEn im';
    if (n.includes('cyp')) return 'EC im';
    if (n.includes('undec')) return 'EUn im';
    if (n.includes('benzo')) return 'EB im';
    return undefined;
  }

  function generateChartConfig() {
    if (!getPKFunctions) {
      // models not loaded yet; wait for pkReady
      return;
    }
    const pkFunctions = getPKFunctions(); // Using default conversion factor (outputs pg/mL)

    const src = injections?.length ? injections : derivedInjections;
    if (!src || src.length === 0) {
      chartData = { labels: [], datasets: [] };
      // Reset options when there's no data
      options = { plugins: { zoom: { pan: { enabled: false }, zoom: { wheel: { enabled: false } } } } };
      return;
    }

    const sortedInjections = [...src].sort((a, b) => a.timestamp - b.timestamp);
    const firstInjectionTime = sortedInjections[0].timestamp;
    const lastSimTime = addDaysMs(Date.now(), 14); // Simulate 14 days into the future
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

        const model = estradiolModelMap[String(injection.type)] ?? toModelKey(String(injection.type));
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

    {
      const now = Date.now();
      const defaultMin = subDaysMs(now, 30);
      const defaultMax = addDaysMs(now, 2);
      if (viewMin == null) viewMin = defaultMin;
      if (viewMax == null) viewMax = defaultMax;
    }

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
        decimation: {
          enabled: true,
          algorithm: 'lttb',
          samples: 1000
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
          pan: {
            enabled: true,
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

  onMount(async () => {
    const chartjs = await import('chart.js');
    const zoomMod = await import('chartjs-plugin-zoom');
    await import('chartjs-adapter-date-fns'); // side-effect registration for time scale

    // Load Estrannaise models on client only
    const mod = await import('@estrannaise/models.js');
    getPKFunctions = mod.PKFunctions;
    pkReady = true;

    Chart = chartjs.Chart;
    zoomPlugin = zoomMod.default;

    Chart.register(
      chartjs.Title,
      chartjs.Tooltip,
      chartjs.Legend,
      chartjs.LineElement,
      chartjs.CategoryScale,
      chartjs.LinearScale,
      chartjs.PointElement,
      chartjs.TimeScale,
      zoomPlugin,
      chartjs.Decimation
    );

    generateChartConfig();
    if (canvasEl) {
      chart = new Chart(canvasEl, {
        type: 'line',
        data: chartData,
        options
      });
    }
  });

  $: {
    if (canvasEl) {
      if (detachDbl) detachDbl();
      const handler = () => resetView();
      canvasEl.addEventListener('dblclick', handler);
      detachDbl = () => canvasEl.removeEventListener('dblclick', handler);
    }
  }
  onDestroy(() => {
    if (detachDbl) detachDbl();
    chart?.destroy();
  });

  // Regenerate chart config whenever injections change
  $: {
    injections; derivedInjections; viewMin; viewMax; pkReady;
    generateChartConfig();
    if (chart) {
      chart.options = options as any;
      chart.data.labels = chartData.labels as any;
      chart.data.datasets = chartData.datasets as any;
      chart.update('none');
    } else if (canvasEl && Chart && pkReady) {
      chart = new Chart(canvasEl, { type: 'line', data: chartData, options });
    }
  }
</script>

<div class="flex gap-2 items-center mb-2">
  <button type="button" onclick={resetView} class="px-2 py-1 border rounded">Reset view (dbl‑click chart)</button>
  <button type="button" onclick={setLast30Days} class="px-2 py-1 border rounded">Last 30 days</button>
  <button type="button" onclick={fitAll} class="px-2 py-1 border rounded">Fit all</button>
</div>

<div class="chart-container w-full" style="height: 400px; min-height: 400px; width: 100%; position: relative;">
  {#if pkReady && chartData.labels.length > 0}
    <canvas bind:this={canvasEl} style="width: 100%; height: 100%; display: block;"></canvas>
  {:else if !pkReady}
    <p>Loading simulation…</p>
  {:else}
    <p>No injection data to display.</p>
  {/if}
</div>
