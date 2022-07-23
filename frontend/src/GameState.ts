import { writable, type Writable } from 'svelte/store';

class Game {
  id?: string
  word?: string
  player?: PlayerData
  allPlayers: PlayerData[] = []
}

export let game = writable(new Game());

export class NewGame {
   id: string
}
  
export class PlayerJoin {
  id: string
  name: string
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