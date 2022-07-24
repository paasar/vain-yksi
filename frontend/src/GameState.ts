import { writable, type Writable } from 'svelte/store';

class Game {
  id?: string
  gameStarted: boolean = false
  word?: string
  player?: PlayerData
  otherPlayers: PlayerData[] = []
  hints: Hint[] = []
  duplicateHints: Hint[] = []
}

export let game = writable(new Game());

export class AllHints {
    duplicates: Hint[]
    hints: Hint[]
}

export class AllHintsToGuesser {
    hints: Hint[]
    usersWithDuplicates: PlayerId[]
}

export class Hint {
    client: PlayerId
    hint: string

    constructor(client: PlayerId, hint: string) {
        this.client = client;
        this.hint = hint;
    }
}

export class HintReceived {
    client: PlayerId
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
  id: PlayerId
  username: string
}

export class PlayerQuit {
  id: PlayerId
}
  
export class YourData {
  id: PlayerId
  username: string
}

export class PlayerData {
  id: PlayerId
  username: string
  hintGiven: boolean = false;
  guesser: boolean = false;

  constructor(id: PlayerId, username: string) {
      this.id = id;
      this.username = username;
  }
}

export type PlayerId = string;