<script lang="ts">
  // From Transfeminine Science: Dose/Conc -> Volume; Volume/Conc -> Dose
  let tfsDoseMg = $state(4);
  let tfsConcMgPerMl = $state(40);
  const tfsVolMl = $derived(tfsConcMgPerMl > 0 ? tfsDoseMg / tfsConcMgPerMl : NaN);

  let tfsVol2Ml = $state(0.1);
  let tfsConc2MgPerMl = $state(40);
  const tfsDose2Mg = $derived(tfsConc2MgPerMl > 0 ? tfsVol2Ml * tfsConc2MgPerMl : NaN);

  // From HRT Cafe: Vial Life & Dose
  let cafeDoseMg = $state(4);
  let cafeFreqDays = $state(7);
  let cafeVialMl = $state(10);
  let cafeConcMgMl = $state(40);

  type Gear = { name: string; dead_uL: number };
  const gears: Gear[] = [
    { name: 'Regular Syringe', dead_uL: 92 },
    { name: 'Low Waste Syringe', dead_uL: 59 },
    { name: 'Low Waste Needle', dead_uL: 17 },
    { name: 'Insulin Syringe (SubQ Only)', dead_uL: 3 }
  ];

  const injVolMl = $derived(cafeConcMgMl > 0 ? cafeDoseMg / cafeConcMgMl : NaN);

  function fmt(x: number, decimals = 3): string {
    if (!isFinite(x)) return '—';
    const s = x.toFixed(decimals);
    return s.replace(/\.?0+$/, '');
  }

  function fmtIUFromMl(ml: number): string {
    if (!isFinite(ml)) return '—';
    return String(Math.round(ml * 100)); // 1 mL = 100 IU
  }

  // Match HRT Cafe examples: show 1 decimal if <2%, else whole number
  function fmtPct(p: number): string {
    if (!isFinite(p)) return '—';
    if (p < 2) {
      const one = Math.round(p * 10) / 10;
      return String(one).replace(/\.0$/, '');
    }
    return String(Math.round(p));
  }

  function calcFor(dead_uL: number) {
    if (!(cafeVialMl > 0) || !(cafeConcMgMl > 0) || !(cafeDoseMg > 0) || !(cafeFreqDays > 0)) {
      return { doses: 0, days: 0, pctWaste: NaN, dead_uL };
    }
    const dead_mL = dead_uL / 1000;
    const dose_mL = cafeDoseMg / cafeConcMgMl;
    const drawn_mL = dose_mL + dead_mL;

    // Total draws possible from vial (estimated)
    const rawCount = cafeVialMl / drawn_mL;

    // HRT Cafe examples round to nearest for both doses and days
    const doses = Math.round(rawCount);
    const days = Math.round(rawCount * cafeFreqDays);

    const pctWaste = 100 * (dead_mL / drawn_mL);
    return { doses, days, pctWaste, dead_uL };
  }

  const gearResults = $derived(gears.map((g) => ({ g, r: calcFor(g.dead_uL) })));
</script>

<svelte:head>
  <title>Calculators</title>
  <meta name="description" content="Dose/Volume/Concentration converter and Vial Life & Dose calculator for injectables." />
</svelte:head>

<div style="max-width: 860px; margin: 0 auto; padding: 1rem;">
  <h1 style="margin: 0 0 1rem 0;">Calculators</h1>

  <!-- TFS: Dose/Volume/Concentration -->
  <section style="border: 1px solid var(--border, #ddd); border-radius: 8px; padding: 1rem; margin-bottom: 1rem;">
    <h2 style="margin-top: 0;">Dose, Volume, and Concentration Conversion (from - Transfeminine Science)</h2>

    <div style="display: grid; grid-template-columns: 1fr; gap: 1rem;">
      <div>
        <h3 style="margin: 0 0 0.5rem 0;">Dose and Concentration to Volume</h3>
        <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
          <label> Dose:
            <input type="number" min="0" step="0.01" bind:value={tfsDoseMg} style="width: 8rem;" /> mg
          </label>
          <label> Concentration:
            <input type="number" min="0" step="0.1" bind:value={tfsConcMgPerMl} style="width: 8rem;" /> mg/mL
          </label>
        </div>
        <p style="margin: 0.5rem 0 0 0;">
          Volume = Dose ÷ Concentration = <strong>{isFinite(tfsVolMl) ? fmt(tfsVolMl, 3) : '—'}</strong> mL {#if isFinite(tfsVolMl)}(<strong>{fmtIUFromMl(tfsVolMl)}</strong> IU){/if}
        </p>
      </div>

      <div>
        <h3 style="margin: 0 0 0.5rem 0;">Volume and Concentration to Dose</h3>
        <div style="display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap;">
          <label> Volume:
            <input type="number" min="0" step="0.01" bind:value={tfsVol2Ml} style="width: 8rem;" /> mL
            <span style="opacity:0.7; font-size: 0.9em;">(≈ {isFinite(tfsVol2Ml) ? fmtIUFromMl(tfsVol2Ml) : '—'} IU)</span>
          </label>
          <label> Concentration:
            <input type="number" min="0" step="0.1" bind:value={tfsConc2MgPerMl} style="width: 8rem;" /> mg/mL
          </label>
        </div>
        <p style="margin: 0.5rem 0 0 0;">
          Dose = Volume × Concentration = <strong>{isFinite(tfsDose2Mg) ? fmt(tfsDose2Mg, 3) : '—'}</strong> mg
        </p>
      </div>
    </div>

    <details style="margin-top: 0.75rem;">
      <summary>Notes</summary>
      <ol style="margin: 0.5rem 0 0 1rem;">
        <li>Volume is meaningless without concentration (for understanding dose).</li>
        <li>State what you use in terms of dose; it’s more interpretable.</li>
      </ol>
    </details>
  </section>

  <!-- HRT Cafe: Vial Life & Dose Calculator -->
  <section style="border: 1px solid var(--border, #ddd); border-radius: 8px; padding: 1rem;">
    <h2 style="margin-top: 0;">Vial Life & Dose Calculator (from HRT Cafe)</h2>

    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 0.75rem;">
      <label>I am injecting
        <input type="number" min="0" step="0.1" bind:value={cafeDoseMg} style="width: 100%;" /> mg
      </label>
      <label>every
        <input type="number" min="0" step="1" bind:value={cafeFreqDays} style="width: 100%;" /> days
      </label>
      <label>My vial is
        <input type="number" min="0" step="0.1" bind:value={cafeVialMl} style="width: 100%;" /> mL
      </label>
      <label>at
        <input type="number" min="0" step="0.1" bind:value={cafeConcMgMl} style="width: 100%;" /> mg/mL
      </label>
    </div>

    <p style="margin-top: 0.75rem;">
      Inject a volume of <strong>{isFinite(injVolMl) ? fmt(injVolMl, 3) : '—'}</strong> mL {#if isFinite(injVolMl)}(<strong>{fmtIUFromMl(injVolMl)}</strong> IU){/if}
    </p>

    <h3 style="margin: 1rem 0 0.5rem 0;">Estimated Vial Lifetime</h3>
    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 0.75rem;">
      {#each gearResults as { g, r } (g.name)}
        <div style="border: 1px solid var(--border, #ddd); border-radius: 8px; padding: 0.75rem;">
          <h4 style="margin: 0 0 0.5rem 0;">{g.name}</h4>
          <div><strong>{r.doses}</strong> doses</div>
          <div><strong>{r.days}</strong> days</div>
          <div><strong>{fmtPct(r.pctWaste)}</strong> pct waste</div>
          <div>{r.dead_uL} uL dead space</div>
        </div>
      {/each}
    </div>

    <details style="margin-top: 0.75rem;">
      <summary>Notes</summary>
      <ul style="margin: 0.5rem 0 0 1rem;">
        <li>Waste% = dead space / (dead space + drawn dose) per injection.</li>
        <li>Doses ≈ round(vial mL / (dose mL + dead space mL)).</li>
        <li>Days ≈ round((vial mL / (dose mL + dead space mL)) × frequency days).</li>
        <li>Estimates only; keep a spare vial and use sterile technique.</li>
      </ul>
    </details>
  </section>
</div>

<style>
  /* Make label text + input + unit nicely spaced */
  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  /* Input appearance */
  input[type="number"] {
    padding: 0.45rem 0.6rem;
    border: 1px solid var(--border, #d1d5db);
    border-radius: 0.5rem;
    background: #ffffff;
    color: inherit;
    outline: none;
    transition: border-color 120ms ease, box-shadow 120ms ease, background-color 120ms ease;
    box-shadow: 0 1px 2px rgba(0,0,0,0.04);
  }
  input[type="number"]:focus {
    border-color: #7c4dff; /* accent outline */
    box-shadow: 0 0 0 3px rgba(124,77,255,0.25);
    background: #fff;
  }

  /* Remove number input spinners */
  input[type="number"]::-webkit-outer-spin-button,
  input[type="number"]::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  input[type="number"] {
    -moz-appearance: textfield;
  }

  /* Dark mode-friendly defaults */
  @media (prefers-color-scheme: dark) {
    input[type="number"] {
      background: rgba(255,255,255,0.06);
      border-color: rgba(148,163,184,0.35);
      box-shadow: 0 1px 1px rgba(0,0,0,0.2) inset;
      color: inherit;
    }
    input[type="number"]:focus {
      border-color: #c4a7e7;
      box-shadow: 0 0 0 3px rgba(196,167,231,0.35);
      background: rgba(255,255,255,0.1);
    }
  }
</style>
