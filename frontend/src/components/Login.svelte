<script lang="ts">
    import { createGame } from '../WebSocket';
    let username;
</script>

<div>
    <label for="username">Nimi</label>
    <input id="username" type="text" maxlength="30" bind:value={username} />
    <div>
        <button on:click={() => createGame(username)}>Luo uusi peli</button>
    </div>
</div>

<style>
    div {
        margin: 1em;
    }
</style>