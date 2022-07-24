import { PlayerData, type NewGame, type NewRound, type OtherPlayers, type PlayerJoin, type PlayerQuit, type YourData } from './GameState';
import { game } from './GameState';

let socket: WebSocket;

// TODO EVENTS:
// TODO start next round (guesser, hinter)
// TODO hint received
// TODO all hints received (guesser, hinter)
// TODO result (correct, incorrect)
enum EventType {
  NEW_GAME = "new_game",
  NEW_ROUND = "new_round",
  OTHER_PLAYERS = "other_players",
  PLAYER_JOIN = "join",
  PLAYER_QUIT = "quit",
  YOUR_DATA = "your_data",
}

interface Event {
  event: EventType
  payload: NewGame | NewRound | OtherPlayers | PlayerJoin | YourData
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

export function startNextRound() {
  console.log("Starting next round");
  socket.send(JSON.stringify({"action": {"start_next_round": true}}));
}

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
        console.log('NewGame event!', newGame.id);
        game.update(g => {g.id = newGame.id; return g;});
        break;
      case EventType.NEW_ROUND:
        let newRound = receivedEvent.payload as NewRound;
        console.log('NewRound event!', newRound.role, newRound.word);
        game.update(g => {g.word = newRound.word; g.player.guesser = newRound.role === "guesser"; return g;});
        game.update(g => {g.otherPlayers = g.otherPlayers.map(player => {
            if (player.id === newRound.guesser) {
              player.guesser = true;
            } else {
              player.guesser = false;
            }
            return player;
          });
          return g;
        });
        game.update(g => {g.gameStarted = true; return g;});
        break;
      case EventType.OTHER_PLAYERS:
        let otherPlayers = receivedEvent.payload as OtherPlayers;
        console.log('Other players', otherPlayers);
        game.update(g => {g.otherPlayers = otherPlayers; return g;})
        break;
      case EventType.PLAYER_JOIN:
        let playerJoin = receivedEvent.payload as PlayerJoin;
        console.log('Join event!', playerJoin.id, playerJoin.username);
        let otherPlayerData = new PlayerData(playerJoin.id, playerJoin.username);
        game.update(g => {g.otherPlayers.push(otherPlayerData); return g;});
        break;
      case EventType.PLAYER_QUIT:
        let playerQuit = receivedEvent.payload as PlayerQuit;
        console.log('Quit event!', playerQuit.id);
        game.update(g => {g.otherPlayers = g.otherPlayers.filter(p => p.id !== playerQuit.id); return g;});
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