import { HashRouter, Route } from '@solidjs/router'
import { render } from 'solid-js/web'
import { MetaProvider } from '@solidjs/meta'
import { lazy } from 'solid-js';

const Home = lazy(() => import("./pages/Home"));
const Login = lazy(() => import("./pages/Login"));
const SignUp = lazy(() => import("./pages/SignUp"));

render(() =>
    <MetaProvider>
        <HashRouter>
            <Route path="/" component={Home} />
            <Route path="/login" component={Login} />
            <Route path="/signup" component={SignUp} />
        </HashRouter>
    </MetaProvider>
    , document.getElementById('root')!)
