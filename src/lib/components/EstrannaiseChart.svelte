<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Plotly from 'svelte-plotly.js';
  import PlotlyLib from 'plotly.js-dist-min';

  let getPKFunctions: ((cf?: number) => any) | null = null;
  let pkReady = $state(false);
  import { hrtData } from '$lib/storage.svelte';

  // PROPS
  // Expects an array of injection events.
  // Each event should have a timestamp (Unix ms), dose (in mg), and type.
  let { injections = [] } = $props<{ injections?: { timestamp: number; dose: number; type: string }[] }>();

  let derivedInjections = $state([] as { timestamp: number; dose: number; type: string }[]);
  

  // Plotly state
  let dataTraces = $state([] as any[]);
  let layout = $state({} as any);
  const config = {
    responsive: true,
    displayModeBar: true,
    displaylogo: false,
    scrollZoom: true,
    doubleClick: 'reset',
    modeBarButtonsToRemove: ['toImage']
  };

  let viewMin: number | null = null;
  let viewMax: number | null = null;
  let lastDataMin = $state<number | null>(null);
  let lastDataMax = $state<number | null>(null);

  let pollId: any = null;

  let readyHandler: ((e?: any) => void) | null = null;

  function recomputeDerived() {
    const hist = hrtData.data?.dosageHistory ?? [];
    const next = hist
      .filter((e: any) =>
        e &&
        e.medicationType === 'injectableEstradiol' &&
        typeof e.dose === 'number' &&
        typeof e.date === 'number' &&
        typeof e.type === 'string'
      )
      .map((e: any) => ({ timestamp: e.date, dose: e.dose, type: e.type }));
    derivedInjections = next;
  }

  const DAY_MS = 24 * 60 * 60 * 1000;
  const addDaysMs = (ms: number, days: number) => ms + days * DAY_MS;
  const subDaysMs = (ms: number, days: number) => ms - days * DAY_MS;

  function setLast30Days() {
    const now = Date.now();
    viewMin = subDaysMs(now, 30);
    viewMax = addDaysMs(now, 2);
    updateXRangeFromView();
  }
  function fitAll() {
    if (lastDataMin != null && lastDataMax != null) {
      viewMin = lastDataMin;
      viewMax = lastDataMax;
      updateXRangeFromView();
    }
  }
  function resetView() {
    setLast30Days();
  }
  function updateXRangeFromView() {
    if (viewMin != null && viewMax != null) {
      layout = {
        ...layout,
        xaxis: { ...(layout.xaxis || {}), type: 'date', range: [new Date(viewMin), new Date(viewMax)] }
      };
    }
  }
  function handleRelayout(e: CustomEvent<any>) {
    const d: any = e.detail || {};
    const r0 = d['xaxis.range[0]'] ?? d?.xaxis?.range?.[0];
    const r1 = d['xaxis.range[1]'] ?? d?.xaxis?.range?.[1];
    if (r0 && r1) {
      const min = new Date(r0).getTime();
      const max = new Date(r1).getTime();
      if (Number.isFinite(min) && Number.isFinite(max)) {
        viewMin = min;
        viewMax = max;
      }
    }
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
      dataTraces = [];
      layout = layout || {};
      return;
    }

    const sortedInjections = [...src].sort((a, b) => a.timestamp - b.timestamp);
    const firstInjectionTime = sortedInjections[0].timestamp;
    let lastSimTime = addDaysMs(Date.now(), 14); // Simulate 14 days into the future
    let totalDays = (lastSimTime - firstInjectionTime) / (1000 * 3600 * 24);
    if (totalDays <= 0) {
      // if first injection is in the future beyond now+14d, simulate 14d from the first injection
      lastSimTime = addDaysMs(firstInjectionTime, 14);
      totalDays = (lastSimTime - firstInjectionTime) / (1000 * 3600 * 24);
    }

    const points: { x: number; y: number }[] = [];
    const MAX_POINTS = 2000;
    const spanDays = Math.max(totalDays, 0.1);
    const step = Math.max(spanDays / MAX_POINTS, 0.05);
    let maxY = 0;

    for (let day = 0; day <= spanDays; day += step) {
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

      points.push({ x: currentTime, y: totalE2 });
      if (totalE2 > maxY) maxY = totalE2;
    }

    lastDataMin = points.length ? points[0].x : null;
    lastDataMax = points.length ? points[points.length - 1].x : null;

    const x = points.map(p => new Date(p.x));
    const y = points.map(p => p.y);

    lastDataMin = points.length ? points[0].x : null;
    lastDataMax = points.length ? points[points.length - 1].x : null;

    {
      const now = Date.now();
      const defaultMin = subDaysMs(now, 30);
      const defaultMax = addDaysMs(now, 2);
      if (viewMin == null) viewMin = defaultMin;
      if (viewMax == null) viewMax = defaultMax;
      const dataMin = points[0]?.x ?? now;
      const dataMax = points[points.length - 1]?.x ?? addDaysMs(now, 1);
      if (viewMax < dataMin || viewMin > dataMax) {
        viewMin = dataMin;
        viewMax = dataMax;
      }
    }

    dataTraces = [
      {
        type: points.length > 5000 ? 'scattergl' : 'scatter',
        mode: 'lines',
        name: 'Simulated Estradiol (pg/mL)',
        x,
        y,
        line: { color: '#ef4444', width: 2 },
        hovertemplate: '%{x|%b %d, %Y %H:%M} (Day %{customdata:.1f})<br>E2: %{y:.0f} pg/mL<extra></extra>',
        customdata: x.map((d: Date) => (d.getTime() - firstInjectionTime) / DAY_MS)
      }
    ];

    layout = {
      autosize: true,
      margin: { t: 20, r: 10, b: 40, l: 60 },
      paper_bgcolor: 'transparent',
      plot_bgcolor: 'transparent',
      xaxis: {
        type: 'date',
        title: 'Date',
        range: [new Date(viewMin!), new Date(viewMax!)]
      },
      yaxis: {
        title: 'Estradiol (pg/mL)',
        rangemode: 'tozero',
        autorange: maxY === 0 ? true : undefined
      },
      showlegend: false
    };
  }

  onMount(async () => {
    try {
      const mod = await import('@estrannaise/models.js');
      getPKFunctions = mod.PKFunctions;
    } finally {
      pkReady = true;
    }

    // kick initial compute and build plot; also listen for hrt-data-ready and short poll
    recomputeDerived();
    const onReady = () => {
      recomputeDerived();
      generateChartConfig();
    };
    readyHandler = onReady;
    window.addEventListener('hrt-data-ready', onReady);
    onReady();

    pollId = setInterval(() => {
      const before = derivedInjections.length;
      recomputeDerived();
      if (derivedInjections.length !== before) {
        generateChartConfig();
        clearInterval(pollId);
        pollId = null;
      }
    }, 400);
  });

  onDestroy(() => {
    if (pollId) clearInterval(pollId);
    if (readyHandler) window.removeEventListener('hrt-data-ready', readyHandler);
  });

  // Regenerate chart config whenever injections change
  $effect(() => {
    // dependencies
    void injections; void derivedInjections; void viewMin; void viewMax; void pkReady;

    generateChartConfig();
  });
</script>

<div class="flex gap-2 items-center mb-2">
  <button type="button" onclick={resetView} class="px-2 py-1 border rounded">Reset view (dbl‑click chart)</button>
  <button type="button" onclick={setLast30Days} class="px-2 py-1 border rounded">Last 30 days</button>
  <button type="button" onclick={fitAll} class="px-2 py-1 border rounded">Fit all</button>
</div>

<div class="chart-container w-full relative" style="height: 400px; min-height: 400px; width: 100%;">
  <Plotly
    plotly={PlotlyLib}
    data={dataTraces}
    layout={layout}
    config={config}
    useResizeHandler
    on:plotly_relayout={handleRelayout}
    style="width: 100%; height: 100%;"
  />
</div>
