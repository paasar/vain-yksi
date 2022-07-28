<script lang="ts">
import { game, type PlayerId } from "../GameState";
import { sendGuess, startNextRound } from '../WebSocket';

import { fade } from 'svelte/transition';

let guesser = $game.otherPlayers.filter(player => player.guesser).at(0);
let guesserUsername = guesser ? guesser.username : '...';

let guess;

function username(id: PlayerId) {
    let matchingClients = $game.otherPlayers.filter(player => player.id === id);
    if (matchingClients.length === 1) {
        return matchingClients.at(0).username;
    } else if ($game.player.id === id) {
        return $game.player.username;
    } else {
        return "N/A";
    }
}
</script>

{#if $game.duplicateHints.length > 0}
    <h2>Duplikaatit vinkit</h2>
    {#each $game.duplicateHints as hint}
        {#if !$game.player.guesser}
            <div class="hint-card duplicate">
                <div class="username">
                    {username(hint.client)}
                </div>
                <div class="hint">
                    {hint.hint}
                </div>
            </div>
        {:else}
        <div class="duplicate">
            {username(hint.client)}
        </div>
        {/if}
    {/each}
{/if}

<h2>Vinkit</h2>
{#each $game.hints as hint}
    <div class="hint-card">
        <div class="username">
            {username(hint.client)}
        </div>
        <div class="hint">
            {hint.hint}
        </div>
    </div>
{:else}
    -
{/each}

{#if $game.result}
    <div class="result" in:fade="{{duration: 500, delay: 500}}">
        <div>Arvaus meni <span class="emphasis">{#if $game.result.correct}oikein{:else}väärin{/if}!</span></div>

        <div>Sana oli <span class="emphasis">{$game.result.word}</span>.</div>
        {#if !$game.result.correct}
            Arvaus oli <span class="emphasis">{$game.result.guess}</span>.
        {/if}

        <div class="next-round">
            <button on:click={() => startNextRound()}>Aloita uusi kierros</button>
        </div>
    </div>
{:else}
    <div class="row" out:fade="{{duration: 500}}">
    {#if $game.player.guesser}
        <div>
            <label for="guess">Arvaus</label>
            <input id="guess" bind:value={guess} />
        </div>
        <button class="button-guess" on:click={() => sendGuess(guess)} disabled={!guess}>Arvaa!</button>
    {:else}
        Odotetaan, että {guesserUsername} arvaa.
    {/if}
</div>
{/if}

<style>
    .emphasis {
        font-size: larger;
        font-weight: 700;
    }

    .button-guess {
        margin-left: 1em;
        margin-top: 0.5em;
    }

    .hint-card {
        background-color: #dddddd;
        border: solid 2px var(--main);
        border-radius: 10px;
        color: var(--lila);
        display: flex;
        flex-direction: column;
        align-items: stretch;
        margin: 0.5em 0;
    }

    .hint {
        display: inline-block;
        border-top: solid 1px var(--main);
        font-size: 1.2em;
        padding: 0.5em;
    }

    .username {
        font-size: 0.8em;
        padding: 0 0.2em;
    }

    .duplicate {
        background-color: #bb4040;
        border-radius: 10px;
        color: var(--light-gray);
        margin: 0.5em 0;
    }

    .result, .next-round {
        margin-top: 1.5em;
    }
</style>