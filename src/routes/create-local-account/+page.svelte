<script lang="ts">
    import { privateKeyToAccount } from "viem/accounts";
    import { invoke } from "@tauri-apps/api/core";
    import { goto } from "$app/navigation";

    let username = $state("");
    let password = $state("");
    let privateKey = $state("");
    let termsAgreed = $state(false);

    let address = $derived.by(() => {
        try {
            return privateKeyToAccount(privateKey as `0x${string}`);
        } catch (e) {
            // no-op
        }
    });

    async function createAccount(e: SubmitEvent) {
        e.preventDefault();
        await invoke("create_account", {
            payload: {
                username,
                password,
                privateKey,
            },
        });
        goto("/dashboard");
    }

    let validFormParameters = $derived(
        address != null && username != "" && password != "" && termsAgreed,
    );
</script>

<div class="flex justify-center items-center h-screen select-none">
    <div>
        <h4 class="mb-8 text-8xl font-medium text-center">🤷‍♂️</h4>
        <form
            action=""
            onsubmit={createAccount}
            class="flex flex-col gap-4 relative"
        >
            <div class="flex gap-4 justify-between">
                <label
                    class="text-xl"
                    for="username"
                    spellcheck="false"
                    data-gramm="false">Username</label
                >
                <input
                    bind:value={username}
                    class="border border-gray-200 pl-1 outline-none"
                    type="text"
                />
            </div>
            <div class="flex gap-4 justify-between">
                <label class="text-xl" for="password">Password</label>
                <input
                    bind:value={password}
                    class="border border-gray-200 pl-1 outline-none"
                    type="password"
                />
            </div>
            <div class="flex gap-4 justify-between">
                <label class="text-xl" for="password">Private Key</label>
                <input
                    bind:value={privateKey}
                    class="border border-gray-200 pl-1 outline-none"
                    type="password"
                />
            </div>

            <div class="flex gap-4 mt-2 items-center">
                <div class="relative top-[-2px]">
                    <input bind:checked={termsAgreed} type="checkbox" />
                </div>
                <label for="t&c" class="text-sm">
                    I understand this cannot be recovered once lost.
                </label>
            </div>
            <div class="flex justify-center items-center">
                <button
                    class="border bg-green-600 text-white py-1 px-4 rounded disabled:cursor-not-allowed disabled:bg-black/60"
                    disabled={!validFormParameters}
                >
                    Create Account
                </button>
            </div>
        </form>
        {#if address}
            <div class="text-sm text-center pt-8 absolute">
                Your account: {address.address}
            </div>
        {/if}
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
</style>
