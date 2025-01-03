<script lang="ts">
    import { type Message } from "./types";
    export let messages: Message[];
</script>

<div class="h-full flex flex-col-reverse gap-4 overflow-scroll select-none">
    {#each messages as m}
        {#if !m.theirs}
            <div class="flex">
                <div
                    class="w-4 h-4 bg-blue-200"
                    style="clip-path: polygon(100% 78%, 84% 42%, 69% 24%, 48% 13%, 20% 6%, 0 0, 100% 0);"
                ></div>
                <span
                    class="bg-blue-200 text-gray-800 max-w-[60%] p-2 rounded rounded-tl-none"
                    >{m.message}</span
                >
            </div>
        {:else}
            <div class="flex justify-end">
                <span
                    class="bg-gray-200 max-w-[60%] p-2 rounded rounded-tr-none"
                    >{m.message}</span
                >
                <div
                    class="w-4 h-4 bg-gray-200"
                    style="transform: rotateY(180deg); clip-path: polygon(100% 78%, 84% 42%, 69% 24%, 48% 13%, 20% 6%, 0 0, 100% 0);"
                ></div>
            </div>
        {/if}

        <div class="flex"></div>
    {/each}
</div>
