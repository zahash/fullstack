import type { ParentComponent } from "solid-js";

import { redirect } from "@solidjs/router";
import styles from "./Main.module.css";

async function logout() {
    const response = await fetch("/logout");
    if (response.ok) {
        alert("logout successful");
        throw redirect("/login");
    } else {
        alert("logout failed");
    }
}

const Main: ParentComponent = (props) => {
    return <>
        <header class={`${styles.header} ${styles.content}`}>
            <a href="/">
                <h1>FullStack</h1>
            </a>

            <nav>
                <ul>
                    <li><a href="/login">Login</a></li>
                    <li><a href="/signup">SignUp</a></li>
                    <li><button type="button" onclick={logout}>logout</button></li>
                </ul>
            </nav>

        </header>

        <main class={styles.content}>{props.children}</main>
    </>;
};

export default Main;