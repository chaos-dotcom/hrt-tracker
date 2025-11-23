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
  const totalInjectionMg = $derived(
    injectableRecords.reduce((sum, d) => sum + (d.unit === 'mg' ? d.dose : 0), 0)
  );

  // User-assumed concentration for volume estimation (mg/mL)
  let assumedConcMgPerMl = $state(40);

  // Estimated volume (mL) for injections
  const totalInjectionMl = $derived(
    assumedConcMgPerMl > 0 ? totalInjectionMg / assumedConcMgPerMl : NaN
  );

  // Days since first recorded dose (any medication)
  const firstDoseDate = $derived(() => {
    const all = hrtData.data.dosageHistory ?? [];
    if (all.length === 0) return null as number | null;
    return Math.min(...all.map((d) => d.date));
  });
  const totalDaysSinceStart = $derived(
    firstDoseDate !== null ? Math.floor((Date.now() - (firstDoseDate as number)) / DAY_MS) : null
  );

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
