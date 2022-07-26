<script type="ts">
    import { game } from '../GameState';
    import { startNextRound } from '../WebSocket';
</script>

<div>
    Pelin tunnus: {$game.id}
    {#if $game.player}
    Sinä: {$game.player.username}, ID: {$game.player.id}
    {/if}
</div>

<button on:click={() => startNextRound()} disabled={$game.otherPlayers.length < 1}>Aloita peli</button>

<div>
    <h2>Liittyneet pelaajat</h2>
    {#each $game.otherPlayers as player}
        <div>{player.username} ({player.id})</div>
    {:else}
        <div>Ei vielä muita pelaajia.</div>
    {/each}
</div>