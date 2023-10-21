import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";
import ArcoVue from "@arco-design/web-vue";
import "@arco-design/web-vue/dist/arco.css";
import ArcoVueIcon from '@arco-design/web-vue/es/icon';

const app = createApp(App);
app.use(ArcoVue);
app.use(ArcoVueIcon);

document.body.setAttribute("arco-theme", "dark");

app.mount("#app");