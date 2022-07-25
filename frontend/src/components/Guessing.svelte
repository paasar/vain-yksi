<script lang="ts">
import { game, type PlayerId } from "../GameState";
import { sendGuess, startNextRound } from '../WebSocket';

let guesser = $game.otherPlayers.filter(player => player.guesser).at(0);

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

<h2>Vinkit</h2>
{#each $game.hints as hint}
    <div>
        {hint.hint} ({username(hint.client)})
    </div>
{:else}
    -
{/each}

{#if $game.duplicateHints.length > 0}
    <h2>Duplikaatin vinkin antoivat</h2>
    {#each $game.duplicateHints as hint}
        <div>
            {username(hint.client)}
        </div>
    {/each}
{/if}

{#if $game.result}
    <div>Arvaus meni <span class="emphasis">{#if $game.result.correct}oikein{:else}väärin{/if}!</span></div>

    <div>Sana oli <span class="emphasis">{$game.result.word}</span>.</div>
    {#if !$game.result.correct}
        Arvaus oli <span class="emphasis">{$game.result.guess}</span>.
    {/if}

    <div>
        <button on:click={() => startNextRound()}>Aloita uusi kierros</button>
    </div>
{:else}
    {#if $game.player.guesser}
        <input id="guess" bind:value={guess} />
        <button on:click={() => sendGuess(guess)} disabled={!guess}>Arvaa!</button>
    {:else}
        Odotetaan, että {guesser.username} arvaa.
    {/if}
{/if}

<style>
    .emphasis {
        font-size: larger;
        font-weight: 700;
    }
</style>