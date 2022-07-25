<script lang="ts">
    import { createGame, joinGame } from '../WebSocket';
    let username;
    let gameIdToJoin;
</script>

<div class="row">
    <div>
        <label for="username">Nimi</label>
        <input id="username" type="text" maxlength="30" bind:value={username} />
    </div>
    <button on:click={() => createGame(username)} disabled={!username}>Luo uusi peli</button>
</div>
<div class="row">
    <div>
        <label for="gameIdToJoin">Pelikoodi</label>
        <input id="gameIdToJoin" type="number" minlength="4" maxlength="4" min="1001" max="9999" bind:value={gameIdToJoin} />
    </div>
    <button on:click={() => joinGame(gameIdToJoin, username)} disabled={!username || !gameIdToJoin}>Liity peliin</button>
</div>

<style>
    .row {
        display: flex;
        justify-content: space-between;
        align-items: flex-end;
    }

    button {
        height: 3em;
        margin-left: 1em;
        min-width: 200px;
    }

    label {
        display: block;
        text-align: left;
    }

    input {
        border: solid #bbb 1px;
        border-radius: 5px;
        display: block;
        font-size: 1.1em;
        margin-top: 5px;
        min-width: 200px;
        padding: 5px;
    }
</style>