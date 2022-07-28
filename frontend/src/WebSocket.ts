import {
  Hint,
  PlayerData,
  Result,
  type AllHints,
  type AllHintsToGuesser,
  type GuessResult,
  type HintReceived,
  type NewGame,
  type NewRound,
  type OtherPlayers,
  type PlayerJoin,
  type PlayerQuit,
  type YourData,
  resetStateForNextRound } from './GameState';
import { game } from './GameState';

let domain = window.location.hostname;
let socket: WebSocket;

enum EventType {
  ALL_HINTS = "all_hints",
  ALL_HINTS_TO_GUESSER = "all_hints_to_guesser",
  GUESS_RESULT = "guess_result",
  HINT_RECEIVED = "hint_received",
  NEW_GAME = "new_game",
  NEW_ROUND = "new_round",
  OTHER_PLAYERS = "other_players",
  PLAYER_JOIN = "join",
  PLAYER_QUIT = "quit",
  YOUR_DATA = "your_data",
}

interface Event {
  event: EventType
  payload: AllHints |
           AllHintsToGuesser |
           GuessResult |
           HintReceived |
           NewGame |
           NewRound |
           OtherPlayers |
           PlayerJoin |
           YourData
}

export function createGame(username: string) {
  // TODO port in dev vs. prod?
  // TODO wss instead of ws in prod?
  socket = new WebSocket(`ws://${domain}:8000/ws/new/${username}`);

  addSocketHandlers(socket);
};

export function joinGame(gameIdToJoin: string, username: string) {
  socket = new WebSocket(`ws://${domain}:8000/ws/join/${gameIdToJoin}/${username}`);

  game.update(g => {g.id = gameIdToJoin; return g;});
  addSocketHandlers(socket);
};

export function startNextRound() {
  console.log("Starting next round");
  socket.send(JSON.stringify({"action": {"start_next_round": true}}));
}

export function sendHint(hint: string) {
  console.log("Sending hint", hint);
  socket.send(JSON.stringify({"action": {"hint": hint}}));
  game.update(g => {g.player.hintGiven = true; return g;});
}

export function sendGuess(guess: string) {
  console.log("Sending guess", guess);
  socket.send(JSON.stringify({"action": {"guess": guess}}));
}

export function skipWord() {
  console.log("Skip word");
  socket.send(JSON.stringify({"action": {"skip_word": true}}));
}

function addSocketHandlers(mySocket: WebSocket) {
  mySocket.onopen = function(e) {
    console.log("[open] Connection established");
  };

  mySocket.onmessage = function(event) {
    console.log(`[message] Data received from server: ${event.data} ${typeof event.data}`);
    let receivedEvent: Event = JSON.parse(event.data);

    switch (receivedEvent.event) {
      case EventType.ALL_HINTS:
        let allHints = receivedEvent.payload as AllHints;
        game.update(g => {
          g.hints = allHints.hints;
          g.duplicateHints = allHints.duplicates;
          return g;
        });
        break;
      case EventType.ALL_HINTS_TO_GUESSER:
        let allHintsToGuesser = receivedEvent.payload as AllHintsToGuesser;
        game.update(g => {
          g.hints = allHintsToGuesser.hints;
          g.duplicateHints = allHintsToGuesser.usersWithDuplicates
            .map(dup => new Hint(dup, ''));
          return g;
        });
        break;
      case EventType.GUESS_RESULT:
        let result = receivedEvent.payload as GuessResult;
        game.update(g => {g.result = new Result(result); return g;});
        break;
      case EventType.HINT_RECEIVED:
        let hintReceived = receivedEvent.payload as HintReceived;
        game.update(g => {g.otherPlayers = g.otherPlayers.map(player => {
            if (player.id === hintReceived.client) {
              player.hintGiven = true;
            }
            return player;
          });
          return g;});
        break;
      case EventType.NEW_GAME:
        let newGame = receivedEvent.payload as NewGame;
        console.log('NewGame event!', newGame.id);
        game.update(g => {g.id = newGame.id; return g;});
        break;
      case EventType.NEW_ROUND:
        let newRound = receivedEvent.payload as NewRound;
        console.log('NewRound event!', newRound.role, newRound.word);
        resetStateForNextRound();
        game.update(g => {
          g.word = newRound.word;
          g.player.guesser = newRound.role === "guesser";
          g.player.hintGiven = false;
          return g;
        });
        game.update(g => {g.otherPlayers = g.otherPlayers.map(player => {
            if (player.id === newRound.guesser) {
              player.guesser = true;
            } else {
              player.guesser = false;
            }
            player.hintGiven = false;
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