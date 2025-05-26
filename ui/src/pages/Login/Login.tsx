import type { Component } from "solid-js";
import { Title } from "@solidjs/meta";
import { redirect } from "@solidjs/router";

import Main from "../../layouts/Main";

const Login: Component = () => {
    let usernameRef: HTMLInputElement;
    let passwordRef: HTMLInputElement;

    async function onsubmit(event: Event) {
        event.preventDefault();

        const response = await fetch("/login", {
            method: "POST",
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({
                "username": usernameRef.value,
                "password": passwordRef.value,
            })
        });

        if (response.ok) {
            alert("login successful");
            throw redirect("/");
        } else {
            alert("login failed");
        }
    }

    return <>
        <Title>Login</Title>

        <Main>
            <h2>Login</h2>
            <form id="login-form" onsubmit={onsubmit}>
                <input ref={ele => usernameRef = ele} type="text" placeholder="username" required />
                <input ref={ele => passwordRef = ele} type="password" placeholder="password" required />
                <button type="submit">login</button>
            </form>
        </Main>
    </>;
}

export default Login;
