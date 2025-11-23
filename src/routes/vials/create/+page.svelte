<script lang="ts">
  import { hrtData } from '$lib/storage.svelte';
  import { InjectableEstradiols } from '$lib/types';
  import { goto } from '$app/navigation';

  const esterOptions = Object.values(InjectableEstradiols);
  let esterKind: string = '';
  let customEster: string = '';
  let suspensionOil: string = '';
  let otherIngredients: string = '';
  let batchNumber: string = '';
  let firstSubNumber: string = '';

  function submit(e: Event) {
    e.preventDefault();
    const ester = esterKind === '__other__' ? (customEster || '').trim() : (esterKind || '').trim();
    const id = hrtData.createVial({
      esterKind: ester || undefined,
      suspensionOil: suspensionOil.trim() || undefined,
      otherIngredients: otherIngredients.trim() || undefined,
      batchNumber: batchNumber.trim() || undefined
    });
    if (firstSubNumber.trim()) {
      hrtData.addSubVial(id, firstSubNumber.trim());
    }
    goto('/vials');
  }
</script>

<div class="p-6 space-y-4">
  <h1 class="text-2xl font-semibold">Create Vial</h1>
  <form class="space-y-4" on:submit={submit}>
    <div>
      <label class="block text-sm font-medium mb-1">Ester kind</label>
      <select class="border rounded px-2 py-2 w-full" bind:value={esterKind}>
        <option value="">Select...</option>
        {#each esterOptions as opt}
          <option value={opt}>{opt}</option>
        {/each}
        <option value="__other__">Other...</option>
      </select>
      {#if esterKind === '__other__'}
        <input class="mt-2 border rounded px-2 py-2 w-full" placeholder="Custom ester" bind:value={customEster} />
      {/if}
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Suspension oil</label>
      <input class="border rounded px-2 py-2 w-full" placeholder="e.g., Castor oil, MCT" bind:value={suspensionOil} />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Other ingredients</label>
      <input class="border rounded px-2 py-2 w-full" placeholder="e.g., Benzyl benzoate, benzyl alcohol" bind:value={otherIngredients} />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">Batch number</label>
      <input class="border rounded px-2 py-2 w-full" placeholder="Batch/lot #" bind:value={batchNumber} />
    </div>
    <div>
      <label class="block text-sm font-medium mb-1">First sub‑vial/cartridge number (optional)</label>
      <input class="border rounded px-2 py-2 w-full" placeholder="e.g., 1" bind:value={firstSubNumber} />
    </div>
    <div class="pt-2">
      <button type="submit" class="bg-latte-rose-pine-foam text-white px-4 py-2 rounded">Create</button>
    </div>
  </form>
</div>
