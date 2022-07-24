import { writable, type Writable } from 'svelte/store';

class Game {
  id?: string
  gameStarted: boolean = false
  word?: string
  player?: PlayerData
  otherPlayers: PlayerData[] = []
}

export let game = writable(new Game());

export class HintReceived {
    client: string
}

export class NewGame {
   id: string
}

export class NewRound {
    role: string
    word?: string
    guesser?: string
}

export type OtherPlayers = PlayerData[]

export class PlayerJoin {
  id: string
  username: string
}

export class PlayerQuit {
  id: string
}
  
export class YourData {
  id: string
  username: string
}

export class PlayerData {
  id: string
  username: string
  hintGiven: boolean = false;
  guesser: boolean = false;

  constructor(id: string, username: string) {
      this.id = id;
      this.username = username;
  }
}