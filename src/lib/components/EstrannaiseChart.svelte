<script lang="ts">
  import { onMount } from 'svelte';
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

  import { PKFunctions } from '../../../../vendor/estrannaise/src/models.js';
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
    const step = 0.1; // Chart resolution in days

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

    options = {
      responsive: true,
      maintainAspectRatio: false,
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
          min: subDays(new Date(), 30).getTime(),
          max: addDays(new Date(), 2).getTime()
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
          pan: {
            enabled: true,
            mode: 'x' as const
          },
          zoom: {
            wheel: {
              enabled: true
            },
            pinch: {
              enabled: true
            },
            mode: 'x' as const
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

  // Regenerate chart config whenever injections change
  $: generateChartConfig();
</script>

<div class="chart-container" style="height: 400px; position: relative;">
  {#if chartData.labels.length > 0}
    <Line {chartData} {options} bind:chart />
  {:else}
    <p>No injection data to display.</p>
  {/if}
</div>
