import type { Component } from "solid-js";
import { Title } from "@solidjs/meta";
import { redirect } from "@solidjs/router";

import styles from "./Login.module.scss";
import button from "../../button.module.scss";

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

        <div class={styles.container}>
            <section class={styles.hero}>
                <h1>Login to your Account</h1>
            </section>

            <form class={styles.form} onsubmit={onsubmit}>
                <p class={styles.signup}>Don't have an account? <a href="/signup">Create a new one →</a></p>

                <div class={styles["form-field"]}>
                    <label for="username">Username</label>
                    <input ref={ele => usernameRef = ele} type="text"
                        id="username" placeholder="Username" required />
                </div>

                <div class={styles["form-field"]}>
                    <label for="password">Password</label>
                    <input ref={ele => passwordRef = ele} type="password"
                        id="password" placeholder="Password" required />
                </div>

                <hr />

                <button type="submit" class={button["primary-btn"]}>login</button>
            </form>
        </div>
    </>;
}

export default Login;
