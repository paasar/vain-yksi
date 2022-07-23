import { PlayerData, type NewGame, type PlayerJoin, type YourData } from './GameState';
import { game } from './GameState';

let socket: WebSocket;

// TODO EVENTS:
// TODO start next round (guesser, hinter)
// TODO hint received
// TODO all hints received (guesser, hinter)
// TODO result (correct, incorrect)
enum EventType {
  NEW_GAME = "new_game",
  PLAYER_JOIN = "join",
  YOUR_DATA = "your_data",
}

interface Event {
  event: EventType
  payload: NewGame | PlayerJoin | YourData
}

export function createGame(username: string) {
  socket = new WebSocket(`ws://localhost:8000/ws/new/${username}`);

  addSocketHandlers(socket);
};

export function joinGame(gameIdToJoin: string, username: string) {
  socket = new WebSocket(`ws://localhost:8000/ws/join/${gameIdToJoin}/${username}`);

  game.update(g => {g.id = gameIdToJoin; return g;});
  addSocketHandlers(socket);
};

function addSocketHandlers(mySocket: WebSocket) {
  mySocket.onopen = function(e) {
    console.log("[open] Connection established");
  };

  mySocket.onmessage = function(event) {
    console.log(`[message] Data received from server: ${event.data} ${typeof event.data}`);
    let receivedEvent: Event = JSON.parse(event.data);

    console.log("Event", receivedEvent.event, receivedEvent.payload.id);

    switch (receivedEvent.event) {
      case EventType.NEW_GAME:
        let newGame = receivedEvent.payload as NewGame;
        console.log('NewGame event!', newGame.id);
        game.update(g => {g.id = newGame.id; return g;});
        break;
      case EventType.PLAYER_JOIN:
        let userJoin = receivedEvent.payload as PlayerJoin;
        console.log('Join event!', userJoin.id, userJoin.name);
        let otherPlayerData = new PlayerData(userJoin.id, userJoin.name);
        game.update(g => {g.allPlayers.push(otherPlayerData); return g;})
        break;
      case EventType.YOUR_DATA:
        let yourData = receivedEvent.payload as YourData;
        let playerData = new PlayerData(yourData.id, yourData.username);
        game.update(g => {g.player = playerData; return g;});
        break;
      default: console.log("Unknown event:", typeof receivedEvent);
    }
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
    console.log(`[error] ${error}`);
  };
}