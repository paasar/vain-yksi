<script lang="ts">
    import { game } from './GameState';
    import CollectingHints from './components/CollectingHints.svelte';
    import Guessing from './components/Guessing.svelte';
    import Lobby from './components/Lobby.svelte';
    import Login from './components/Login.svelte';

    import { fly } from 'svelte/transition';

    let flyIn = {x: 50, duration: 500, delay: 510};
    let flyOut = {x: -50, duration: 500};
</script>

<div class="content">
  <main>
    <h1 class:started="{$game.id}">Vain Yksi</h1>
    <div class="game-info">
      {#if $game.hints.length > 0 || $game.duplicateHints.length > 0}
        <div out:fly="{flyOut}" in:fly="{flyIn}">
          <Guessing />
        </div>
      {:else if $game.gameStarted}
        <div out:fly="{flyOut}" in:fly="{flyIn}">
          <CollectingHints />
        </div>
      {:else if $game.id}
        <div out:fly="{flyOut}" in:fly="{flyIn}">
          <Lobby />
        </div>
      {:else}
        <div out:fly="{flyOut}">
          <Login />
        </div>
      {/if}
    </div>
  </main>
</div>

<style>
  .content {
    margin: 0 auto;
  }

  .game-info {
    font-weight: bold;
    margin-top: 40px;
  }

  .started {
    font-size: 1.5em;
  }
</style>
