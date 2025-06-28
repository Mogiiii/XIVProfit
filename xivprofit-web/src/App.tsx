import { useState, useEffect } from "react";
import type { Item, Recipe, datacenter, world } from "./types";
import { datacenters, worlds } from "./data";
import { backendUrl } from "./config";
import { RecipeDisplay } from "./recipeDisplay";
import axios from 'axios';
import './App.css'

type HopType =
  | "WORLD"
  | "DC"
  | "REGION"

function App() {
  const [selectedDc, setSelectedDc] = useState<datacenter>(datacenters[4]); //primal
  const [selectedWorld, setSelectedWorld] = useState<world>(worlds[52]); //ultros
  const [searchedItem, setSearchedItem] = useState<string>("");
  const [craftQuantity, setCraftQuantity] = useState<number>(1);
  const [hq, setHq] = useState<boolean>(false);
  const [craftableItems, setCraftableItems] = useState<Item[]>([]);
  const [items, setItems] = useState<Item[]>([]);
  const [recipes, setRecipes] = useState<Recipe[]>([]);
  const [recipesForSelectedItem, setRecipesForSelectedItem] = useState<Recipe[]>([]);
  const [selectedHopType, setSelectedHopType] = useState<HopType>("WORLD")
  const [selectedItemCurrentSalePriceEach, setSelectedItemCurrentSalePriceEach] = useState<number>(100);

  const getSelectedLocation = () => {
    switch (selectedHopType) {
      case "DC":
        return selectedDc.name;
      case "REGION":
        return selectedDc.region;
      case "WORLD":
      default:
        return selectedWorld.name;
    }
  }

  const handleSelectedDcChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const dc = datacenters.find(dc => dc.name == event.target.value) || datacenters[0]
    setSelectedDc(dc);
    setSelectedWorld(worlds.find(w => w.id == dc.worlds[0]) || worlds[0]);
  };

  const handleSelectedWorldChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedWorld(worlds.find(w => w.name == event.target.value) || worlds[0]);
  };

  const handleSelectedHopTypeChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    setSelectedHopType(event.target.value as HopType)
  }
  const handleSelectedHqChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setHq(event.target.checked);
  };

  //page init
  useEffect(() => {

    axios.get<Item[]>(backendUrl + 'craftable_items')
      .then(response => setCraftableItems(response.data))
      .catch(error => console.error('Error fetching craftable items:', error));

    axios.get<Item[]>(backendUrl + 'items')
      .then(response => setItems(response.data))
      .catch(error => console.error('Error fetching items:', error));

    axios.get<Recipe[]>(backendUrl + 'recipes')
      .then(response => setRecipes(response.data))
      .catch(error => console.error('Error fetching recipes:', error));
  }, []);

  const ItemSearch = () => {
    const [suggestions, setSuggestions] = useState<Array<string>>([]);
    const [searchInput, setSearchInput] = useState<string>(searchedItem);
    const [craftQuantityInterior, setCraftQuantityInterior] = useState<number>(craftQuantity);


    const handleSearchInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
      const value = event.target.value;
      setSearchInput(value);

      // Filter options based on input value
      const filtered = craftableItems.map(i => i.name).filter((option) =>
        option.toLowerCase().includes(value.toLowerCase())
      );
      setSuggestions(filtered);
    };

    const handleSearchQuantityChange = (event: React.ChangeEvent<HTMLInputElement>) => {
      setCraftQuantityInterior(parseInt(event.target.value))
    }

    const handleSearchInputSubmit = (_event: React.ChangeEvent<HTMLFormElement>) => {
      if (searchInput != "") {
        const item = craftableItems.find(i => i.name == searchInput)
        if (item != undefined) {
          setSearchedItem(searchInput);
          setRecipesForSelectedItem(recipes.filter(r => r.result_item_id == item.id))
          axios.get<number>(backendUrl + 'saleprice?item_id=' + item.id + '&location=' + selectedWorld.name)
            .then(response => setSelectedItemCurrentSalePriceEach(response.data))
            .catch(error => console.error('Error fetching sale price:', error));
        }
      }
      if (craftQuantity != craftQuantityInterior) {
        setCraftQuantity(craftQuantityInterior)
      }
    }

    useEffect(() => {
      setSuggestions(craftableItems.map(i => i.name));
    }, [])

    return <>
      <form>
        <label htmlFor="selectedDc">Select your datacenter: </label>
        <select id="selectedDc" name="selectedDc" value={selectedDc.name} onChange={handleSelectedDcChange}>
          {Array.from(datacenters).map(dc =>
            <option key={dc.name} value={dc.name}>{dc.name}</option>
          )}
        </select>
      </form>

      <form>
        <label htmlFor="selected_world">Select your world: </label>
        <select id="selected_world" name="selected_world" value={selectedWorld.name} onChange={handleSelectedWorldChange}>
          {selectedDc.worlds.map(worldId => worlds.filter(w => w.id == worldId).map(w =>
            <option key={w.id} value={w.name}>{w.name}</option>
          )
          )}
        </select>
      </form>
      World hop: <select id="hoptype" onChange={handleSelectedHopTypeChange} value={selectedHopType}>
        <option value={"WORLD"}>My world only</option>
        <option value={"DC"}>Within my DC</option>
        <option value={"REGION"}>Within my Region</option>
      </select>
      <br></br>

      <form onSubmit={handleSearchInputSubmit}>
        search for an item: <input type="text" id="input field" list="suggestions" onChange={handleSearchInputChange} value={searchInput}></input><br></br>
        quantity: <input type="number" id="craftQuantity" value={craftQuantityInterior} onChange={handleSearchQuantityChange} min={1} max={1000}></input><br></br>
        <input type="checkbox" id="hqitems" checked={hq} onChange={handleSelectedHqChange}></input> Prefer HQ crafting materials<br></br>
        <button>Submit</button>
        <datalist id="suggestions">
          {
            suggestions.map((option, index) => <option key={index} value={option} />)
          }
        </datalist> <br></br>
      </form>
    </>
  }

  const SearchResultsDisplay = () => {
    if (searchedItem != "") {
      return <>
        <p>showing results for {searchedItem} x {craftQuantity}</p>
        <p>currently selling for {selectedItemCurrentSalePriceEach || "No marketboard listings "} on {selectedWorld.name}</p>
        <ul>
          {
            recipesForSelectedItem.map(r =>
              <li key={r.id}>
                <RecipeDisplay recipe={r} searchCriteria={{ location: getSelectedLocation(), quantity: craftQuantity, hq: hq }} items={items} productSalePrice={selectedItemCurrentSalePriceEach}></RecipeDisplay>
                <br></br>
              </li>)
          }
        </ul>
      </>
    }

  }
  return (
    <main>
      <div>
        <ItemSearch></ItemSearch>
        <SearchResultsDisplay></SearchResultsDisplay>
      </div>
    </main>
  )
}

export default App
