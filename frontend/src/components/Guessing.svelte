<script lang="ts">
import { game, type PlayerId } from "../GameState";

let guesser = $game.otherPlayers.filter(player => player.guesser).at(0);

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


{#if $game.player.guesser}
    <input id="guess" />
    <button>Arvaa!</button>
{:else}
    Odotetaan, että {guesser.username} arvaa.
{/if}