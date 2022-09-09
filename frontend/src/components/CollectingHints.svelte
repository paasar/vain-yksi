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
            <input id="hint" bind:value={hint}
                             on:keydown={(e) => {if (e.key === "Enter" && hint) sendHint(hint)}}/>
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
    <div class="player-state-card" class:guesser={player.guesser}>
        <div class="username">
            {player.username}
        </div>
        <div class="state" class:ready={player.hintGiven}>
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
    </div>
{:else}
    <div>Ei muita pelaajia.</div>
{/each}
</div>

<div class="footer">
    <div class="footer-content">
        {#if $game.player && !$game.player.guesser }
        <button on:click={() => {
            if (!confirm("Haluatko varmasti vaihtaa sanan?")) {
              return;
            }
            skipWord();
          }
        }>Vaihda sana</button>
        {/if}
        <button on:click={() => {
            if (!confirm("Haluatko varmasti vaihtaa arvaajan ja sanan?")) {
              return;
            }
            startNextRound();
          }
        }>Vaihda arvaaja ja sana</button>
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

    .guesser .state {
        background-color: var(--main);
        color: var(--main-complement);
    }

    .state {
        border-radius: 0 7px 7px 0;
        width: 60px;
    }

    .state.ready {
        background-color: var(--main);
        color: var(--light-gray);
    }

    .username {
        border-right: solid 1px var(--main);
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