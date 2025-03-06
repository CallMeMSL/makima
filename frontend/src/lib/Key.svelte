<script lang="ts">
    import {key} from "./store";
    import redirect from "./redirect";
</script>

<div class="text-center">
    <div class="relative">
        <input type="password" bind:value={$key}
               class="block w-full max-w-md min-w-2xs p-4 text-sm text-gray-200 border border-gray-500 rounded-lg bg-gray-800 dark:bg-neutral-900 dark:border-gray-700 dark:placeholder-gray-500 dark:text-white"
               placeholder="API Key..." required/>
        <button on:click={redirect}
                class="text-white absolute end-2.5 bottom-2.5 bg-red-800 hover:bg-red-900 focus:ring-2 focus:outline-none focus:ring-pink-950 font-medium rounded-lg text-sm px-4 py-2 dark:bg-pink-900 dark:hover:bg-pink-950 dark:focus:ring-red-500">
            Submit
        </button>
    </div>
</div>