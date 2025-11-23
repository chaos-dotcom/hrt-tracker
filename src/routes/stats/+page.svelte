<script lang="ts">
  export const ssr = false;

  import { hrtData } from '$lib/storage.svelte';

  const DAY_MS = 24 * 60 * 60 * 1000;

  // Records
  const estrogenRecords = $derived(
    (hrtData.data.dosageHistory ?? []).filter(
      (d) => d.medicationType === 'injectableEstradiol' || d.medicationType === 'oralEstradiol'
    )
  );
  const injectableRecords = $derived(
    (hrtData.data.dosageHistory ?? []).filter((d) => d.medicationType === 'injectableEstradiol')
  );

  // Totals (mg)
  const totalEstrogenMg = $derived(
    estrogenRecords.reduce((sum, d) => sum + (d.unit === 'mg' ? d.dose : 0), 0)
  );

  // Fallback concentration for records without a vial conc (mg/mL)
  let assumedConcMgPerMl = $state(40);

  // Sum volume using recorded vial concentrations when available
  const volumeAgg = $derived(() => {
    let sumMl = 0;
    let usedFallbackCount = 0;
    for (const d of injectableRecords) {
      if (d.unit !== 'mg') continue;
      const vial = (d as any).vialId
        ? hrtData.data.vials.find((v) => v.id === (d as any).vialId)
        : undefined;
      const conc = vial?.concentrationMgPerMl;
      const usedConc = typeof conc === 'number' && conc > 0
        ? conc
        : (assumedConcMgPerMl > 0 ? assumedConcMgPerMl : undefined);
      if (usedConc) {
        sumMl += d.dose / usedConc;
        if (!(typeof conc === 'number' && conc > 0)) usedFallbackCount++;
      }
    }
    return { sumMl, usedFallbackCount };
  });
  const totalInjectionMl = $derived(volumeAgg.sumMl);

  // Days since first dose (centralized helper)
  const totalDaysSinceStart = $derived(hrtData.getDaysSinceFirstDose());

  function fmt(n: number, decimals = 2): string {
    if (!isFinite(n)) return '—';
    const s = n.toFixed(decimals);
    return s.replace(/\.?0+$/, '');
  }
  function fmtIUFromMl(ml: number): string {
    if (!isFinite(ml)) return '—';
    return String(Math.round(ml * 100)); // 1 mL = 100 IU
  }
</script>

<div class="p-6 space-y-6 max-w-3xl mx-auto">
  <h1 class="text-2xl font-semibold">Stats</h1>

  <section class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow">
    <h2 class="text-lg font-medium mb-2">Total Estrogen Taken</h2>
    <div class="text-sm text-gray-700 dark:text-gray-300">
      <div class="mb-1">
        <strong>{fmt(totalEstrogenMg, 2)}</strong> mg (injectable + oral)
      </div>

      {#if injectableRecords.length > 0}
        <div class="mt-3">
          <label class="block text-sm font-medium mb-1">Assumed injectable concentration (for volume estimate)</label>
          <div class="flex items-center gap-2">
            <input
              type="number"
              min="0"
              step="0.1"
              class="border rounded px-2 py-1 w-28"
              bind:value={assumedConcMgPerMl}
            />
            <span>mg/mL</span>
          </div>
          <div class="mt-2">
            Estimated total injection volume:
            <strong>{isFinite(totalInjectionMl) ? fmt(totalInjectionMl, 3) : '—'}</strong> mL
            {#if isFinite(totalInjectionMl)}(<strong>{fmtIUFromMl(totalInjectionMl)}</strong> IU){/if}
            {#if volumeAgg.usedFallbackCount > 0}
              <div class="text-xs opacity-70 mt-1">
                Used fallback concentration for {volumeAgg.usedFallbackCount} dose(s) without recorded vial concentration.
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </section>

  <section class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow">
    <h2 class="text-lg font-medium mb-2">Days Since Starting</h2>
    {#if totalDaysSinceStart !== null}
      <div class="text-sm text-gray-700 dark:text-gray-300">
        <strong>{totalDaysSinceStart}</strong> days
      </div>
    {:else}
      <div class="text-sm text-gray-500 dark:text-gray-400 italic">
        No doses recorded yet.
      </div>
    {/if}
  </section>
</div>
