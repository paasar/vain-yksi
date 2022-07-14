<script lang="ts">
    import {onMount} from 'svelte';
    let socket;

    let gameId;

    // TODO EVENTS:
    // TODO player join
    // TODO start next round (guesser, hinter)
    // TODO hint received
    // TODO all hints received (guesser, hinter)
    // TODO result (correct, incorrect)
    interface Event {
      event: String,
      payload: NewGame
    }

    class NewGame {
      id: String
    }

    function createGame() {
      socket = new WebSocket("ws://localhost:8000/ws/new/user1");

      socket.onopen = function(e) {
        console.log("[open] Connection established");
        console.log("Sending to server");
        socket.send("My name is John");
      };

      socket.onmessage = function(event) {
        console.log(`[message] Data received from server: ${event.data} ${typeof event.data}`);
        let receivedEvent: Event = JSON.parse(event.data);

        console.log("Event", receivedEvent.event, receivedEvent.payload.id);

        gameId = receivedEvent.payload.id;
      };

      socket.onclose = function(event) {
        if (event.wasClean) {
          console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
          // e.g. server process killed or network down
          // event.code is usually 1006 in this case
          console.log('[close] Connection died');
        }
      };

      socket.onerror = function(error) {
        console.log(`[error] ${error.message}`);
      };
    };
</script>

<main>
  <h1>Vain Yksi</h1>
  <div class="game-info">
    {#if gameId}
      Pelin tunnus: {gameId}
    {/if}
  </div>

  <div>
    <button on:click={createGame}>Luo uusi peli</button>
  </div>
</main>

<style>
  .game-info {
    font-weight: bold;
    margin: 1rem;
  }
</style>
