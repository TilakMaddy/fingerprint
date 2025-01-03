<script lang="ts">
    import { goto } from "$app/navigation";
    import { invoke } from "@tauri-apps/api/core";

    let username = $state("");
    let password = $state("");

    async function login() {
        const correctPassword = await invoke("check_if_password_looks_fine", {
            p: {
                username,
                password,
            },
        });

        if (!correctPassword) {
            return;
        }

        await invoke("login", {
            payload: {
                username,
                password,
            },
        });

        // Close the login window and open the main window
        goto("/dashboard");
    }
</script>

<div class="flex justify-center items-center">
    <div class="p-4 select-none">
        <h2 class="text-2xl">Weclome! Please login.</h2>
        <form action="/login" class="flex flex-col mt-8 gap-4">
            <label for="username" class="flex flex-col gap-2">
                Username
                <input
                    bind:value={username}
                    type="text"
                    class="p-1 border outline-none"
                    placeholder="johnny"
                    name="username"
                />
            </label>
            <label for="passwd" class="flex flex-col gap-2">
                Password
                <input
                    bind:value={password}
                    type="password"
                    class="p-1 border outline-none"
                    name="passwd"
                />
            </label>
            <div class="mt-4 flex justify-between">
                <button
                    class="bg-green-600 text-white p-2 px-4 text-sm rounded"
                    onclick={login}>Login</button
                >
                <button class="bg-red-600 text-white p-2 px-4 text-sm rounded"
                    >Reset</button
                >
            </div>
        </form>
        <div class="mt-8">
            <a class="text-sm rounded underline" href="/create-local-account">
                Create Account
            </a>
        </div>
    </div>
</div>

<style lang="postcss">
    :global(html) {
        height: 100%;
    }

    :global(body) {
        position: fixed;
        width: 100%;
        height: 100%;
        overflow: hidden;
    }
</style>
