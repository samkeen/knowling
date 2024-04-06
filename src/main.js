import {createApp} from "vue";
import "./styles.css";
import App from "./App.vue";
import router from "./router";
import {attachConsole} from "tauri-plugin-log-api";

attachConsole();
createApp(App).use(router).mount("#app");
