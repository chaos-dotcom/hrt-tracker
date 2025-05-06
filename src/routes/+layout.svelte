<script>
    import { loadHRTData } from "$lib/storage";
    loadHRTData();
    let { children } = $props();
    import "../app.css";
</script>

{@render children()}
