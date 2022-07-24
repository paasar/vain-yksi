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

<button on:click={() => startNextRound()}>Aloita peli</button>

<div>
    <h2>Liittyneet pelaajat</h2>
<ul>
{#each $game.otherPlayers as player}
    <li>{player.username} ({player.id})</li>
{/each}
</ul>
</div>