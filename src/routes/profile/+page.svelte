<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let name = $state("");
    let account = $state("");
    let balance = $state("");

    interface Friend {
        name: string;
        public_address: string;
    }

    let friendsList = $state<Friend[]>([]);

    onMount(async () => {
        [account, name, balance] = await invoke("get_account_0x");

        friendsList = await invoke("select_friends");
    });
</script>

<div
    class="bg-gray-200 h-full absolute inset-0 flex flex-col gap-8 p-8 items-center select-none"
>
    <div class="flex flex-col gap-4 items-center">
        <span class="text-8xl"> 👤 </span>
        <span class="text-xl">{name} (me)</span>
        <span class="text-2xl cursor-not-allowed">{account} 🔒</span>
        <span class="text-2xl">{balance} wei</span>
    </div>

    <div class="flex flex-col gap-4">
        <h2 class="text-xl">Friends ({friendsList.length})</h2>
        <ul
            class="flex flex-col gap-4 max-h-[20rem] overflow-scroll hide-scrollbar"
        >
            {#each friendsList as f}
                <li class="bg-white p-2 rounded">
                    <b>{f.name}</b>
                    <p>{f.public_address}</p>
                </li>
            {/each}
        </ul>
    </div>

    <!-- <div class="flex flex-col">
        <button class="bg-red-600 text-white p-2 px-4 rounded">
            Reset Account
        </button>
    </div> -->
</div>

<div class="flex flex-col fixed top-10 right-[10%] justify-center items-center">
    <button
        class="text-3xl drop-shadow-lg shadow-black rounded p-2 px-4 translate-x-[50%]"
        onclick={() => goto("/dashboard")}
    >
        🏠
    </button>
</div>

<style lang="postcss">
    :global(html) {
        height: 100%;
    }
    :global(body) {
        position: fixed;
        overflow: hidden;
        width: 100%;
        height: 100%;
    }

    /* Hide scrollbar for Chrome, Safari and Opera */
    .hide-scrollbar::-webkit-scrollbar {
        display: none;
    }

    /* Hide scrollbar for IE, Edge and Firefox */
    .hide-scrollbar {
        -ms-overflow-style: none; /* IE and Edge */
        scrollbar-width: none; /* Firefox */
    }
</style>
