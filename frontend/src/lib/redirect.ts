import {key} from "./store";
import axios from 'axios';
import {get} from "svelte/store";

export default async function redirect() {
    const api_key = get(key)
    // get value of r parameter from URL
    const url = new URL(window.location.href)
    const r = url.searchParams.get('r')
    // if r is not null, redirect to the value of r
    if (r === null || api_key === '') {
        return
    }
    //url decode r
    const decoded = decodeURIComponent(r)
    const data = new FormData();
    data.append('src', decoded);
    await axios.post('https://www.premiumize.me/api/transfer/create?apikey=' + api_key, data, {
        headers: {
            'Content-Type': 'multipart/form-data',
            accept: 'application/json'
        }
    }).then(response => {
        if (response.data.status !== 'error') {
            window.location.href = 'https://www.premiumize.me/transfers'
        } else {
            console.error(response)
        }
    }).catch(error => {
        console.error(error)
    })
}