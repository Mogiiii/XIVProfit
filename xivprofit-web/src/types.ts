export interface Item {
  id: number;
  name: string;
}

export interface Recipe {
  id: number,
  result_item_id: number,
  result_item_quantity: number,
  //itemid, quantity
  ingredients: Array<[number, number]>,
}


export interface ItemListing {
  item_id: number,
  world_id: number,
  price_per_unit: number,
  quantity: number,
  total_price: number,
  hq: boolean,
  retainer_name: String,
}

export interface datacenter {
  name: string;
  region: string;
  worlds: number[];
}

export interface world {
  name: string;
  id: number
}