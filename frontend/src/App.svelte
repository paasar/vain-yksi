<script lang="ts">
    import { game } from './GameState';
    import Login from './components/Login.svelte';
</script>

<main>
  <h1>Vain Yksi</h1>
  <div class="game-info">
    {#if $game.id}
      Pelin tunnus: {$game.id}
      {#if $game.player}
      Sinä: {$game.player.username}, ID: {$game.player.id}
      {/if}
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
