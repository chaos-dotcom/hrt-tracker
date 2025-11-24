<script lang="ts">
  import EstrannaiseChart from '$lib/components/EstrannaiseChart.svelte';
  import { hrtData } from '$lib/storage.svelte';
  import type { DosageHistoryEntry, InjectableEstradiols } from '$lib/types';

  let injections = $state([] as { timestamp: number; dose: number; type: string }[]);

  $effect(() => {
    const hist = hrtData.data?.dosageHistory ?? [];
    injections = hist
      .filter(
        (e): e is Extract<
          DosageHistoryEntry,
          { medicationType: 'injectableEstradiol' } & { dose: number; date: number; type: InjectableEstradiols }
        > =>
          e.medicationType === 'injectableEstradiol' &&
          typeof e.dose === 'number' &&
          typeof e.date === 'number' &&
          typeof e.type === 'string'
      )
      .map((e) => ({
        timestamp: e.date,
        dose: e.dose,
        type: e.type as any as InjectableEstradiols
      }));
  });
</script>

<svelte:head><title>Estradiol Simulation</title></svelte:head>

<div class="container p-4 mx-auto">
  <h1 class="text-2xl font-bold mb-3">Estradiol Simulation</h1>
  <p class="text-sm mb-4">Pan with mouse drag, zoom with wheel or pinch. Hold Shift and drag to box‑zoom. Double‑click to reset.</p>
  <EstrannaiseChart {injections} />
</div>
