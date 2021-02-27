import Vue from "vue";
import App from "./App.vue";
import router from "./router";
import store from "./store";
import { BootstrapVue, BootstrapVueIcons } from "bootstrap-vue";
import VueCookies from "vue-cookies";
import axios from "axios";

// Install BootstrapVue
Vue.use(BootstrapVue);
Vue.use(BootstrapVueIcons);
Vue.use(VueCookies);

import "@/assets/css/custom.scss";

Vue.config.productionTip = false;
Vue.prototype.$http = axios.create({
  baseURL: process.env.VUE_APP_AXIOS_BASE || "http://localhost:3001"
});

// Add a request interceptor
Vue.prototype.$http.interceptors.request.use(function (config) {
  const token = $cookies.get("token");
  config.headers.Authorization = `Bearer ${token}`;
  return config;
});

new Vue({
  router,
  store,
  render: (h) => h(App),
}).$mount("#app");
