<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let addresses = $state<string[]>([]);

    let por = $state<string[]>([]);
    let currentAddress = $state<string | null>(null);
    let showModal = $state<boolean>(false);
    let personName = $state<string | null>(null);

    async function acceptClicked(address: string) {
        currentAddress = address;
        showModal = true;
        await loadAAA();
    }

    async function okClicked() {
        if (personName && currentAddress) {
            await invoke("add_friend", {
                name: personName,
                friendAddress: currentAddress,
            });
        }

        personName = null;
        showModal = false;
        currentAddress = null;
        await loadAAA();
    }

    async function cancelClicked() {
        personName = null;
        showModal = false;
        currentAddress = null;
        await loadAAA();
    }

    async function loadAAA() {
        addresses = await invoke("get_incoming_friend_requests");
        addresses = addresses.filter(
            (a) => !a.startsWith("0x00000000000000000"),
        );
        por = await invoke("get_pending_outgoing_requests");
    }

    onMount(async () => {
        await loadAAA();
    });
</script>

<div class="bg-gray-200 p-4 flex flex-col gap-4 select-none h-[50%]">
    <h2 class="text-2xl">Incoming Friend Requests</h2>

    <div class="flex flex-col gap-4 overflow-scroll hide-scrollbar">
        {#each addresses as ad}
            <div
                class="bg-white max-w-[70%] rounded-lg p-4 text-xl flex flex-col gap-4"
            >
                <p>{ad}</p>
                <div class="flex justify-between">
                    <button
                        class="bg-green-600 text-white text-sm p-1 px-4 rounded drop-shadow-md"
                        onclick={() => acceptClicked(ad)}>Accept</button
                    >
                    <button
                        class="bg-red-600 text-white text-sm p-1 px-4 rounded drop-shadow-md"
                        >Reject</button
                    >
                </div>
            </div>
        {/each}
    </div>
</div>
<div class="bg-gray-200 h-[50%] p-4 flex flex-col gap-4 select-none pb-8 pt-5">
    <h2 class="text-2xl">Pending Friend Requests</h2>

    <div class="flex flex-col gap-4 overflow-scroll hide-scrollbar">
        {#each por as p}
            <div
                class="bg-white max-w-[70%] rounded-lg p-4 text-xl flex justify-between gap-4 items-start"
            >
                <p>
                    <!-- <b class="pr-2 w-[30%] max-w-[30%] line-clamp-3 pb-2"
                        >Pending friend request</b
                    > -->{p}
                </p>
                <div>
                    <div
                        class="w-4 h-4 rounded-full bg-yellow-600 translate-y-1"
                    ></div>
                </div>
            </div>
        {/each}
    </div>
</div>

<div class="flex flex-col fixed top-20 right-20 justify-center items-center">
    <button
        class="text-3xl drop-shadow-lg shadow-black rounded p-2 px-4"
        onclick={() => goto("/dashboard")}
    >
        🏠
    </button>
    <span class="text-xl">Dashboard</span>
</div>
<!-- hidden -> flex -->
<div
    class="bg-black/50 {showModal
        ? 'flex'
        : 'hidden'} absolute inset-0 h-full w-full justify-center items-center"
>
    <div class="bg-white p-4 rounded w-[50%]">
        <form action="" class="flex flex-col gap-4">
            <label for="name" class="text-xl">Give this person a name</label>
            <input
                bind:value={personName}
                type="text"
                placeholder="Jon Doe"
                class="outline-none border p-2"
            />
            <div class="flex justify-between">
                <button
                    class="p-1 px-4 bg-green-600 text-white drop-shadow-md rounded"
                    onclick={okClicked}>OK</button
                >
                <button
                    class="p-1 px-4 bg-gray-600 text-white drop-shadow-md rounded"
                    onclick={cancelClicked}>Cancel</button
                >
            </div>
        </form>
    </div>
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
