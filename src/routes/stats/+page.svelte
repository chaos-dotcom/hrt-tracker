<script lang="ts">
  export const ssr = false;

  import { hrtData } from '$lib/storage.svelte';
  import { ProgesteroneRoutes } from '$lib/types';

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
    estrogenRecords.reduce((sum, d: any) => {
      if (d.unit !== 'mg') return sum;
      if (d.medicationType === 'oralEstradiol') {
        const qty = Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1;
        return sum + d.dose * qty;
      }
      return sum + d.dose; // injectables
    }, 0)
  );

  // Sum volume only from doses with recorded vial concentration
  const totalInjectionMl = $derived(() => {
    let sumMl = 0;
    for (const d of injectableRecords) {
      if (d.unit !== 'mg') continue;
      const vial = (d as any).vialId
        ? hrtData.data.vials.find((v) => v.id === (d as any).vialId)
        : undefined;
      const conc = vial?.concentrationMgPerMl;
      if (typeof conc === 'number' && conc > 0) {
        sumMl += d.dose / conc;
      }
    }
    return sumMl;
  });

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

  function parseNeedleLengthToMm(raw: string): number | null {
    const s = String(raw || '').trim().toLowerCase();
    if (!s) return null;
    const m = s.match(/([0-9]+(?:\.[0-9]+)?)/);
    if (!m) return null;
    const val = parseFloat(m[1]);
    if (!isFinite(val) || val <= 0) return null;
    if (/\bcm\b|centimet(er|re)s?/.test(s)) return val * 10;
    if (/\bmm\b|millimet(er|re)s?/.test(s)) return val;
    if (/"/.test(s) || /\binches?\b/.test(s)) return val * 25.4;
    // No unit specified: assume mm
    return val;
  }

  const needleAgg = $derived(() => {
    let sumMm = 0;
    let skipped = 0;
    for (const d of injectableRecords) {
      const nl = (d as any).needleLength;
      if (!nl || String(nl).trim() === '') {
        skipped++;
        continue;
      }
      const mm = parseNeedleLengthToMm(String(nl));
      if (typeof mm === 'number' && isFinite(mm) && mm > 0) {
        sumMm += mm;
      } else {
        skipped++;
      }
    }
    return { sumMm, skipped };
  });
  const totalNeedleLengthMm = $derived(needleAgg.sumMm);

  // Pills: oral estradiol and progesterone (Boofed = "boofed")
  const oralEstradiolRecords = $derived(
    (hrtData.data.dosageHistory ?? []).filter((d) => d.medicationType === 'oralEstradiol')
  );
  const totalOralPillsCount = $derived(
    oralEstradiolRecords.reduce((sum, d: any) => sum + (Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1), 0)
  );
  const totalOralEstradiolMg = $derived(
    oralEstradiolRecords.reduce(
      (sum, d: any) =>
        sum + (d.unit === 'mg' ? d.dose * (Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1) : 0),
      0
    )
  );

  const progesteroneRecords = $derived(
    (hrtData.data.dosageHistory ?? []).filter((d) => d.medicationType === 'progesterone')
  );
  const boofedProgesteroneRecords = $derived(
    progesteroneRecords.filter((d: any) => d.route === ProgesteroneRoutes.Boofed)
  );
  const boofedProgesteroneCount = $derived(
    boofedProgesteroneRecords.reduce((sum, d: any) => sum + (Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1), 0)
  );
  const boofedProgesteroneMg = $derived(
    boofedProgesteroneRecords.reduce(
      (sum, d: any) =>
        sum + (d.unit === 'mg' ? d.dose * (Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1) : 0),
      0
    )
  );
  const totalProgesteroneMg = $derived(
    progesteroneRecords.reduce(
      (sum, d: any) =>
        sum + (d.unit === 'mg' ? d.dose * (Number(d.pillQuantity) > 0 ? Number(d.pillQuantity) : 1) : 0),
      0
    )
  );
  const totalPillsCount = $derived(totalOralPillsCount + boofedProgesteroneCount);
  const totalPillsMgCombined = $derived(totalOralEstradiolMg + boofedProgesteroneMg);
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
          <div>
            Total injection volume (from recorded vial concentrations):
            <strong>{isFinite(totalInjectionMl) ? fmt(totalInjectionMl, 3) : '—'}</strong> mL
            {#if isFinite(totalInjectionMl)}(<strong>{fmtIUFromMl(totalInjectionMl)}</strong> IU){/if}
          </div>
        </div>
      {/if}
    </div>
  </section>

  <section class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow">
    <h2 class="text-lg font-medium mb-2">Pills</h2>
    <div class="text-sm text-gray-700 dark:text-gray-300 space-y-1">
      <div>
        Estradiol pills taken:
        <strong>{totalOralPillsCount}</strong>
        {#if totalOralPillsCount > 0}
          (<strong>{fmt(totalOralEstradiolMg, 2)}</strong> mg total)
        {/if}
      </div>
      <div class="mt-1">
        Progesterone pills boofed:
        <strong>{boofedProgesteroneCount}</strong>
        {#if boofedProgesteroneCount > 0}
          (<strong>{fmt(boofedProgesteroneMg, 2)}</strong> mg total)
        {/if}
      </div>
      {#if totalPillsCount > 0}
        <div class="mt-1">
          All pills combined:
          <strong>{totalPillsCount}</strong> {totalPillsCount === 1 ? 'pill' : 'pills'}
          (<strong>{fmt(totalPillsMgCombined, 2)}</strong> mg total)
        </div>
      {/if}
      {#if progesteroneRecords.length > 0}
        <div class="text-xs opacity-70 mt-1">
          Total progesterone (all routes): <strong>{fmt(totalProgesteroneMg, 2)}</strong> mg
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

  <section class="border rounded-lg p-4 bg-white dark:bg-rose-pine-surface shadow">
    <h2 class="text-lg font-medium mb-2">Needle Usage</h2>
    <div class="text-sm text-gray-700 dark:text-gray-300">
      <div>
        Total combined needle length:
        <strong>{isFinite(totalNeedleLengthMm) ? fmt(totalNeedleLengthMm, 1) : '—'}</strong> mm
        {#if isFinite(totalNeedleLengthMm)}(<strong>{fmt(totalNeedleLengthMm / 25.4, 2)}</strong> in){/if}
      </div>
      {#if needleAgg.skipped > 0}
        <div class="text-xs opacity-70 mt-1">
          Skipped {needleAgg.skipped} injection(s) without a parsable needle length.
        </div>
      {/if}
    </div>
  </section>
</div>
