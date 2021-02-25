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

// Set a base URL and then override it with something more specific
let baseURL = "http://localhost:3001";

if (process.env.VUE_APP_AXIOS_BASE !== undefined) {
  baseURL = process.env.VUE_APP_AXIOS_BASE;
}

const axiosConfig = {
  baseURL: baseURL,
};

Vue.config.productionTip = false;
Vue.prototype.$http = axios.create(axiosConfig);

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
