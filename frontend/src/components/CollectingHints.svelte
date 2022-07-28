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
    <div class="row">
        <div>
            <label for="hint">Vinkki</label>
            <input id="hint" bind:value={hint}/>
        </div>
        <button on:click={() => sendHint(hint)} disabled={!hint}>Lähetä vinkki</button>
    </div>
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
{:else}
    <div>Ei muita pelaajia.</div>
{/each}
</div>

<div class="footer">
    <div class="footer-content">
        {#if $game.player && !$game.player.guesser }
        <button on:click={() => skipWord()}>Vaihda sana</button>
        {/if}
        <button on:click={() => startNextRound()}>Vaihda arvaaja ja sana</button>
    </div>
</div>

<style>
    .word-to-guess {
        font-size: larger;
        font-weight: 700;
    }

    button {
        margin-left: 0.5em;
        margin-top: 0.5em;
    }

    .footer {
        border-top: solid 1px var(--main);
        margin-top: 100px;
        padding: 0 2rem;
        width: calc(100% - 2.5rem);
    }

    .footer-content {
        display: flex;
        justify-content: space-evenly;
        align-items: center;
        margin: 0 auto;
        max-width: 800px;
        padding: 0.5em 0;
    }
</style>