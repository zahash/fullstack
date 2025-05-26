import { Title } from "@solidjs/meta";
import { createEffect, createSignal, onMount, type Component } from "solid-js";
import { redirect } from "@solidjs/router";

import init, { validate_password } from "@lib/wasm/wasm";
import debounce from "@lib/utils/debounce";

import Main from "../../layouts/Main";

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

        <Main>
            <h2>Sign Up</h2>
            <form id="signup-form" onsubmit={onsubmit}>
                <input type="text" oninput={debounced_checkUsernameAvailability} ref={ele => usernameRef = ele} required placeholder="username" />
                <span id="signup-msg-username">
                    {{
                        unavailable: "username taken",
                        invalid: usernameStatus().message || "invalid username",
                    }[usernameStatus().status || ""] || ""}
                </span>

                <input type="password" oninput={checkPasswordStrength} ref={ele => passwordRef = ele} required placeholder="password" />
                <span id="signup-msg-password">
                    {passwordStatus().status === "weak" ? passwordStatus().message : ""}
                </span>

                <input type="email" oninput={debounced_checkEmailAvailability} ref={ele => emailRef = ele} required placeholder="email" />
                <span id="signup-msg-email">
                    {{
                        unavailable: "email taken",
                        invalid: emailStatus().message || "invalid email",
                    }[emailStatus().status || ""] || ""}
                </span>

                <button type="submit" disabled={!canSignUp()}>sign up</button>
            </form>
        </Main>
    </>;
}

export default SignUp;