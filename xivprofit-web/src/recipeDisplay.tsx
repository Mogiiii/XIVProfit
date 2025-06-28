import { useState, useEffect } from "react";
import type { Item, Recipe, ItemListing } from "./types";
import { worlds } from "./data";
import { backendUrl } from "./config";
import axios from 'axios';

type SearchCriteria = {
    location: string,
    quantity: number,
    hq: boolean
}

export const RecipeDisplay = ({ recipe: r, searchCriteria, items, productSalePrice }:
    { recipe: Recipe, searchCriteria: SearchCriteria, items: Item[], productSalePrice: number }) => {
    const [listings, setListings] = useState<Map<string, Array<ItemListing>>>(new Map());
    const [ingredientCost, setIngredientCost] = useState<number>(0);
    const [ingredientCostEach, setIngredientCostEach] = useState<number>(0);
    const [profit, setProfit] = useState<number>(0);
    let ingredientPrices: Map<string, number> = new Map();
    let ingredientPricesEach: Map<string, number> = new Map();

    const listingKey = (itemId: number, amount: number) => { return location + "-" + itemId + "|" + amount };

    const getCheapListings = (itemId: number, location: string, amount: number, hq: boolean) => {
        if (listings.get(listingKey(itemId, amount)) == undefined) {
            const path = 'cheapestlistings?item_id=' + itemId + '&location=' + location + '&amount=' + amount + '&hq=' + hq
            axios.get<ItemListing[]>(backendUrl + path).then(resp => {
                listings.set(listingKey(itemId, amount), resp.data)
                setListings(new Map(listings))
            })
        }
    }

    useEffect(() => {
        r.ingredients.map(([id, amount]: [number, number]) =>
            getCheapListings(id, searchCriteria.location, amount * searchCriteria.quantity, searchCriteria.hq)
        )
    }, [searchCriteria])

    useEffect(() => {
        let cost = 0
        ingredientPrices.forEach((val) => {
            cost += val
        });
        setIngredientCost(cost)

        let costEach = 0
        ingredientPricesEach.forEach((val) => {
            costEach += val
        });
        setIngredientCostEach(costEach)
        setProfit(productSalePrice - costEach)

    }, [listings])

    const ListingDisplay = (listing: ItemListing) => {
        return <>
            Buy {listing.quantity} from {listing.retainer_name} on {worlds.find(w => w.id == listing.world_id)?.name} for {listing.total_price}g {listing.hq ? "HQ" : "NQ"} [{listing.price_per_unit}g each]
        </>
    }

    const IngredientDisplay = ({ id, amount }: { id: number, amount: number }) => {
        if (!listings) {
            return <>Undefined listings!</>
        }
        let l = listings.get(listingKey(id, amount));
        if (!l) {
            return <>Loading...</>
        } else {
            // l.map((listing, index) => ingredientPrices[index] = listing.total_price)
            if (l.length > 0) {
                return (
                    <ul> {
                        l.map((listing, index) => {
                            let id = listing.item_id + "-" + index
                            ingredientPrices.set(id, listing.total_price)
                            ingredientPricesEach.set(id, listing.price_per_unit)
                            return (
                                <li key={id}>
                                    {ListingDisplay(listing)}
                                </li>)
                        })
                    } </ul>
                )
            } else {
                return <ul>
                    <li>Not enough marketboard listings found to buy {amount}</li>
                </ul>
            }
        }
    }
    return <>
        Recipe #{r.id}<br></br>
        Cost of ingredients to craft 1: {ingredientCostEach}<br></br>
        Profit per craft: {profit}<br></br>
        Cost to buy everything: {ingredientCost} (May leave you with leftover materials)<br></br>
        Ingredients and where to buy them:
        <ul>
            {r.ingredients.map(([id, amount]: [number, number]) =>
                <li key={id}>
                    {items.find(i => i.id == id)?.name} x {amount} ({amount * searchCriteria.quantity})
                    <br></br>
                    <IngredientDisplay id={id} amount={amount * searchCriteria.quantity}></IngredientDisplay>
                </li>)
            }
        </ul>
        <br></br>
    </>
}