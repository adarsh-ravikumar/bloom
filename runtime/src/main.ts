import "./style/global.scss"

import { App } from "./core/app";
import { Container } from "./widgets/container";
import { Counter } from "./widgets/counter";

const app = new App("Bloom");

const root = new Container();
root.compose(new Counter());

app.mount(root);

