import { writable } from 'svelte/store';

let socket: WebSocket;

export let gameId = writable('');

// TODO EVENTS:
// TODO player join
// TODO start next round (guesser, hinter)
// TODO hint received
// TODO all hints received (guesser, hinter)
// TODO result (correct, incorrect)
interface Event {
  event: string,
  payload: NewGame
}

class NewGame {
  id: string
}

export function createGame(username: string) {
  socket = new WebSocket(`ws://localhost:8000/ws/new/${username}`);

  socket.onopen = function(e) {
    console.log("[open] Connection established");
  };

  socket.onmessage = function(event) {
    console.log(`[message] Data received from server: ${event.data} ${typeof event.data}`);
    let receivedEvent: Event = JSON.parse(event.data);

    console.log("Event", receivedEvent.event, receivedEvent.payload.id);

    gameId.set(receivedEvent.payload.id);
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