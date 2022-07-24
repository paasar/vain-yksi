<script lang="ts">
    import { game } from './GameState';
    import CollectingHints from './components/CollectingHints.svelte';
    import Guessing from './components/Guessing.svelte';
    import Lobby from './components/Lobby.svelte';
    import Login from './components/Login.svelte';
</script>

<main>
  <h1>Vain Yksi</h1>
  <div class="game-info">
    {#if $game.hints.length > 0 || $game.duplicateHints.length > 0}
      <Guessing />
    {:else if $game.gameStarted}
      <CollectingHints />
    {:else if $game.id}
      <Lobby />
    {:else}
      <Login />
    {/if}
  </div>
</main>

<style>
  .game-info {
    font-weight: bold;
    margin: 1rem;
  }
</style>
