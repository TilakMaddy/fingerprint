<script>
    import splash from "$lib/assets/splash-image.webp";
</script>

<div class="max-h-[100%] overflow-clip">
    <img
        draggable="false"
        src={splash}
        alt="Man trying to remember his passwords"
    />
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
