<script type="ts">
    import { game } from '../GameState';
    import { startNextRound } from '../WebSocket';

    import { fade, fly } from 'svelte/transition';
</script>

<div>
    <h3>Pelin tunnus</h3>
    <div class="game-id">{$game.id}</div>
    {#if $game.player}
    Pelaat nimellä <div class="username">{$game.player.username}</div>
    {/if}
</div>

<button on:click={() => startNextRound()} disabled={$game.otherPlayers.length < 1}>Aloita peli</button>

<div>
    <h2>Muut pelaajat ({$game.otherPlayers.length})</h2>
    {#each $game.otherPlayers as player (player.id)}
        <div class="player-card" in:fly="{{x: 50, duration: 500}}" out:fade="{{duration: 1000}}">{player.username}</div>
    {:else}
        <div>Ei vielä muita pelaajia.</div>
    {/each}
</div>

<style>
    h3 {
        margin-bottom: 10px;
    }

    .username {
        display: inline-block;
        font-size: 1.5em;
        margin-top: 0;
    }

    .game-id {
        border: solid 4px var(--main);
        border-radius: 5px;
        font-size: 2em;
        margin-top: 0;
        margin-bottom: 2rem;
        padding: 0.4em;
    }

    button {
        margin-top: 1rem;
    }

    .player-card {
        background-color: var(--main);
        border-radius: 1em;
        box-shadow: 4px 4px 5px 0px rgba(0,0,0,0.7) inset;
        -webkit-box-shadow: 4px 4px 5px 0px rgba(0,0,0,0.7) inset;
        -moz-box-shadow: 4px 4px 5px 0px rgba(0,0,0,0.7) inset;
        color: var(--main-complement);
        font-size: 1.3em;
        margin-top: 0.5em;
        padding: 0.5em;
    }
</style>