<script lang="ts">
    import { createGame, joinGame } from '../WebSocket';
    let username;
    let gameIdToJoin;
</script>

<div>
    <label for="username">Nimi</label>
    <input id="username" type="text" maxlength="30" bind:value={username} />
    <div>
        <button on:click={() => createGame(username)} disabled={!username}>Luo uusi peli</button>
    </div>
</div>
<div>
    <label for="gameIdToJoin">Pelikoodi</label>
    <input id="gameIdToJoin" type="number" minlength="4" maxlength="4" min="1001" max="9999" bind:value={gameIdToJoin} />
    <div>
        <button on:click={() => joinGame(gameIdToJoin, username)} disabled={!username || !gameIdToJoin}>Liity peliin</button>
    </div>
</div>

<style>
    div {
        margin: 1em;
    }
</style>