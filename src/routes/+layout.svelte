<script>
    import { browser } from "$app/environment";
    import { loadHRTData } from "$lib/storage";
    if (browser) {
        let result = loadHRTData();
    }
    let { children } = $props();
    import "../app.css";
</script>

{@render children()}
