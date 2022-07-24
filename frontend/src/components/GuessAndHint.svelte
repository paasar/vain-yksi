<script lang="ts">
    import { game } from '../GameState';
</script>

<div>
{#if $game.player && $game.player.guesser }
    Sinä arvaat! Odota vinkkejä.
{:else}
    <div>
    Anna vinkki sanalle: <div class="word-to-guess">{$game.word}</div>
    </div>
    <label for="hint">Vinkki</label>
    <input id="hint" />
    <button>Lähetä vinkki</button>
{/if}
</div>

<h2>Muut</h2>
<div>
{#each $game.otherPlayers as player}
    <div>
    {player.username}
    {#if player.guesser}
        arvaamisvuorossa
    {:else}
        {#if player.hintGiven}
            Vinkki annettu
        {:else}
            Odotellaan vinkkiä
        {/if}
    {/if}
    </div>
{/each}
</div>

<style>
    .word-to-guess {
        font-size: larger;
        font-weight: 700;
    }
</style>