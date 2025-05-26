import type { Component } from "solid-js";
import { Title } from "@solidjs/meta";

import Main from "../../layouts/Main";

async function fetchPrivate() {
    const response = await fetch("/private");
    if (response.ok) {
        console.log(await response.text());
    } else {
        console.log(await response.json());
    }
}

const Home: Component = () => {
    return <>
        <Title>Home</Title>

        <Main>
            <button id="private-btn" type="button" onclick={fetchPrivate}>private</button>
        </Main>
    </>;
};

export default Home;
