<script>
    import { initHRTStorage } from "$lib/storage";
    initHRTStorage();
    let { children } = $props();
    import "../app.css";
</script>

{@render children()}
