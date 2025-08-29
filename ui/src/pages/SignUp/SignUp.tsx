import { Title } from "@solidjs/meta";
import { createEffect, createSignal, onMount, type Component } from "solid-js";
import { redirect } from "@solidjs/router";

import init, { validate_password } from "@lib/wasm/wasm";
import debounce from "@lib/utils/debounce";

import styles from "./Signup.module.scss";
import button from "../../button.module.scss";

const SignUp: Component = () => {
    onMount(async () => await init());

    let usernameRef: HTMLInputElement;
    let passwordRef: HTMLInputElement;
    let emailRef: HTMLInputElement;

    const [usernameStatus, setUsernameStatus] = createSignal<{ status?: string; message?: string }>({});
    const [passwordStatus, setPasswordStatus] = createSignal<{ status?: string; message?: string }>({});
    const [emailStatus, setEmailStatus] = createSignal<{ status?: string; message?: string }>({});

    const canSignUp = () =>
        usernameStatus().status === "ok"
        && passwordStatus().status === "ok"
        && emailStatus().status === "ok"
        ;

    createEffect(() => {
        if (!usernameRef) return;

        const { status, message } = usernameStatus();
        if (status === "unavailable") {
            usernameRef.setCustomValidity("Username is taken");
            usernameRef.reportValidity();
        }
        else if (status === "invalid") {
            usernameRef.setCustomValidity(message || "Invalid username");
            usernameRef.reportValidity();
        }
        else {
            usernameRef.setCustomValidity("");
        }
    });

    createEffect(() => {
        if (!passwordRef) return;

        const { status, message } = passwordStatus();
        if (status === "weak") {
            passwordRef.setCustomValidity(message || "Weak password");
            passwordRef.reportValidity();
        }
        else {
            passwordRef.setCustomValidity("");
        }
    });

    createEffect(() => {
        if (!emailRef) return;

        const { status, message } = emailStatus();
        if (status === "unavailable") {
            emailRef.setCustomValidity("Email is taken");
            emailRef.reportValidity();
        }
        else if (status === "invalid") {
            emailRef.setCustomValidity(message || "Invalid email");
            emailRef.reportValidity();
        }
        else {
            emailRef.setCustomValidity("");
        }
    });

    const debounced_checkUsernameAvailability = debounce(async () => {
        setUsernameStatus({});
        const response = await fetch(`/check/username-availability?username=${usernameRef.value}`);
        if (response.status == 200) setUsernameStatus({ status: "ok" });
        else if (response.status == 400) {
            const json = await response.json();
            const message = json["error"];
            setUsernameStatus({ status: "invalid", message });
        }
        else if (response.status == 409) setUsernameStatus({ status: "unavailable" });
        else setUsernameStatus({});
    }, 1000);

    const debounced_checkEmailAvailability = debounce(async () => {
        setEmailStatus({});
        const response = await fetch(`/check/email-availability?email=${emailRef.value}`);
        if (response.status == 200) setEmailStatus({ status: "ok" });
        else if (response.status == 400) {
            const json = await response.json();
            const message = json["error"];
            setEmailStatus({ status: "invalid", message });
        }
        else if (response.status == 409) setEmailStatus({ status: "unavailable" });
        else setEmailStatus({});
    }, 1000);

    function checkPasswordStrength() {
        const password = passwordRef.value;
        const { valid, error } = validate_password(password);
        setPasswordStatus({
            status: valid ? "ok" : "weak",
            message: error
        });
    }

    async function onsubmit(event: Event) {
        event.preventDefault();

        const response = await fetch("/signup", {
            method: "POST",
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({
                "username": usernameRef.value,
                "password": passwordRef.value,
                "email": emailRef.value,
            })
        });

        if (response.ok) {
            alert("signup successful!");
            throw redirect("/login");
        }
        else alert(JSON.stringify(await response.json()));
    }

    return <>
        <Title>Sign Up</Title>

        <div class={styles.container}>
            <section class={styles.hero}>
                <h1>Create a new Account</h1>
            </section>

            <form class={styles.form} onsubmit={onsubmit}>
                <p class={styles.login}>Already have an account? <a href="/login">Login →</a></p>

                <div class={styles["form-field"]}>
                    <label for="username">Username</label>
                    <input type="text" id="username"
                        ref={ele => usernameRef = ele}
                        oninput={debounced_checkUsernameAvailability}
                        placeholder="Username" required />
                </div>

                <div class={styles["form-field"]}>
                    <label for="password">Password</label>
                    <input type="password" id="password"
                        ref={ele => passwordRef = ele}
                        oninput={checkPasswordStrength}
                        placeholder="Password" required />
                </div>

                <div class={styles["form-field"]}>
                    <label for="email">Email</label>
                    <input type="email" id="email"
                        ref={ele => emailRef = ele}
                        oninput={debounced_checkEmailAvailability}
                        placeholder="Email" required />
                </div>

                <hr />

                <button type="submit" class={button["primary-btn"]} disabled={!canSignUp()}>Sign Up</button>
            </form>
        </div>

    </>;
}

export default SignUp;