<script lang="ts">
  import { hrtData } from '$lib/storage.svelte';
  import { goto } from '$app/navigation';

  let newSubNumbers: Record<string, string> = $state({});

  function addSub(vialId: string) {
    const val = (newSubNumbers[vialId] || '').trim();
    if (!val) return;
    hrtData.addSubVial(vialId, val);
    newSubNumbers[vialId] = '';
  }
  function delVial(id: string) {
    if (confirm('Delete this vial?')) hrtData.deleteVial(id);
  }
  function delSub(vialId: string, subId: string) {
    hrtData.deleteSubVial(vialId, subId);
  }
</script>

<div class="p-6 space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold">Vials</h1>
    <a href="/vials/create" class="text-latte-rose-pine-iris hover:text-rose-pine-love transition-colors">Create New Vial</a>
  </div>

  {#if (hrtData.data.vials?.length ?? 0) === 0}
    <p>No vials yet.</p>
  {:else}
    <div class="grid gap-4">
      {#each hrtData.data.vials as v}
        <div class="border rounded-lg p-4">
          <div class="flex items-start justify-between">
            <div>
              <div class="font-medium">{v.esterKind || '—'}</div>
              <div class="text-sm opacity-80">Batch: {v.batchNumber || '—'}</div>
              <div class="text-sm opacity-80">Source: {v.source || '—'}</div>
              <div class="text-sm opacity-80">Concentration: {v.concentrationMgPerMl ?? '—'} mg/mL</div>
              <div class="text-sm opacity-80">Suspension oil: {v.suspensionOil || '—'}</div>
              <div class="text-sm opacity-80">Other ingredients: {v.otherIngredients || '—'}</div>
              <div class="text-xs opacity-60 mt-1">Created {new Date(v.createdAt).toLocaleString()}</div>
            </div>
            <div class="flex gap-3">
              <a class="text-blue-600 hover:underline" href={`/vials/${v.id}`}>Edit</a>
              <button class="text-red-600 hover:underline" on:click={() => delVial(v.id)}>Delete</button>
            </div>
          </div>

          <div class="mt-3">
            <div class="font-medium mb-1">Sub‑vials / Cartridges</div>
            {#if (v.subVials?.length ?? 0) === 0}
              <div class="text-sm opacity-70">None</div>
            {:else}
              <ul class="list-disc pl-5 space-y-1">
                {#each v.subVials as s}
                  <li class="flex items-center justify-between">
                    <span>#{s.personalNumber} <span class="opacity-60 text-xs">({new Date(s.createdAt).toLocaleDateString()})</span></span>
                    <button class="text-red-600 hover:underline text-sm" on:click={() => delSub(v.id, s.id)}>Delete</button>
                  </li>
                {/each}
              </ul>
            {/if}
            <div class="mt-2 flex items-center gap-2">
              <input class="border rounded px-2 py-1" placeholder="Add sub‑vial number" bind:value={newSubNumbers[v.id]} />
              <button class="bg-latte-rose-pine-foam text-white px-3 py-1 rounded" on:click={() => addSub(v.id)}>Add</button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
