import { Component } from 'solid-js';
import { Title } from '@solidjs/meta';

import SignUp from '@/src/components/SignUp';
import Login from '@/src/components/Login';

const privateUrl = `${import.meta.env.VITE_API_BASE_URL}/private`;

const Home: Component = () => {
  return <>
    <Title>zahash</Title>
    <p>Home Page</p>
    <SignUp />
    <Login />

    <button onclick={async () => {
      const response = await fetch(privateUrl, {credentials: 'include'});
      if (response.ok) {
        console.log(response.body);
      } else {
        console.log(await response.json());
      }
    }}> private </button>
  </>;
};

export default Home;
