import { Component, createSignal } from 'solid-js';

import styles from './Signup.module.css';

const re_special = /[!@#$%^&*()_+\-=\[\]{};':"\\|,\.<>\/?]/;
const re_digit = /\d/;
const re_lowercase = /[a-z]/;
const re_uppercase = /[A-Z]/;

function debounce<F extends (...args: any[]) => any>(fn: F, delay: number): (...args: Parameters<F>) => void {
    let timeout: NodeJS.Timeout | undefined;
    return (...args: Parameters<F>) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn(...args), delay);
    };
}

const SignUp: Component = () => {
    let usernameRef: HTMLInputElement;
    let passwordRef: HTMLInputElement;

    let [usernameStatus, set_usernameStatus] = createSignal<"ok" | "unavailable" | "invalid" | undefined>(undefined);
    let [passwordHasSpecial, set_passwordHasSpecial] = createSignal(false);
    let [passwordHasDigit, set_passwordHasDigit] = createSignal(false);
    let [passwordHasLowercase, set_passwordHasLowercase] = createSignal(false);
    let [passwordHasUppercase, set_passwordHasUppercase] = createSignal(false);

    let canSubmit = () =>
        usernameStatus() === "ok" &&
        passwordHasSpecial() &&
        passwordHasDigit() &&
        passwordHasLowercase() &&
        passwordHasUppercase();

    const debounced_checkUsernameAvailability = debounce(async () => {
        const response = await fetch("/check-username-availability", {
            method: "POST",
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({
                "username": usernameRef.value,
            })
        });

        if (response.status == 200) set_usernameStatus("ok");
        else if (response.status == 400) set_usernameStatus("invalid");
        else if (response.status == 409) set_usernameStatus("unavailable");
        else set_usernameStatus(undefined);
    }, 1000);

    const checkPasswordStrength = () => {
        const password = passwordRef.value;

        set_passwordHasSpecial(re_special.test(password));
        set_passwordHasDigit(re_digit.test(password));
        set_passwordHasLowercase(re_lowercase.test(password));
        set_passwordHasUppercase(re_uppercase.test(password));
    }

    const onsubmit = async (e: SubmitEvent) => {
        e.preventDefault();

        const response = await fetch("/signup", {
            method: "POST",
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: new URLSearchParams({
                "username": usernameRef.value,
                "password": passwordRef.value,
            })
        });

        if (!response.ok) {
            const err = await response.json();
            console.log(err);
        }
    }

    return (
        <form onsubmit={onsubmit} class={styles.SignUp}>
            <input ref={ele => usernameRef = ele} type='text' placeholder='username' required
                oninput={() => { set_usernameStatus(undefined); debounced_checkUsernameAvailability(); }} />
            <p style={{ display: usernameStatus() === "unavailable" ? "block" : "none" }}>username taken</p>
            <p style={{ display: usernameStatus() === "invalid" ? "block" : "none" }}>invalid username</p>

            <input ref={ele => passwordRef = ele} oninput={checkPasswordStrength} type='password' placeholder='password' required />
            <p style={{ display: passwordHasSpecial() ? "none" : "block" }}>password must have atleast one special character</p>
            <p style={{ display: passwordHasDigit() ? "none" : "block" }}>password must have atleast one digit</p>
            <p style={{ display: passwordHasLowercase() ? "none" : "block" }}>password must have atleast one lowercase letter</p>
            <p style={{ display: passwordHasUppercase() ? "none" : "block" }}>password must have atleast one uppercase letter</p>

            <button type='submit' disabled={!canSubmit()}>sign up</button>
        </form>
    );
};

export default SignUp;
