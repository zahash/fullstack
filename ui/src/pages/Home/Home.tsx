import { Component } from 'solid-js';
import { Title } from '@solidjs/meta';

import SignUp from '@/src/components/SignUp';
import styles from './Home.module.css';

const Home: Component = () => {
  return <>
    <Title>zahash</Title>
    <p>Home Page</p>
    <SignUp />
  </>;
};

export default Home;
