<script lang="ts">
    import { goto } from "$app/navigation";
    import ChatMessages from "$lib/components/chat-messages.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { type Message } from "$lib/components/types";

    let showFriendAdder = $state(false);
    let friendAdderName = $state("");
    let friendAdderAddress = $state("");

    let activeFriendName = $state<string | null>(null);
    let activeFriendAddress = $state<string | null>(null);

    let messages = $state<Message[]>([]);

    interface Friend {
        name: string;
        public_address: string;
    }

    let friendsList = $state<Friend[]>([]);

    let textMessage = $state<string>("");

    let files = $state<FileList>();
    let fileText = $state("");
    let fileIsChosen = $state(false);
    let fileValue = $state(null);

    let reader = new FileReader();
    reader.onload = function (event: ProgressEvent<FileReader>) {
        if (!event.target) {
            return;
        }
        let arrayBuffer = event.target.result as ArrayBuffer;
        let fileBytes = new Uint8Array(arrayBuffer);
        fileText = new TextDecoder().decode(fileBytes);
    };

    async function sendSubmitEvent(e: SubmitEvent) {
        if (!activeFriendName || !activeFriendAddress) {
            return;
        }

        e.preventDefault();
        if (files && fileIsChosen) {
            // Send `fileText` under the name of `files[0].name`
            console.log(fileText);
            console.log(files[0].name);
        } else {
            // Send `textMessage`
            await invoke("send_message", {
                friendAddress: activeFriendAddress,
                friendName: activeFriendName,
                message: textMessage,
            });
            console.log(textMessage);
            textMessage = "";
            await loadMessages(activeFriendAddress, activeFriendName);
        }
        fileIsChosen = false;
    }

    async function addFriend() {
        console.log(friendAdderAddress);
        console.log(friendAdderName);
        console.log("submitting");

        await invoke("add_friend", {
            name: friendAdderName,
            friendAddress: friendAdderAddress,
        });

        friendAdderName = "";
        friendAdderAddress = "";

        await loadFriends();

        showFriendAdder = false;
    }

    async function loadMessages(friendAddress: string, friendName: string) {
        let dmessages = (await invoke("read_messages", {
            friendAddress,
            friendName,
        })) as Message[];
        messages = dmessages;
    }

    async function loadFriends() {
        friendsList = await invoke("select_friends");
    }

    function first3last3(publicAddress: string) {
        let res = "0x";
        res += publicAddress[0];
        res += "...";
        res += publicAddress[3];
        res += publicAddress[4];
        res += publicAddress[5];
        return res;
    }

    async function clickedOnFriend(name: string, friendAddress: string) {
        // check ssk
        await invoke("check_and_update_ssk", {
            name,
            friendAddress,
        });
        activeFriendName = name;
        activeFriendAddress = friendAddress;
        await loadMessages(friendAddress, name);
    }

    $effect(() => {
        if (files) {
            reader.readAsArrayBuffer(files[0]);
            textMessage = "";
            fileIsChosen = true;
        }
    });

    onMount(async () => {
        let stateIsFilled = await invoke("state_is_filled");
        if (!stateIsFilled) {
            goto("/login");
        }
        await loadFriends();
    });
</script>

<div class="p-2 flex gap-4 h-screen w-full">
    <div
        class="w-[25%] h-full bg-gray-200 rounded p-4 flex flex-col justify-between select-none"
    >
        <div>
            <h2 class="text-2xl mb-4 relative">
                Friends
                <span class="absolute top-0 right-0 bottom-0">
                    <button
                        class="bg-white rounded px-2 drop-shadow-lg"
                        onclick={() => (showFriendAdder = true)}>+</button
                    >
                </span>
            </h2>
            <ul class="flex flex-col gap-4">
                {#each friendsList as f}
                    <li
                        class="bg-white rounded cursor-pointer px-4 py-1 text-gray-800 drop-shadow"
                        onclick={() =>
                            clickedOnFriend(f.name, f.public_address)}
                    >
                        {f.name} - {first3last3(f.public_address)}
                    </li>
                {/each}
            </ul>
        </div>
        <div class="flex gap-4">
            <button
                class="text-2xl mb-4 relative drop-shadow-lg cursor-pointer"
                onclick={() => goto("/notifications")}
            >
                🔔
                <span
                    class="text-sm text-white w-2 h-2 bg-red-600 rounded-full absolute top-[-4px] translate-x-4 left-0 flex justify-center items-center"
                >
                </span>
            </button>
            <button
                title="profile"
                class="text-2xl mb-4 relative drop-shadow-lg cursor-pointer"
                onclick={() => goto("/profile")}
            >
                👤
            </button>
            <button
                title="logout"
                class="text-2xl mb-4 relative drop-shadow-lg cursor-pointer"
                onclick={() => goto("/login")}
            >
                ↪️
            </button>
        </div>
    </div>
    {#if activeFriendName && activeFriendAddress}
        <div class="flex-1 h-full rounded p-4 py-0 flex flex-col gap-2">
            <h2 class="text-2xl mb-2 relative select-none">Messages</h2>
            <ChatMessages {messages} />
            <span
                class="flex justify-between gap-2 items-center h-16 mt-4 select-none"
            >
                {#if fileIsChosen && files}
                    {files[0].name}
                {:else}
                    <textarea
                        rows="1"
                        class="flex-1 p-1 border rounded-lg outline-none drop-shadow-sm"
                        placeholder="Your message here"
                        style="resize: none;"
                        bind:value={textMessage}
                    ></textarea>
                {/if}
                <form
                    action=""
                    onsubmit={sendSubmitEvent}
                    class="flex items-center gap-2"
                >
                    <div style="transform: rotateY(180deg);">
                        <label class="cursor-pointer">
                            <input
                                type="file"
                                oncancel={() => (fileIsChosen = false)}
                                onclick={() => (fileValue = null)}
                                bind:value={fileValue}
                                bind:files
                                style="display: none;"
                            />
                            📎
                        </label>
                    </div>
                    <div>
                        <button
                            class="bg-blue-600/[90%] relative text-white p-2 rounded-full drop-shadow-lg group"
                        >
                            Send Message ➤
                        </button>
                    </div>
                </form>
            </span>
        </div>
    {:else}
        <div class="flex flex-1 justify-center items-center text-4xl">
            Click on a chat
        </div>
    {/if}
</div>

<!-- hidden -> flex-->
<div
    class="fixed inset-0 bg-black/[0.6] {showFriendAdder
        ? 'flex'
        : 'hidden'} justify-center items-center"
>
    <div class="bg-white w-[50%] max-w-[50%] rounded p-4 text-sm">
        <form action="" class="flex flex-col gap-4">
            <label for="name" class="flex flex-col gap-2">
                Name
                <input
                    bind:value={friendAdderName}
                    class="border p-2 outline-none text-xl"
                    placeholder="Jon Doe"
                    type="text"
                    name="name"
                />
            </label>
            <label for="address" class="flex flex-col gap-2">
                Account
                <input
                    bind:value={friendAdderAddress}
                    class="border p-2 outline-none text-xl"
                    placeholder="0x000000000a10123ef2e2aab1c9"
                    type="text"
                    name="address"
                />
            </label>
            <div class="pt-4 flex justify-between">
                <button
                    class="bg-green-600 text-white p-1 px-4 rounded text-xl"
                    onclick={addFriend}>Add friend</button
                >
                <button
                    class="bg-gray-600 text-white p-1 px-4 rounded text-xl"
                    onclick={() => (showFriendAdder = false)}>Cancel</button
                >
            </div>
        </form>
    </div>
</div>

<!-- 

    <span
        class="text-sm group-hover:block text-gray-400 w-60 absolute top-[25%] left-0 translate-x-[-100%] hidden"
    >
        first copy the message to clipboard
    </span>

-->
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
</style>
