<script lang="ts">
    import { gameId } from './WebSocket';
    import Login from './components/Login.svelte';
</script>

<main>
  <h1>Vain Yksi</h1>
  <div class="game-info">
    {#if $gameId}
      Pelin tunnus: {$gameId}
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
