<script lang="ts">
    import { game } from '../GameState';
    import { sendHint, skipWord, startNextRound } from '../WebSocket';

    let hint;
</script>

<div>
{#if $game.player && $game.player.guesser }
    Sinä arvaat! Odota vinkkejä.
{:else if $game.player && !$game.player.hintGiven}
    <div>
    Anna vinkki sanalle: <div class="word-to-guess">{$game.word}</div>
    </div>
    <label for="hint">Vinkki</label>
    <input id="hint" bind:value={hint}/>
    <button on:click={() => sendHint(hint)} disabled={!hint}>Lähetä vinkki</button>
{:else}
    Odotellaan muita.
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

<div>
    {#if $game.player && !$game.player.guesser }
        <button on:click={() => skipWord()}>Vaihda sana</button>
    {/if}
    <button on:click={() => startNextRound()}>Vaihda arvaaja ja sana</button>
</div>

<style>
    .word-to-guess {
        font-size: larger;
        font-weight: 700;
    }
</style>