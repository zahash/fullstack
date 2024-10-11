import { Component } from "solid-js";

import styles from './Login.module.css';

const Login: Component = () => {
    let usernameRef: HTMLInputElement;
    let passwordRef: HTMLInputElement;
    let rememberRef: HTMLInputElement;

    const onsubmit = async (e: SubmitEvent) => {
        e.preventDefault();

        const response = await fetch("/login", {
          method: "POST",
          credentials: "include",
          headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
          body: new URLSearchParams({
            "username": usernameRef.value,
            "password": passwordRef.value,
            "remember": rememberRef.checked ? "true" : "false"
          })
        });

        if (response.ok) {
          console.log("login successful");
        } else {
          console.log(await response.json());
        }

        console.log(usernameRef.value);
        console.log(passwordRef.value);
        console.log(rememberRef.checked);
    }
  
    return (
        <form onsubmit={onsubmit} class={styles.SignUp}>
            <input ref={ele => usernameRef = ele} type='text' placeholder='username' required/>
            <input ref={ele => passwordRef = ele} type='password' placeholder='password' required />
            <label>
                <input ref={ele => rememberRef = ele} type="checkbox" />
                Remember me
            </label>
            <button type='submit'>login</button>
        </form>
    );
}

export default Login;
