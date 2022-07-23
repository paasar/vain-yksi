import { writable, type Writable } from 'svelte/store';

class Game {
  id?: string
  word?: string
  player?: YourData
  allPlayers: PlayerData[] = []
}

export let game = writable(new Game());

export class NewGame {
   id: string
}
  
export class UserJoin {
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
  hintGiven?: boolean
}