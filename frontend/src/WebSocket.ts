import { writable, type Writable } from 'svelte/store';

let socket: WebSocket;

export let gameId = writable('');
export let playerData: Writable<YourData> = writable(null);

// TODO
// class Game {
//   word: string
//   player: YourData
//   allPlayers: PlayerData[]
// }

// class PlayerData {
//   id: string
//   username: string
//   hintGiven?: boolean
// }

// TODO EVENTS:
// TODO player join
// TODO start next round (guesser, hinter)
// TODO hint received
// TODO all hints received (guesser, hinter)
// TODO result (correct, incorrect)
enum EventType {
  NEW_GAME = "new_game",
  USER_JOIN = "join",
  YOUR_DATA = "your_data",
}

interface Event {
  event: EventType
  payload: NewGame | UserJoin | YourData
}

class NewGame {
  id: string
}

class UserJoin {
  id: string
  name: string
}

class YourData {
  id: string
  username: string
}

export function createGame(username: string) {
  socket = new WebSocket(`ws://localhost:8000/ws/new/${username}`);

  addSocketHandlers(socket);
};

export function joinGame(gameIdToJoin: string, username: string) {
  socket = new WebSocket(`ws://localhost:8000/ws/join/${gameIdToJoin}/${username}`);

  addSocketHandlers(socket);
};

function addSocketHandlers(mySocket: WebSocket) {
  mySocket.onopen = function(e) {
    console.log("[open] Connection established");
  };

  mySocket.onmessage = function(event) {
    console.log(`[message] Data received from server: ${event.data} ${typeof event.data}`);
    let receivedEvent: Event = JSON.parse(event.data);

    switch (receivedEvent.event) {
      case EventType.NEW_GAME:
        let newGame = receivedEvent.payload as NewGame;
        console.log('2NewGame event!', newGame.id);
        break;
      case EventType.USER_JOIN:
        let userJoin = receivedEvent.payload as UserJoin;
        console.log('2Join event!', userJoin.id, userJoin.name);
        break;
      case EventType.YOUR_DATA:
        let yourData = receivedEvent.payload as YourData;
        playerData.set(yourData);
        break;
      default: console.log("2Unknown event:", typeof receivedEvent);
    }

    console.log("Event", receivedEvent.event, receivedEvent.payload.id);

    gameId.set(receivedEvent.payload.id);
  };

  mySocket.onclose = function(event) {
    if (event.wasClean) {
      console.log(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
    } else {
      // e.g. server process killed or network down
      // event.code is usually 1006 in this case
      console.log('[close] Connection died');
    }
  };

  mySocket.onerror = function(error) {
    console.log(`[error] ${error.message}`);
  };
}