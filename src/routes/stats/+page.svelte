<script lang="ts">
  import { hrtData } from '$lib/storage.svelte';
  import { ProgesteroneRoutes, SyringeKinds } from '$lib/types';

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

  const totalInjectableEstradiolMg = $derived(
    injectableRecords.reduce((sum, d: any) => sum + (d.unit === 'mg' ? d.dose : 0), 0)
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
    for (const d of injectableRecords as any[]) {
      if (d.unit !== 'mg') continue;
      const dose = Number(d.dose);
      if (!Number.isFinite(dose) || dose <= 0) continue;

      const vial = d.vialId ? hrtData.data.vials.find((v) => v.id === d.vialId) : undefined;
      const concVal = typeof vial?.concentrationMgPerMl === 'number'
        ? vial!.concentrationMgPerMl
        : Number(vial?.concentrationMgPerMl);
      if (!Number.isFinite(concVal) || concVal <= 0) continue;

      sumMl += dose / concVal;
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

  function deadspaceULFor(kind: string | undefined): number | null {
    if (!kind) return null;
    // Normalize to enum labels
    switch (kind) {
      case SyringeKinds.RegularSyringe: return 92;
      case SyringeKinds.LowWasteSyringe: return 59;
      case SyringeKinds.LowWasteNeedle: return 17;
      case SyringeKinds.InsulinSyringe: return 3;
      case SyringeKinds.InsulinPen: return 3;
      default: return null;
    }
  }

  const wastageAgg = $derived(() => {
    let totalMl = 0;
    let totalMg = 0;
    let skippedNoKind = 0;
    let skippedNoConcForMg = 0;
    let counted = 0;
    let deadForPctMl = 0;
    let drawnForPctMl = 0;

    for (const d of injectableRecords as any[]) {
      const dsUL = deadspaceULFor(d.syringeKind);
      if (dsUL === null) {
        skippedNoKind++;
        continue;
      }
      const dsMl = dsUL / 1000;
      totalMl += dsMl;
      counted++;

      const vial = d.vialId ? hrtData.data.vials.find((v) => v.id === d.vialId) : undefined;
      const conc = vial?.concentrationMgPerMl;
      if (typeof conc === 'number' && conc > 0) {
        totalMg += conc * dsMl;
        // For percent, need dose volume (mg -> mL) alongside deadspace
        if (d.unit === 'mg' && typeof d.dose === 'number' && d.dose > 0) {
          const doseMl = d.dose / conc;
          deadForPctMl += dsMl;
          drawnForPctMl += dsMl + doseMl;
        }
      } else {
        skippedNoConcForMg++;
      }
    }
    return { totalMl, totalMg, skippedNoKind, skippedNoConcForMg, counted, deadForPctMl, drawnForPctMl };
  });

  const wastagePct = $derived(
    wastageAgg.drawnForPctMl > 0 ? (100 * wastageAgg.deadForPctMl) / wastageAgg.drawnForPctMl : NaN
  );

  function parseNeedleLengthToMm(raw: string): number | null {
    // normalize, including unicode primes for inches and the small “㎜” symbol
    const s = String(raw || '')
      .trim()
      .toLowerCase()
      .replace(/[′’]/g, "'")
      .replace(/[″”]/g, '"')
      .replace(/\u339c/g, 'mm'); // ㎜ -> mm
    if (!s) return null;

    // 1) explicit mm (prefer this, e.g., "32g 4mm" -> 4)
    let m = s.match(/(\d+(?:\.\d+)?)\s*mm\b/);
    if (m) {
      const val = parseFloat(m[1]);
      return isFinite(val) && val > 0 ? val : null;
    }

    // 2) explicit cm
    m = s.match(/(\d+(?:\.\d+)?)\s*cm\b/);
    if (m) {
      const val = parseFloat(m[1]);
      return isFinite(val) && val > 0 ? val * 10 : null;
    }

    // 3) explicit inches (supports 1/2", 1 1/2", 0.5", 1 in, inches)
    const inchMatch = s.match(
      /([0-9]+(?:\.[0-9]+)?(?:\s+[0-9]+\/[0-9]+)?|[0-9]+\/[0-9]+)\s*(?:in|inch|inches|")\b/
    );
    if (inchMatch) {
      const token = inchMatch[1].trim();
      let val: number | null = null;
      if (token.includes('/')) {
        // mixed or simple fraction
        const parts = token.split(/\s+/);
        if (parts.length === 2 && parts[1].includes('/')) {
          const whole = parseFloat(parts[0]);
          const [num, den] = parts[1].split('/').map(Number);
          if (isFinite(whole) && isFinite(num) && isFinite(den) && den > 0) {
            val = whole + num / den;
          }
        } else if (parts.length === 1 && parts[0].includes('/')) {
          const [num, den] = parts[0].split('/').map(Number);
          if (isFinite(num) && isFinite(den) && den > 0) {
            val = num / den;
          }
        }
      } else {
        const n = parseFloat(token);
        if (isFinite(n) && n > 0) val = n;
      }
      return val && val > 0 ? val * 25.4 : null;
    }

    // 4) fallback: assume mm, use the last number in the string
    const nums = Array.from(s.matchAll(/\d+(?:\.\d+)?/g)).map((x) => parseFloat(x[0]));
    if (nums.length) {
      const val = nums[nums.length - 1];
      return isFinite(val) && val > 0 ? val : null;
    }

    return null;
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
        Injectable estradiol total: <strong>{fmt(totalInjectableEstradiolMg, 2)}</strong> mg
      </div>
      <div class="mb-1">
        Oral estradiol total: <strong>{fmt(totalOralEstradiolMg, 2)}</strong> mg
      </div>
      <div class="text-xs opacity-70 mb-1">
        Combined: <strong>{fmt(totalInjectableEstradiolMg + totalOralEstradiolMg, 2)}</strong> mg
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
      <div class="mt-2">
        Wastage from needle dead space:
        <strong>{isFinite(wastageAgg.totalMl) ? fmt(wastageAgg.totalMl, 3) : '—'}</strong> mL
        {#if isFinite(wastageAgg.totalMl)}(<strong>{fmtIUFromMl(wastageAgg.totalMl)}</strong> IU){/if}
        {#if wastageAgg.totalMg > 0}
          · ≈ <strong>{fmt(wastageAgg.totalMg, 2)}</strong> mg
        {/if}
        {#if isFinite(wastagePct)}
          · <strong>{fmt(wastagePct, 1)}</strong>% of drawn volume
        {/if}
      </div>
      {#if wastageAgg.skippedNoKind > 0 || wastageAgg.skippedNoConcForMg > 0}
        <div class="text-xs opacity-70 mt-1">
          {#if wastageAgg.skippedNoKind > 0}
            Skipped {wastageAgg.skippedNoKind} injection(s) without a syringe kind.
          {/if}
          {#if wastageAgg.skippedNoConcForMg > 0}
            {wastageAgg.skippedNoKind > 0 ? ' ' : ''}No mg estimate for {wastageAgg.skippedNoConcForMg} injection(s) lacking vial concentration.
          {/if}
        </div>
      {/if}
      {#if needleAgg.skipped > 0}
        <div class="text-xs opacity-70 mt-1">
          Skipped {needleAgg.skipped} injection(s) without a parsable needle length.
        </div>
      {/if}
    </div>
  </section>
</div>
