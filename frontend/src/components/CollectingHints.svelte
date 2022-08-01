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
    <div class="player-state-card">
        <div class="state">
            {#if player.guesser}
                ARVAA
            {:else}
                {#if player.hintGiven}
                    VALMIS
                {:else}
                    Vinkkaa
                {/if}
            {/if}
        </div>
        <div class="username">
            {player.username}
        </div>
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

    .player-state-card {
        background-color: var(--light-gray);
        border: solid 2px var(--main);
        border-radius: 10px;
        color: var(--lila);
        display: flex;
        margin: 0.5em 0;
    }

    .state, .username {
        display: inline-block;
        padding: 0.5em;
    }

    .state {
        width: 60px;
        border-right: solid 1px var(--main);
    }

    .username {
        flex-grow: 1;
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