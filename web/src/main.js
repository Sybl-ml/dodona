import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import { BootstrapVue, BootstrapVueIcons } from "bootstrap-vue";
import VueCookies from "vue-cookies";
import axios from "axios";
import VueNativeSock from "vue-native-websocket";

// Install BootstrapVue
Vue.use(BootstrapVue);
Vue.use(BootstrapVueIcons);
Vue.use(VueCookies);

let ws_route = "";
if (window.location.protocol.startsWith("https")) ws_route += "wss://";
else ws_route += "ws://";

if (window.location.hostname === "localhost") ws_route += "localhost:3001";
else ws_route += window.location.hostname;

Vue.use(VueNativeSock, ws_route + "/project_updates", {
  reconnection: true,
  reconnectionAttempts: 5,
  reconnectionDelay: 3000,
  format: "json",
  store: store,
  passToStoreHandler: function (eventName, event, next) {
    if (eventName === "SOCKET_onmessage") {
      store.dispatch("SOCKET_ONMESSAGE", JSON.parse(event.data))
    }
    next(eventName, event)
  }
});

import "@/assets/css/custom.scss";

Vue.config.productionTip = false;
Vue.prototype.$http = axios.create({
  baseURL: process.env.VUE_APP_AXIOS_BASE || "http://localhost:3001",
});

// Add a request interceptor
Vue.prototype.$http.interceptors.request.use(function(config) {
  const token = $cookies.get("token");
  config.headers.Authorization = `Bearer ${token}`;
  return config;
});

Vue.config.devtools = true;
new Vue({
  router,
  store,
  render: (h) => h(App),
}).$mount("#app");
