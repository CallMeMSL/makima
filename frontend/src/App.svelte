<script lang="ts">
    import Header from "./lib/Header.svelte";
    import Key from "./lib/Key.svelte";
    import redirect from "./lib/redirect";

    $: redirect();
</script>

<main>
    <Header></Header>
    <div class="w-full h-screen bg-gray-100 dark:bg-neutral-900 flex items-center justify-center">
        <Key></Key>
    </div>

</main>

