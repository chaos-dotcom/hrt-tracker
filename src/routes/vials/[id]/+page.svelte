<script lang="ts">
  import { hrtData } from '$lib/storage.svelte';
  import { InjectableEstradiols } from '$lib/types';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';

  const esterOptions = Object.values(InjectableEstradiols);

  const vialId = $derived($page.params.id);
  const vial = $derived(hrtData.data.vials.find(v => v.id === vialId));

  let esterKind: string = $state('');
  let customEster: string = $state('');
  let suspensionOil: string = $state('');
  let otherIngredients: string = $state('');
  let batchNumber: string = $state('');
  let source: string = $state('');
  let concentrationMgPerMl: number | '' = $state('');
  let createdDate: string = $state('');
  let useByDate: string = $state('');
  let isSpent: boolean = $state(false);
  let spentDate: string = $state('');

  $effect(() => {
    if (!vial) return;
    esterKind = typeof vial.esterKind === 'string' ? vial.esterKind : (vial.esterKind ?? '');
    suspensionOil = vial.suspensionOil ?? '';
    otherIngredients = vial.otherIngredients ?? '';
    batchNumber = vial.batchNumber ?? '';
    source = vial.source ?? '';
    concentrationMgPerMl = typeof vial.concentrationMgPerMl === 'number' ? vial.concentrationMgPerMl : '';
    createdDate = vial?.createdAt ? new Date(vial.createdAt).toISOString().slice(0,10) : '';
    useByDate = vial?.useBy ? new Date(vial.useBy).toISOString().slice(0,10) : '';
    isSpent = !!vial?.isSpent;
    spentDate = vial?.spentAt ? new Date(vial.spentAt).toISOString().slice(0,10) : '';
    customEster = '';
  });

  function save(e: Event) {
    e.preventDefault();
    if (!vial) return;
    const newEster = (esterKind === '__other__' ? customEster : esterKind).trim() || undefined;
    hrtData.updateVial(vialId, {
      esterKind: newEster,
      suspensionOil: suspensionOil.trim() || undefined,
      otherIngredients: otherIngredients.trim() || undefined,
      batchNumber: batchNumber.trim() || undefined,
      source: source.trim() || undefined,
      concentrationMgPerMl: Number.isFinite(+concentrationMgPerMl) && +concentrationMgPerMl > 0 ? +concentrationMgPerMl : undefined,
      createdAt: createdDate ? new Date(createdDate).getTime() : (vial.createdAt ?? Date.now()),
      useBy: useByDate ? new Date(useByDate).getTime() : undefined,
      isSpent,
      spentAt: isSpent ? (spentDate ? new Date(spentDate).getTime() : (vial.spentAt ?? Date.now())) : undefined
    });
    goto('/vials');
  }
</script>

<div class="p-6 space-y-4">
  {#if !vial}
    <p>Vial not found.</p>
  {:else}
    <h1 class="text-2xl font-semibold">Edit Vial</h1>
    <form class="space-y-4" on:submit={save}>
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
        <label class="block text-sm font-medium mb-1">Concentration (mg/mL)</label>
        <input class="border rounded px-2 py-2 w-full" type="number" step="any" min="0" bind:value={concentrationMgPerMl} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Suspension oil</label>
        <input class="border rounded px-2 py-2 w-full" bind:value={suspensionOil} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Other ingredients</label>
        <input class="border rounded px-2 py-2 w-full" bind:value={otherIngredients} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Batch number</label>
        <input class="border rounded px-2 py-2 w-full" bind:value={batchNumber} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Vial date</label>
        <input class="border rounded px-2 py-2 w-full" type="date" bind:value={createdDate} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Use by date</label>
        <input class="border rounded px-2 py-2 w-full" type="date" bind:value={useByDate} />
      </div>
      <div>
        <label class="block text-sm font-medium mb-1">Manufacturer / Source</label>
        <input class="border rounded px-2 py-2 w-full" bind:value={source} />
      </div>
      <div class="flex items-center gap-2">
        <label class="block text-sm font-medium">Spent</label>
        <input type="checkbox" bind:checked={isSpent} />
        {#if isSpent}
          <input class="border rounded px-2 py-2" type="date" bind:value={spentDate} />
        {/if}
      </div>
      <div class="pt-2">
        <button type="submit" class="bg-latte-rose-pine-foam text-white px-4 py-2 rounded">Save</button>
      </div>
    </form>
  {/if}
</div>
