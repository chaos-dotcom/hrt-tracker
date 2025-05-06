<script>
    // use context
    let storage = $state("");

    $effect(() => {
        storage = localStorage.getItem("storage") || "";
        if (storage === "") {
            localStorage.setItem("storage", storage);
        }
    });
</script>

<div class="p-10 flex flex-col space-y-2">
    <h1 class="text-4xl font-bold underline font-outfit">hrt-tracker</h1>
    <p>get started with tracking your doses and blood tests!</p>
    <div class="flex flex-col items-start">
        <a
            class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors"
            href="/backup">import data</a
        >
        <a
            class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors"
            href="/create/blood-test">create blood test</a
        >
        <a
            class="text-latte-rose-pine-iris dark:text-rose-pine-iris hover:text-rose-pine-love transition-colors"
            href="/create/dosage">set up dosage tracker</a
        >
    </div>
</div>
