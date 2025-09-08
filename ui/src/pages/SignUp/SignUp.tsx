import { Title } from "@solidjs/meta";
import { createSignal, onMount, type Component } from "solid-js";
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

    const [usernameStatus, setUsernameStatus] = createSignal<{ status?: "ok" | "invalid" | "unavailable"; message?: string }>({});
    const [passwordStatus, setPasswordStatus] = createSignal<{ status?: "ok" | "weak"; message?: string }>({});
    const [emailStatus, setEmailStatus] = createSignal<{ status?: "ok" | "invalid" | "unavailable"; message?: string }>({});

    const canSignUp = () =>
        usernameStatus().status === "ok"
        && passwordStatus().status === "ok"
        && emailStatus().status === "ok"
        ;

    const debounced_checkUsernameAvailability = debounce(async () => {
        const response = await fetch(`/check/username-availability?username=${usernameRef.value}`);
        if (response.status == 200) setUsernameStatus({ status: "ok" });
        else if (response.status == 400) {
            const json = await response.json();
            const message = json["message"];
            setUsernameStatus({ status: "invalid", message });
        }
        else if (response.status == 409) setUsernameStatus({ status: "unavailable" });
        else setUsernameStatus({});
    }, 1000);

    const debounced_checkEmailAvailability = debounce(async () => {
        const response = await fetch(`/check/email-availability?email=${emailRef.value}`);
        if (response.status == 200) setEmailStatus({ status: "ok" });
        else if (response.status == 400) {
            const json = await response.json();
            const message = json["message"];
            setEmailStatus({ status: "invalid", message });
        }
        else if (response.status == 409) setEmailStatus({ status: "unavailable" });
        else setEmailStatus({});
    }, 1000);

    function checkPasswordStrength() {
        const { valid, error } = validate_password(passwordRef.value);
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

    function statusColor(status: string | undefined): string | undefined {
        if (status === "ok") return "var(--success)";
        else if (status === "invalid" || status === "unavailable" || status === "weak") return "var(--error)";
        return undefined;
    }

    return <>
        <Title>Sign Up</Title>

        <div class={styles.container}>
            <section class={styles["left-section"]}>
                <a href="/" class={styles.backlink}>← Back to Home</a>
                <div class={styles.hero}>
                    <h1>Create a new Account</h1>
                </div>
            </section>

            <form class={styles.form} onsubmit={onsubmit}>
                <p class={styles.login}>Already have an account? <a href="/login">Login →</a></p>

                <div class={styles["form-field"]}>
                    <label for="username">Username</label>
                    <input type="text" id="username"
                        ref={ele => usernameRef = ele}
                        oninput={debounced_checkUsernameAvailability}
                        style={{ "border-color": statusColor(usernameStatus().status) }}
                        placeholder="Username" required />
                    <p style={{ color: statusColor(usernameStatus().status) }}>
                        {usernameStatus().status === "invalid" && (usernameStatus().message || "invalid username")}
                        {usernameStatus().status === "unavailable" && "username taken"}
                    </p>
                </div>

                <div class={styles["form-field"]}>
                    <label for="password">Password</label>
                    <input type="password" id="password"
                        ref={ele => passwordRef = ele}
                        oninput={checkPasswordStrength}
                        style={{ "border-color": statusColor(passwordStatus().status) }}
                        placeholder="Password" required />
                    <p style={{ color: statusColor(passwordStatus().status) }}>
                        {passwordStatus().status === "weak" && (passwordStatus().message || "weak password")}
                    </p>
                </div>

                <div class={styles["form-field"]}>
                    <label for="email">Email</label>
                    <input type="email" id="email"
                        ref={ele => emailRef = ele}
                        oninput={debounced_checkEmailAvailability}
                        style={{ "border-color": statusColor(emailStatus().status) }}
                        placeholder="Email" required />
                    <p style={{ color: statusColor(emailStatus().status) }}>
                        {emailStatus().status === "invalid" && (emailStatus().message || "invalid email")}
                        {emailStatus().status === "unavailable" && "email taken"}
                    </p>
                </div>

                <hr />

                <button type="submit" class={button["btn-dark"]} disabled={!canSignUp()}>Sign Up</button>
            </form>
        </div>

    </>;
}

export default SignUp;