<script lang="ts">
    import { hrtData } from "$lib/storage.svelte";
    import { HormoneUnits, type BloodTest } from "$lib/types";

    function recordDummy() {
        const test: BloodTest = {
            date: Date.now(),
            estradiolLevel: 150,
            testLevel: 12,
            estradiolUnit: HormoneUnits.E2_pg_mL,
            testUnit: HormoneUnits.T_ng_dL,
            notes: "hello",
        };
        hrtData.addBloodTest(test);
    }
</script>

<h2>Tests so far: {hrtData.data.bloodTests.length}</h2>
<button on:click={recordDummy}>Add Dummy Test</button>

<ul>
    <button on:click={recordDummy}>Add Dummy Test</button>

    {#each hrtData.data.bloodTests as t}
        <li>
            {new Date(t.date).toLocaleString()}: E2={t.estradiolLevel}{t.estradiolUnit}
        </li>
    {/each}
</ul>
