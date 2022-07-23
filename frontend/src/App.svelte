<script lang="ts">
    import { game } from './GameState';
    import Lobby from './components/Lobby.svelte';
    import Login from './components/Login.svelte';
</script>

<main>
  <h1>Vain Yksi</h1>
  <div class="game-info">
    {#if $game.id}
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
